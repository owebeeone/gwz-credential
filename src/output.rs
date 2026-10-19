use crate::{provider::LocalSshKeyPort, state::*, *};
use std::{collections::VecDeque, sync::Arc};

pub struct AuthOutput {
    pub(crate) child: Arc<Child>,
    pub(crate) bytes: Option<SecretBytes>,
}
struct Queued {
    child: Arc<Child>,
    bytes: Option<SecretBytes>,
}
impl Drop for Queued {
    fn drop(&mut self) {
        lock(&self.child.inner).outputs -= 1;
        self.child.hub.ports.progress.wake();
    }
}
pub struct EndpointSendQueue {
    peer: PeerContext,
    capacity: usize,
    queue: VecDeque<Queued>,
}
/// Only the executing endpoint's trusted native send adapter implements this port.
pub trait LocalAuthSink {
    fn peer(&self) -> &PeerContext;
    fn send(&mut self, bytes: &[u8]) -> Result<(), AuthError>;
}
impl EndpointSendQueue {
    pub fn new(peer: PeerContext, capacity: usize) -> Result<Self, AuthError> {
        peer.validate()?;
        if capacity == 0 || capacity > 64 {
            return Err(AuthError::InvalidInput);
        }
        Ok(Self {
            peer,
            capacity,
            queue: VecDeque::new(),
        })
    }
    pub fn deliver_next(&mut self, sink: &mut dyn LocalAuthSink) -> Result<bool, AuthError> {
        if sink.peer() != &self.peer {
            return Err(AuthError::PeerMismatch);
        }
        let Some(queued) = self.queue.pop_front() else {
            return Ok(false);
        };
        if let Some(bytes) = &queued.bytes {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sink.send(&bytes.0)))
                .unwrap_or(Err(AuthError::ProviderFault))?;
        }
        // The output was already committed: cancellation cannot unsend it.
        Ok(true)
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
impl AuthOutput {
    pub fn commit(&mut self, queue: &mut EndpointSendQueue) -> Result<(), AuthError> {
        if self.bytes.is_none() {
            return Err(AuthError::WrongState);
        }
        let now = self.child.hub.ports.clock.now_ms();
        let authority = lock(&self.child.hub.state);
        let mut state = lock(&self.child.inner);
        if let Some(error) = self.child.error(&authority, &state, now) {
            return Err(error);
        }
        if state.terminal {
            return Err(AuthError::WrongState);
        }
        if queue.peer != self.child.peer {
            return Err(AuthError::PeerMismatch);
        }
        if queue.queue.len() >= queue.capacity {
            return Err(AuthError::WouldBlock);
        }
        queue.queue.push_back(Queued {
            child: self.child.clone(),
            bytes: self.bytes.take(),
        });
        state.committed = true;
        Ok(())
    }
}
impl Drop for AuthOutput {
    fn drop(&mut self) {
        if self.bytes.is_some() {
            lock(&self.child.inner).outputs -= 1;
            self.child.hub.ports.progress.wake();
        }
    }
}

