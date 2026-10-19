//! Controlled source acquisition; the returned Future yields a prepared snapshot once.
use crate::{acquisition_state::*, provider::*, state::*, *};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, atomic::AtomicBool},
    task::{Context, Poll},
};

pub enum AcquisitionPurpose {
    BeforeLookup,
    VerifiedRequest {
        peer: PeerContext,
        challenge: AuthChallenge,
    },
}
impl AcquisitionPurpose {
    fn charge(&self) -> Result<usize, AuthError> {
        match self {
            Self::BeforeLookup => Ok(0),
            Self::VerifiedRequest { peer, challenge } => {
                peer.validate()?;
                peer.verified_peer
                    .capacity()
                    .checked_add(peer.channel_binding.capacity())
                    .and_then(|n| n.checked_add(peer.endpoint.capacity() + peer.target.bytes()))
                    .and_then(|n| n.checked_add(challenge.0.0.capacity()))
                    .ok_or(AuthError::InputBudget)
            }
        }
    }
}
#[derive(Clone)]
pub struct SourceControl {
    pub(crate) deadline: u64,
    pub(crate) request_cancel: Cancellation,
    pub(crate) stopped: Cancellation,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) progress: Arc<dyn Progress>,
}
impl SourceControl {
    pub fn check(&self) -> Result<(), AuthError> {
        self.check_at(self.clock.now_ms())
    }
    pub(crate) fn check_at(&self, now: u64) -> Result<(), AuthError> {
        if self.stopped.is_cancelled() || self.request_cancel.is_cancelled() {
            return Err(AuthError::Cancelled);
        }
        if now >= self.deadline {
            return Err(AuthError::Timeout);
        }
        Ok(())
    }
    pub fn wake(&self) {
        self.progress.wake();
    }
    pub(crate) fn stop(&self) {
        self.stopped.cancel();
    }
}
pub struct CredentialAcquiring {
    pub(crate) record: Option<Arc<SourceRecord>>,
    pub(crate) observation: Option<AcquisitionObservation>,
    pub(crate) failure: Option<AuthError>,
    pub(crate) completed: Option<AcquisitionTerminal>,
}
impl CredentialContext {
    pub fn acquire(
        &self,
        selection: ExchangeSelection,
        purpose: AcquisitionPurpose,
        control: AuthControl,
    ) -> CredentialAcquiring {
        match self.register_source(selection, purpose, control) {
            Ok(record) => CredentialAcquiring {
                observation: Some(record.observation.clone()),
                record: Some(record),
                failure: None,
                completed: None,
            },
            Err(error) => CredentialAcquiring {
                record: None,
                observation: None,
                failure: Some(error),
                completed: None,
            },
        }
    }
    fn register_source(
        &self,
        selection: ExchangeSelection,
        purpose: AcquisitionPurpose,
        control: AuthControl,
    ) -> Result<Arc<SourceRecord>, AuthError> {
        selection.validate()?;
        if !self.operation.input.allowed.contains(&selection) {
            return Err(AuthError::Unsupported);
        }
        if let AcquisitionPurpose::VerifiedRequest { peer, .. } = &purpose
            && (peer.endpoint != self.operation.input.endpoint || peer.target != selection.target)
        {
            return Err(AuthError::PeerMismatch);
        }
        let hub = &self.operation.hub;
        let source_control = SourceControl {
            deadline: control.deadline_ms,
            request_cancel: control.cancellation,
            stopped: Cancellation::default(),
            clock: hub.ports.clock.clone(),
            progress: hub.ports.progress.clone(),
        };
        source_control.check()?;
        let charge = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            hub.provider.acquisition_charge(&selection, &purpose)
        }))
        .unwrap_or(Err(AuthError::ProviderFault))?;
        let snapshot_bytes = selection
            .charge()?
            .checked_add(purpose.charge()?)
            .and_then(|n| n.checked_add(charge.snapshot_bytes))
            .ok_or(AuthError::InputBudget)?;
        // Covers the source record, transient duplicate choice and fixed Arc/worker envelopes.
        let work_bytes = selection
            .charge()?
            .checked_add(4096)
            .and_then(|n| n.checked_add(charge.work_bytes))
            .ok_or(AuthError::InputBudget)?;
        let observation_bytes = selection
            .charge()?
            .checked_add(selection.target.bytes())
            .and_then(|n| {
                n.checked_add(
                    std::mem::size_of::<crate::acquisition_observation::SourceObserved>()
                        + std::mem::size_of::<SourceRecord>()
                        + 256,
                )
            })
            .ok_or(AuthError::InputBudget)?;
        let total = snapshot_bytes
            .checked_add(work_bytes)
            .and_then(|n| n.checked_add(observation_bytes))
            .ok_or(AuthError::InputBudget)?;
        let snapshot_reservation = hub.ports.retention.reserve(RetentionCharge {
            bytes: snapshot_bytes,
            records: 0,
        })?;
        let work_reservation = hub.ports.retention.reserve(RetentionCharge {
            bytes: work_bytes,
            records: 1,
        })?;
        let observation_reservation = hub.ports.retention.reserve(RetentionCharge {
            bytes: observation_bytes,
            records: 0,
        })?;
        let now = hub.ports.clock.now_ms();
        hub.charge(&self.operation, total)?;
        let result = {
            let mut authority = lock(&hub.state);
            let valid = hub
                .check(&authority, &self.operation)
                .and_then(|()| source_control.check_at(now));
            let id = authority.serial.checked_add(1);
            if let Err(error) = valid {
                Err(error)
            } else if authority.usage.conversations + authority.usage.acquisitions
                >= hub.config.max_conversations
            {
                Err(AuthError::Capacity)
            } else if let Some(id) = id {
                authority.serial = id;
                let observation = AcquisitionObservation(Arc::new(
                    crate::acquisition_observation::SourceObserved {
                        operation: self.operation.clone(),
                        id,
                        target: selection.target.clone(),
                        kind: SourceKind::of(&selection.source),
                        bytes: observation_bytes,
                        _reservation: observation_reservation,
                        facts: Default::default(),
                    },
                ));
                let record = Arc::new(SourceRecord {
                    operation: self.operation.clone(),
                    id,
                    choice: selection,
                    control: source_control,
                    provider_bytes: charge.snapshot_bytes,
                    work_bytes,
                    observation,
                    driving: AtomicBool::new(false),
                    state: std::sync::Mutex::new(SourceState {
                        purpose: Some(purpose),
                        snapshot_charge: Some(SelectedCharge {
                            operation: self.operation.clone(),
                            bytes: snapshot_bytes,
                            _reservation: snapshot_reservation,
                        }),
                        material: None,
                        provider: None,
                        started: false,
                        reason: None,
                        cleanup: false,
                        transferred: false,
                        disposed: false,
                        waker: None,
                    }),
                });
                authority.sources.insert(id, Arc::downgrade(&record));
                authority.usage.acquisitions += 1;
                Ok((record, work_reservation))
            } else {
                Err(AuthError::Capacity)
            }
        };
        match result {
            Ok((record, work_reservation)) => {
                // The registered record owns all effects before any worker can start.
                hub.ports.retention.retain(record.clone(), work_reservation);
                hub.ports.progress.wake();
                Ok(record)
            }
            Err(error) => {
                let mut authority = lock(&hub.state);
                authority.usage.retained_bytes -= total;
                lock(&self.operation.state).bytes -= total;
                Err(error)
            }
        }
    }
}
impl CredentialAcquiring {
    pub fn observation(&self) -> Option<AcquisitionObservation> {
        self.observation.clone()
    }
    pub fn cancel(mut self) -> AcquisitionEnd {
        if self.completed.is_none() {
            let reason = self.failure.unwrap_or(AuthError::Cancelled);
            if let Some(record) = &self.record {
                record.cancel(reason);
            }
            self.completed = Some(
                self.observation
                    .as_ref()
                    .and_then(|o| o.terminal())
                    .unwrap_or(AcquisitionTerminal {
                        reason: Some(reason),
                        material_published: false,
                    }),
            );
        }
        let terminal = self.completed.unwrap();
        AcquisitionEnd {
            reason: terminal.reason,
            material_published: terminal.material_published,
            cleanup: self.observation.take().map(|o| o.cleanup()),
        }
    }
}
impl Future for CredentialAcquiring {
    type Output = Result<PreparedCredential, AuthError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed.is_some() {
            return Poll::Ready(Err(AuthError::WrongState));
        }
        if let Some(error) = self.failure.take() {
            self.completed = Some(AcquisitionTerminal {
                reason: Some(error),
                material_published: false,
            });
            return Poll::Ready(Err(error));
        }
        let record = self.record.as_ref().expect("live source").clone();
        let now = record.control.clock.now_ms();
        let result = {
            let authority = lock(&record.operation.hub.state);
            let mut state = lock(&record.state);
            if let Some(error) = record.error(&authority, &state, now) {
                record.seal(&mut state, error);
                Some(Err(error))
            } else if state.cleanup && state.material.is_some() {
                let selected = Arc::new(Selected {
                    operation: record.operation.clone(),
                    choice: record.choice.clone(),
                    provider: state.material.take().unwrap(),
                    _charge: state.snapshot_charge.take().unwrap(),
                    source: Some(SourceAuthority {
                        control: AuthControl {
                            deadline_ms: record.control.deadline,
                            cancellation: record.control.request_cancel.clone(),
                        },
                        purpose: state.purpose.take().unwrap(),
                    }),
                });
                state.transferred = true;
                lock(&record.observation.0.facts).terminal = Some(AcquisitionTerminal {
                    reason: None,
                    material_published: true,
                });
                Some(Ok(PreparedCredential { selected }))
            } else {
                state.waker = Some(cx.waker().clone());
                None
            }
        };
        if let Some(result) = result {
            self.completed = self.observation.as_ref().and_then(|o| o.terminal());
            self.record = None;
            record.operation.hub.ports.progress.wake();
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}
impl Drop for CredentialAcquiring {
    fn drop(&mut self) {
        if self.completed.is_none()
            && let Some(record) = &self.record
        {
            record.cancel(AuthError::Cancelled);
        }
    }
}

/// Endpoint-local exact source equality; never checkout authorization or a key getter.
#[derive(Clone)]
pub struct SourceIdentity(Arc<Selected>);
impl PreparedCredential {
    pub fn source_identity(&self) -> SourceIdentity {
        SourceIdentity(self.selected.clone())
    }
}
impl SourceIdentity {
    pub fn same_material(&self, other: &Self) -> bool {
        if !Arc::ptr_eq(&self.0.operation.hub, &other.0.operation.hub)
            || self.0.operation.input.endpoint != other.0.operation.input.endpoint
            || self.0.choice.target != other.0.choice.target
            || SourceKind::of(&self.0.choice.source) != SourceKind::of(&other.0.choice.source)
        {
            return false;
        }
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.0.provider.same_material(other.0.provider.as_ref())
                && other.0.provider.same_material(self.0.provider.as_ref())
        }))
        .unwrap_or(false)
    }
}
