use crate::{provider::ProviderStep, state::*, *};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub use crate::state::CleanupReceipt;

pub struct AuthOpening {
    pub(crate) child: Option<Arc<Child>>,
    pub(crate) error: Option<AuthError>,
}
impl AuthOpening {
    pub fn wait(self) -> Self {
        self
    }
}
impl Future for AuthOpening {
    type Output = Result<AuthSession, AuthFailure>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(error) = self.error.take() {
            return Poll::Ready(Err(error));
        }
        let child = self
            .child
            .as_ref()
            .expect("opening polled after completion");
        let now = child.hub.ports.clock.now_ms();
        let result = {
            let authority = lock(&child.hub.state);
            let mut state = lock(&child.inner);
            if let Some(error) = child.error(&authority, &state, now) {
                Some(Err(error))
            } else if let Some(result) = state.start_result {
                Some(result)
            } else {
                state.waker = Some(cx.waker().clone());
                None
            }
        };
        match result {
            Some(Ok(())) => Poll::Ready(Ok(AuthSession {
                child: self.child.take(),
            })),
            Some(Err(error)) => {
                if let Some(child) = self.child.take() {
                    child.cancel(error);
                }
                Poll::Ready(Err(error))
            }
            None => Poll::Pending,
        }
    }
}
impl Drop for AuthOpening {
    fn drop(&mut self) {
        if let Some(child) = &self.child {
            child.cancel(AuthError::Cancelled);
        }
    }
}
pub enum AuthStep {
    AwaitPeer,
    ProviderComplete,
    Response(AuthOutput),
}
impl std::fmt::Debug for AuthStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::AwaitPeer => "AwaitPeer",
            Self::ProviderComplete => "ProviderComplete",
            Self::Response(_) => "Response([redacted])",
        })
    }
}
pub struct AuthResponse<'a> {
    pub(crate) child: Arc<Child>,
    pub(crate) error: Option<AuthError>,
    pub(crate) completed: bool,
    pub(crate) borrow: PhantomData<&'a mut AuthSession>,
}
impl Future for AuthResponse<'_> {
    type Output = Result<AuthStep, AuthFailure>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(error) = self.error.take() {
            self.completed = true;
            return Poll::Ready(Err(error));
        }
        let now = self.child.hub.ports.clock.now_ms();
        let result = {
            let authority = lock(&self.child.hub.state);
            let mut state = lock(&self.child.inner);
            if let Some(error) = self.child.error(&authority, &state, now) {
                Some(Err(error))
            } else {
                let result = state.response.take();
                if let Some(Ok(ProviderStep::Response(_))) = &result {
                    state.outputs += 1;
                }
                if result.is_none() {
                    state.waker = Some(cx.waker().clone());
                }
                result
            }
        };
        if let Some(result) = result {
            self.completed = true;
            Poll::Ready(result.map(|step| match step {
                ProviderStep::AwaitPeer => AuthStep::AwaitPeer,
                ProviderStep::Complete => AuthStep::ProviderComplete,
                ProviderStep::Response(bytes) => AuthStep::Response(AuthOutput {
                    child: self.child.clone(),
                    bytes: Some(bytes),
                }),
            }))
        } else {
            Poll::Pending
        }
    }
}
impl Drop for AuthResponse<'_> {
    fn drop(&mut self) {
        if !self.completed {
            self.child.cancel(AuthError::Cancelled);
        }
    }
}

pub struct AuthEnd {
    pub reason: Option<AuthError>,
    pub peer_result: PeerAuthResult,
    pub publication_committed: bool,
    pub disposition: PhysicalDisposition,
    pub cleanup: CleanupReceipt,
    pub scope: Option<ScopeLease>,
}
pub struct AuthFinishing {
    pub(crate) child: Arc<Child>,
    pub(crate) completed: bool,
}
pub type AuthClosing = AuthFinishing;
impl Future for AuthFinishing {
    type Output = AuthEnd;
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<AuthEnd> {
        assert!(!self.completed, "terminal future polled after completion");
        let end = {
            let authority = lock(&self.child.hub.state);
            let mut state = lock(&self.child.inner);
            let reason = if authority.closed {
                Some(AuthError::ProviderClosed)
            } else {
                state.reason
            };
            let scope = if reason.is_none() && state.scope_pending {
                state.scopes += 1;
                Some(ScopeLease {
                    child: self.child.clone(),
                })
            } else {
                None
            };
            state.scope_pending = false;
            let disposition = if reason.is_some() || state.peer_result != PeerAuthResult::Accepted {
                PhysicalDisposition::Retire
            } else if scope.is_some() {
                PhysicalDisposition::BoundScope
            } else {
                PhysicalDisposition::RequestSettled
            };
            AuthEnd {
                reason,
                peer_result: state.peer_result,
                publication_committed: state.committed,
                disposition,
                cleanup: self.child.receipt.clone(),
                scope,
            }
        };
        self.completed = true;
        self.child.hub.ports.progress.wake();
        Poll::Ready(end)
    }
}
impl Drop for AuthFinishing {
    fn drop(&mut self) {
        if !self.completed {
            lock(&self.child.inner).scope_pending = false;
        }
        self.child.hub.ports.progress.wake();
    }
}

/// Native conversation cleanup report. Captured inputs and prepared selections
/// remain charged while their handles survive; Confirmed does not erase those pins.
pub struct ProviderEnd {
    pub cleanup: CleanupStatus,
    pub pending_records: usize,
}
pub struct ProviderClosing {
    pub(crate) hub: Arc<Hub>,
}
impl Future for ProviderClosing {
    type Output = ProviderEnd;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<ProviderEnd> {
        let records = lock(&self.hub.state).usage.conversations;
        Poll::Ready(ProviderEnd {
            cleanup: if records == 0 {
                CleanupStatus::Confirmed
            } else {
                CleanupStatus::Pending
            },
            pending_records: records,
        })
    }
}
pub struct OperationClosing {
    pub(crate) operation: Arc<Operation>,
}
impl Future for OperationClosing {
    type Output = ProviderEnd;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<ProviderEnd> {
        let children: Vec<_> = lock(&self.operation.hub.state)
            .children
            .values()
            .filter_map(std::sync::Weak::upgrade)
            .collect();
        let records = children
            .iter()
            .filter(|c| c.operation.id == self.operation.id)
            .count();
        Poll::Ready(ProviderEnd {
            cleanup: if records == 0 {
                CleanupStatus::Confirmed
            } else {
                CleanupStatus::Pending
            },
            pending_records: records,
        })
    }
}
