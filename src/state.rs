use crate::{provider::*, *};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard, Weak, atomic::AtomicBool},
    task::Waker,
};

pub(crate) fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
pub(crate) struct Hub {
    pub config: CredentialConfig,
    pub ports: CredentialPorts,
    pub provider: Arc<dyn Provider>,
    pub state: Mutex<Authority>,
}
#[derive(Default)]
pub(crate) struct Authority {
    pub closed: bool,
    pub serial: u64,
    pub usage: Snapshot,
    pub operations: BTreeMap<u64, Weak<Operation>>,
    pub children: BTreeMap<u64, Weak<Child>>,
    pub sources: BTreeMap<u64, Weak<crate::acquisition_state::SourceRecord>>,
}
pub(crate) struct Operation {
    pub hub: Arc<Hub>,
    pub id: u64,
    pub caller: u128,
    pub input: CredentialInput,
    pub state: Mutex<OperationState>,
    pub _reservation: Box<dyn Reservation>,
}
pub(crate) struct OperationState {
    pub closed: bool,
    pub bytes: usize,
}
impl Drop for Operation {
    fn drop(&mut self) {
        let mut authority = lock(&self.hub.state);
        authority.operations.remove(&self.id);
        authority.usage.operations -= 1;
        authority.usage.retained_bytes -= lock(&self.state).bytes;
    }
}
pub(crate) struct Selected {
    pub operation: Arc<Operation>,
    pub choice: ExchangeSelection,
    pub provider: Arc<dyn SelectedCredential>,
    pub source: Option<crate::acquisition_state::SourceAuthority>,
    // Last field: material/source payload drops before its accounting is released.
    pub _charge: SelectedCharge,
}
pub(crate) struct SelectedCharge {
    pub operation: Arc<Operation>,
    pub bytes: usize,
    pub _reservation: Box<dyn Reservation>,
}
impl Drop for SelectedCharge {
    fn drop(&mut self) {
        let mut authority = lock(&self.operation.hub.state);
        lock(&self.operation.state).bytes -= self.bytes;
        authority.usage.retained_bytes -= self.bytes;
    }
}
pub(crate) struct Child {
    pub hub: Arc<Hub>,
    pub operation: Arc<Operation>,
    pub id: u64,
    pub peer: Arc<PeerContext>,
    pub control: AuthControl,
    pub scope: SecurityScope,
    pub record_bytes: usize,
    pub driving: AtomicBool,
    pub inner: Mutex<ChildState>,
    pub receipt: CleanupReceipt,
}
pub(crate) struct ChildState {
    pub selected: Option<Arc<Selected>>,
    pub provider: Option<Box<dyn ProviderSession>>,
    pub started: bool,
    pub start_result: Option<Result<(), AuthError>>,
    pub challenge: Option<AuthChallenge>,
    pub response: Option<Result<ProviderStep, AuthError>>,
    pub terminal: bool,
    pub reason: Option<AuthError>,
    pub peer_result: PeerAuthResult,
    pub committed: bool,
    pub cleanup: bool,
    pub disposed: bool,
    pub outputs: usize,
    pub scopes: usize,
    pub scope_pending: bool,
    pub scope_transferred: bool,
    pub key_calls: usize,
    pub key_cleanup: Option<Arc<dyn KeyCleanup>>,
    pub external_active: bool,
    pub rounds: usize,
    pub waker: Option<Waker>,
}
impl Hub {
    pub fn check(&self, authority: &Authority, operation: &Operation) -> Result<(), AuthError> {
        if authority.closed {
            return Err(AuthError::ProviderClosed);
        }
        if lock(&operation.state).closed {
            return Err(AuthError::OperationClosed);
        }
        Ok(())
    }
    pub fn charge(&self, operation: &Operation, bytes: usize) -> Result<(), AuthError> {
        let mut authority = lock(&self.state);
        self.check(&authority, operation)?;
        let mut state = lock(&operation.state);
        let op_bytes = state
            .bytes
            .checked_add(bytes)
            .ok_or(AuthError::InputBudget)?;
        let total = authority
            .usage
            .retained_bytes
            .checked_add(bytes)
            .ok_or(AuthError::InputBudget)?;
        if op_bytes > self.config.max_operation_bytes || total > self.config.max_bytes {
            return Err(AuthError::InputBudget);
        }
        state.bytes = op_bytes;
        authority.usage.retained_bytes = total;
        Ok(())
    }
    pub fn close(&self, operation: Option<&Operation>) {
        let mut wakes = Vec::new();
        let children;
        let sources;
        {
            let mut authority = lock(&self.state);
            if let Some(operation) = operation {
                lock(&operation.state).closed = true;
            } else {
                authority.closed = true;
            }
            sources = authority
                .sources
                .values()
                .filter_map(Weak::upgrade)
                .collect::<Vec<_>>();
            for source in &sources {
                if operation.is_none_or(|operation| operation.id == source.operation.id) {
                    let mut state = lock(&source.state);
                    source.seal(
                        &mut state,
                        if operation.is_some() {
                            AuthError::OperationClosed
                        } else {
                            AuthError::ProviderClosed
                        },
                    );
                    if let Some(waker) = state.waker.take() {
                        wakes.push(waker);
                    }
                }
            }
            children = authority
                .children
                .values()
                .filter_map(Weak::upgrade)
                .collect::<Vec<_>>();
            for child in &children {
                if operation.is_none_or(|operation| operation.id == child.operation.id) {
                    let mut state = lock(&child.inner);
                    if !state.terminal {
                        let reason = if operation.is_some() {
                            AuthError::OperationClosed
                        } else {
                            AuthError::ProviderClosed
                        };
                        state.terminate(Some(reason), PeerAuthResult::Unknown);
                    }
                    if operation.is_none() {
                        lock(&child.receipt.0.facts).revoked = true;
                        if state.scope_pending && state.reason.is_none() {
                            state.reason = Some(AuthError::ProviderClosed);
                        }
                        state.scope_pending = false;
                    } else if state.scope_pending {
                        state.scope_pending = false;
                        state.reason.get_or_insert(AuthError::OperationClosed);
                    }
                    child.publish_terminal(&state);
                    if let Some(waker) = state.waker.take() {
                        wakes.push(waker);
                    }
                }
            }
        }
        for wake in wakes {
            wake.wake();
        }
        self.ports.progress.wake();
    }
}
impl ChildState {
    pub fn terminate(&mut self, reason: Option<AuthError>, peer: PeerAuthResult) {
        if !self.terminal {
            self.terminal = true;
            self.reason = reason;
            self.peer_result = peer;
            self.challenge = None;
        }
    }
}
impl Child {
    pub fn error(&self, authority: &Authority, state: &ChildState, now: u64) -> Option<AuthError> {
        if let Some(reason) = state.reason {
            return Some(reason);
        }
        if authority.closed {
            return Some(AuthError::ProviderClosed);
        }
        if lock(&self.operation.state).closed && !state.terminal {
            return Some(AuthError::OperationClosed);
        }
        if (!state.terminal || state.scope_pending)
            && let Some(source) = state.selected.as_ref().and_then(|s| s.source.as_ref())
        {
            if source.control.cancellation.is_cancelled() {
                return Some(AuthError::Cancelled);
            }
            if now >= source.control.deadline_ms {
                return Some(AuthError::Timeout);
            }
        }
        if (!state.terminal || state.scope_pending) && self.control.cancellation.is_cancelled() {
            return Some(AuthError::Cancelled);
        }
        if (!state.terminal || state.scope_pending) && now >= self.control.deadline_ms {
            return Some(AuthError::Timeout);
        }
        None
    }
    pub fn cancel(&self, reason: AuthError) {
        let wake = {
            let _authority = lock(&self.hub.state);
            let mut state = lock(&self.inner);
            state.terminate(Some(reason), PeerAuthResult::Unknown);
            self.publish_terminal(&state);
            state.waker.take()
        };
        if let Some(wake) = wake {
            wake.wake();
        }
        self.hub.ports.progress.wake();
    }
}

impl Child {
    pub fn refresh_pending(&self, authority: &Authority, state: &mut ChildState, now: u64) {
        if (!state.terminal || state.scope_pending)
            && let Some(error) = self.error(authority, state, now)
        {
            if state.terminal {
                state.reason.get_or_insert(error);
                state.scope_pending = false;
            } else {
                state.terminate(Some(error), PeerAuthResult::Unknown);
            }
        }
        self.publish_terminal(state);
    }
    pub fn publish_terminal(&self, state: &ChildState) {
        if state.terminal && !state.scope_pending {
            let mut facts = lock(&self.receipt.0.facts);
            if facts.terminal.is_none() {
                facts.terminal = Some(TerminalRecord {
                    reason: state.reason,
                    peer_result: state.peer_result,
                    publication_committed: state.committed,
                    disposition: if state.reason.is_some()
                        || state.peer_result != PeerAuthResult::Accepted
                        || !state.scope_transferred && self.scope == SecurityScope::Connection
                    {
                        PhysicalDisposition::Retire
                    } else if state.scope_transferred {
                        PhysicalDisposition::BoundScope
                    } else {
                        PhysicalDisposition::RequestSettled
                    },
                });
            }
        }
    }
}