pub struct ScopeLease {
    pub(crate) child: Arc<Child>,
}
pub struct ScopeGrant<'a> {
    lease: &'a ScopeLease,
    operation: Arc<Operation>,
    peer: PeerContext,
    control: AuthControl,
}
impl ScopeLease {
    pub fn grant<'a>(
        &'a self,
        context: &CredentialContext,
        peer: PeerContext,
        control: AuthControl,
    ) -> Result<ScopeGrant<'a>, AuthError> {
        if !Arc::ptr_eq(&self.child.hub, &context.operation.hub) {
            return Err(AuthError::PeerMismatch);
        }
        let same_identity = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.child
                .operation
                .input
                .caller
                .same_identity(context.operation.input.caller.as_ref())
                && context
                    .operation
                    .input
                    .caller
                    .same_identity(self.child.operation.input.caller.as_ref())
        }))
        .unwrap_or(false);
        if !same_identity {
            return Err(AuthError::PeerMismatch);
        }
        let authority = lock(&self.child.hub.state);
        self.child.hub.check(&authority, &context.operation)?;
        let state = lock(&self.child.inner);
        let choice = &state.selected.as_ref().ok_or(AuthError::WrongState)?.choice;
        if !context.operation.input.allowed.contains(choice)
            || context.operation.input.endpoint != peer.endpoint
            || self.child.operation.caller != context.operation.caller
            || peer != self.child.peer
        {
            return Err(AuthError::PeerMismatch);
        }
        Ok(ScopeGrant {
            lease: self,
            operation: context.operation.clone(),
            peer,
            control,
        })
    }
}
impl ScopeGrant<'_> {
    pub fn commit(self, queue: &mut EndpointSendQueue) -> Result<(), AuthError> {
        let child = &self.lease.child;
        if self.control.cancellation.is_cancelled() {
            return Err(AuthError::Cancelled);
        }
        if child.hub.ports.clock.now_ms() >= self.control.deadline_ms {
            return Err(AuthError::Timeout);
        }
        let authority = lock(&child.hub.state);
        child.hub.check(&authority, &self.operation)?;
        if self.peer != queue.peer {
            return Err(AuthError::PeerMismatch);
        }
        if queue.queue.len() >= queue.capacity {
            return Err(AuthError::WouldBlock);
        }
        let mut state = lock(&child.inner);
        state.outputs += 1;
        queue.queue.push_back(Queued {
            child: child.clone(),
            bytes: None,
        });
        Ok(())
    }
}
impl Drop for ScopeLease {
    fn drop(&mut self) {
        lock(&self.child.inner).scopes -= 1;
        self.child.hub.ports.progress.wake();
    }
}

struct KeyUse(Arc<Child>);
impl Drop for KeyUse {
    fn drop(&mut self) {
        lock(&self.0.inner).key_calls -= 1;
        self.0.hub.ports.progress.wake();
    }
}
impl AuthSession {
    pub fn use_ssh_key(
        &mut self,
        port: &mut dyn LocalSshKeyPort,
    ) -> Result<SshProgress, AuthFailure> {
        let child = self.child.as_ref().expect("live session");
        let port_peer = port.peer().clone();
        let cleanup = port.key_cleanup();
        let now = child.hub.ports.clock.now_ms();
        let selected = {
            let authority = lock(&child.hub.state);
            let mut state = lock(&child.inner);
            if let Some(error) = child.error(&authority, &state, now) {
                return Err(error);
            }
            if state.terminal
                || state.challenge.is_some()
                || state.external_active
                || state.response.is_some()
            {
                return Err(AuthError::WrongState);
            }
            if port_peer != child.peer {
                return Err(AuthError::PeerMismatch);
            }
            if !matches!(child.peer.target, Target::Ssh { .. }) {
                return Err(AuthError::Unsupported);
            }
            let selected = state.selected.as_ref().expect("admitted key").clone();
            if !matches!(selected.choice.source, Source::ExplicitKey(_)) {
                return Err(AuthError::Unsupported);
            }
            if state
                .key_cleanup
                .as_ref()
                .is_some_and(|old| !Arc::ptr_eq(old, &cleanup))
            {
                return Err(AuthError::PeerMismatch);
            }
            state.key_cleanup = Some(cleanup);
            state.key_calls += 1;
            selected
        };
        let _pin = KeyUse(child.clone());
        let key = selected
            .provider
            .private_key()
            .ok_or(AuthError::Unsupported)?;
        // No library state lock is held while native authentication runs.
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| port.authenticate(key)))
                .unwrap_or(Err(AuthError::ProviderFault));
        let now = child.hub.ports.clock.now_ms();
        {
            let authority = lock(&child.hub.state);
            let state = lock(&child.inner);
            if let Some(error) = child.error(&authority, &state, now) {
                return Err(error);
            }
        }
        if result == Ok(SshProgress::Rejected) {
            child.cancel(AuthError::ProviderRejected);
        }
        result
    }
}
