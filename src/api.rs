use crate::{provider::*, state::*, *};
use std::sync::{Arc, Mutex, atomic::AtomicBool};

pub struct CredentialOwner {
    pub(crate) hub: Arc<Hub>,
}
#[derive(Clone)]
pub struct CredentialHandle {
    pub(crate) hub: Arc<Hub>,
}
pub struct OperationCredentials {
    pub(crate) operation: Arc<Operation>,
}
#[derive(Clone)]
pub struct CredentialContext {
    pub(crate) operation: Arc<Operation>,
}
pub struct PreparedCredential {
    pub(crate) selected: Arc<Selected>,
}
pub struct AuthSession {
    pub(crate) child: Option<Arc<Child>>,
}

pub fn build(
    config: CredentialConfig,
    providers: ProviderSet,
    ports: CredentialPorts,
) -> Result<(CredentialOwner, CredentialHandle), BuildError> {
    config.validate()?;
    let hub = Arc::new(Hub {
        config,
        ports,
        provider: providers.0,
        state: Mutex::default(),
    });
    Ok((
        CredentialOwner { hub: hub.clone() },
        CredentialHandle { hub },
    ))
}
impl CredentialOwner {
    pub fn close(self) -> ProviderClosing {
        self.hub.close(None);
        ProviderClosing {
            hub: self.hub.clone(),
        }
    }
}
impl Drop for CredentialOwner {
    fn drop(&mut self) {
        self.hub.close(None);
    }
}
impl CredentialHandle {
    pub fn prepare(&self, input: CredentialInput) -> Result<OperationCredentials, PrepareError> {
        let bytes = input.charge()?;
        let caller = input.caller.identity();
        let reservation = self
            .hub
            .ports
            .retention
            .reserve(RetentionCharge { bytes, records: 0 })?;
        let mut authority = lock(&self.hub.state);
        if authority.closed {
            return Err(AuthError::ProviderClosed);
        }
        if authority.usage.operations >= self.hub.config.max_operations {
            return Err(AuthError::Capacity);
        }
        let total = authority
            .usage
            .retained_bytes
            .checked_add(bytes)
            .ok_or(AuthError::InputBudget)?;
        if bytes > self.hub.config.max_operation_bytes || total > self.hub.config.max_bytes {
            return Err(AuthError::InputBudget);
        }
        let id = authority.serial.checked_add(1).ok_or(AuthError::Capacity)?;
        authority.serial = id;
        let operation = Arc::new(Operation {
            hub: self.hub.clone(),
            id,
            caller,
            input,
            state: Mutex::new(OperationState {
                closed: false,
                bytes,
            }),
            _reservation: reservation,
        });
        authority.operations.insert(id, Arc::downgrade(&operation));
        authority.usage.operations += 1;
        authority.usage.retained_bytes = total;
        Ok(OperationCredentials { operation })
    }
    pub fn snapshot(&self) -> Snapshot {
        lock(&self.hub.state).usage
    }
}
impl OperationCredentials {
    pub fn context(&self) -> CredentialContext {
        CredentialContext {
            operation: self.operation.clone(),
        }
    }
    pub fn close(self) -> OperationClosing {
        self.operation.hub.close(Some(&self.operation));
        OperationClosing {
            operation: self.operation.clone(),
        }
    }
}
impl Drop for OperationCredentials {
    fn drop(&mut self) {
        self.operation.hub.close(Some(&self.operation));
    }
}
impl CredentialContext {
    pub fn select(&self, selection: ExchangeSelection) -> Result<PreparedCredential, SelectError> {
        selection.validate()?;
        if !self.operation.input.allowed.contains(&selection) {
            return Err(AuthError::Unsupported);
        }
        let hub = &self.operation.hub;
        let provider_bytes = hub.provider.selection_charge(&selection)?;
        let bytes = selection
            .charge()?
            .checked_add(provider_bytes)
            .ok_or(AuthError::InputBudget)?;
        let reservation = hub
            .ports
            .retention
            .reserve(RetentionCharge { bytes, records: 0 })?;
        hub.charge(&self.operation, bytes)?;
        // Selection is source admission only: providers must not start native auth here.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let provider = hub.provider.select(&self.operation.input, &selection)?;
            if provider.retained_bytes() > provider_bytes {
                return Err(AuthError::InputBudget);
            }
            Ok(provider)
        }))
        .unwrap_or(Err(AuthError::ProviderFault));
        let provider = match result {
            Ok(provider) => provider,
            Err(error) => {
                let mut authority = lock(&hub.state);
                lock(&self.operation.state).bytes -= bytes;
                authority.usage.retained_bytes -= bytes;
                return Err(error);
            }
        };
        let selected = Arc::new(Selected {
            operation: self.operation.clone(),
            choice: selection,
            provider,
            bytes,
            _reservation: reservation,
        });
        let valid = {
            let authority = lock(&hub.state);
            hub.check(&authority, &self.operation)
        };
        valid?;
        Ok(PreparedCredential { selected })
    }
}
impl PreparedCredential {
    pub fn begin(self, peer: PeerContext, control: AuthControl) -> AuthOpening {
        match self.register(peer, control) {
            Ok(child) => AuthOpening {
                child: Some(child),
                error: None,
            },
            Err(error) => AuthOpening {
                child: None,
                error: Some(error),
            },
        }
    }
    fn register(self, peer: PeerContext, control: AuthControl) -> Result<Arc<Child>, AuthError> {
        peer.validate()?;
        let operation = self.selected.operation.clone();
        let hub = operation.hub.clone();
        if peer.endpoint != operation.input.endpoint || peer.target != self.selected.choice.target {
            return Err(AuthError::PeerMismatch);
        }
        if control.cancellation.is_cancelled() {
            return Err(AuthError::Cancelled);
        }
        if hub.ports.clock.now_ms() >= control.deadline_ms {
            return Err(AuthError::Timeout);
        }
        let scope = self.selected.provider.scope();
        if scope == SecurityScope::Connection && self.selected.choice.source != Source::WindowsLogon
        {
            return Err(AuthError::Unsupported);
        }
        let observation_bytes = peer
            .verified_peer
            .capacity()
            .checked_add(peer.channel_binding.capacity())
            .and_then(|n| n.checked_add(peer.endpoint.capacity() + peer.target.bytes()))
            .and_then(|n| n.checked_add(std::mem::size_of::<crate::observation::Observed>() + 64))
            .ok_or(AuthError::InputBudget)?;
        let observation_reservation = hub.ports.retention.reserve(RetentionCharge {
            bytes: observation_bytes,
            records: 0,
        })?;
        let record_bytes = (2 * 65536usize + 2048)
            .checked_add(self.selected.provider.conversation_charge())
            .ok_or(AuthError::InputBudget)?;
        let reservation = hub.ports.retention.reserve(RetentionCharge {
            bytes: record_bytes,
            records: 1,
        })?;
        // Reserve a fixed challenge/output envelope before any provider work.
        let total_bytes = record_bytes
            .checked_add(observation_bytes)
            .ok_or(AuthError::InputBudget)?;
        hub.charge(&operation, total_bytes)?;
        let registered = {
            let mut authority = lock(&hub.state);
            let valid = hub.check(&authority, &operation);
            if let Err(error) = valid {
                Err(error)
            } else if authority.usage.conversations >= hub.config.max_conversations {
                Err(AuthError::Capacity)
            } else if let Some(id) = authority.serial.checked_add(1) {
                authority.serial = id;
                let peer = Arc::new(peer);
                let receipt = CleanupReceipt(Arc::new(crate::observation::Observed {
                    operation: operation.clone(),
                    id,
                    peer: peer.clone(),
                    bytes: observation_bytes,
                    _reservation: observation_reservation,
                    facts: Mutex::default(),
                }));
                let child = Arc::new(Child {
                    hub: hub.clone(),
                    operation: operation.clone(),
                    id,
                    peer,
                    control,
                    scope,
                    record_bytes,
                    driving: AtomicBool::new(false),
                    receipt,
                    inner: Mutex::new(ChildState {
                        selected: Some(self.selected.clone()),
                        provider: None,
                        started: false,
                        start_result: None,
                        challenge: None,
                        response: None,
                        terminal: false,
                        reason: None,
                        peer_result: PeerAuthResult::Unknown,
                        committed: false,
                        cleanup: false,
                        disposed: false,
                        outputs: 0,
                        scopes: 0,
                        scope_pending: false,
                        scope_transferred: false,
                        key_calls: 0,
                        key_cleanup: None,
                        external_active: false,
                        rounds: 0,
                        waker: None,
                    }),
                });
                authority.children.insert(id, Arc::downgrade(&child));
                authority.usage.conversations += 1;
                Ok(child)
            } else {
                Err(AuthError::Capacity)
            }
        };
        let child = match registered {
            Ok(child) => child,
            Err(error) => {
                let mut authority = lock(&hub.state);
                lock(&operation.state).bytes -= total_bytes;
                authority.usage.retained_bytes -= total_bytes;
                return Err(error);
            }
        };
        hub.ports.retention.retain(child.clone(), reservation);
        hub.ports.progress.wake();
        Ok(child)
    }
}
impl AuthSession {
    pub fn observation(&self) -> AuthObservation {
        self.child
            .as_ref()
            .expect("live session")
            .receipt
            .observation()
    }
    pub fn respond(&mut self, challenge: AuthChallenge) -> AuthResponse<'_> {
        let child = self.child.as_ref().expect("live session").clone();
        let now = child.hub.ports.clock.now_ms();
        let error = {
            let authority = lock(&child.hub.state);
            let mut state = lock(&child.inner);
            if let Some(error) = child.error(&authority, &state, now) {
                Some(error)
            } else if state.terminal || state.challenge.is_some() || state.response.is_some() {
                Some(AuthError::WrongState)
            } else if state.outputs != 0 || state.key_calls != 0 || state.external_active {
                Some(AuthError::WouldBlock)
            } else if state.rounds >= child.hub.config.max_rounds {
                Some(AuthError::ProviderRejected)
            } else {
                state.rounds += 1;
                state.challenge = Some(challenge);
                None
            }
        };
        child.hub.ports.progress.wake();
        AuthResponse {
            child,
            error,
            completed: false,
            borrow: std::marker::PhantomData,
        }
    }
    pub fn finish(mut self, peer: PeerAuthResult) -> AuthFinishing {
        let child = self.child.take().expect("live session");
        let now = child.hub.ports.clock.now_ms();
        {
            let authority = lock(&child.hub.state);
            let mut state = lock(&child.inner);
            if !state.terminal {
                let reason = child.error(&authority, &state, now);
                state.terminate(reason, peer);
                state.scope_pending = reason.is_none()
                    && peer == PeerAuthResult::Accepted
                    && child.scope == SecurityScope::Connection;
            }
            child.publish_terminal(&state);
        }
        child.hub.ports.progress.wake();
        AuthFinishing {
            child,
            completed: false,
        }
    }
    pub fn cancel(mut self) -> AuthClosing {
        let child = self.child.take().expect("live session");
        child.cancel(AuthError::Cancelled);
        AuthFinishing {
            child,
            completed: false,
        }
    }
}
impl Drop for AuthSession {
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            child.cancel(AuthError::Cancelled);
        }
    }
}
