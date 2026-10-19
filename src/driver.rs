use crate::{provider::*, state::*, *};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, atomic::Ordering},
    task::{Context, Poll},
};

enum Work {
    Create(Arc<Selected>),
    Start(Box<dyn ProviderSession>),
    Respond(Box<dyn ProviderSession>, AuthChallenge),
    Close(Box<dyn ProviderSession>, bool),
    None,
}
struct Driving<'a>(&'a Child);
impl Drop for Driving<'_> {
    fn drop(&mut self) {
        self.0.driving.store(false, Ordering::Release);
    }
}

impl RetainedWork for Child {
    fn advance(&self, cx: &mut Context<'_>) -> Poll<()> {
        if self.driving.swap(true, Ordering::AcqRel) {
            return Poll::Pending;
        }
        let _driving = Driving(self);
        let now = self.hub.ports.clock.now_ms();
        let work = {
            let authority = lock(&self.hub.state);
            let mut state = lock(&self.inner);
            if !state.terminal
                && let Some(error) = self.error(&authority, &state, now)
            {
                state.terminate(Some(error), PeerAuthResult::Unknown);
            }
            let work = if state.key_calls != 0 {
                Work::None
            } else if state.terminal {
                if let Some(provider) = state.provider.take() {
                    Work::Close(provider, state.reason.is_some())
                } else {
                    if !state.started {
                        state.cleanup = true;
                    }
                    Work::None
                }
            } else if !state.started {
                state.started = true;
                Work::Create(state.selected.as_ref().expect("admitted selection").clone())
            } else if state.start_result.is_none() {
                state.provider.take().map_or(Work::None, Work::Start)
            } else if let Some(challenge) = state.challenge.take() {
                if let Some(provider) = state.provider.take() {
                    Work::Respond(provider, challenge)
                } else {
                    state.challenge = Some(challenge);
                    Work::None
                }
            } else {
                Work::None
            };
            state.external_active = !matches!(work, Work::None);
            work
        };
        // Every provider call occurs outside both authority and payload mutexes.
        match work {
            Work::Create(selected) => {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    selected.provider.start(
                        &self.peer,
                        self.control.deadline_ms,
                        self.control.cancellation.clone(),
                    )
                }));
                let _authority = lock(&self.hub.state);
                let mut state = lock(&self.inner);
                match result {
                    Ok(Ok(provider)) => state.provider = Some(provider),
                    Ok(Err(error)) => {
                        state.start_result = Some(Err(error));
                        state.terminate(Some(error), PeerAuthResult::Unknown);
                        state.cleanup = true; // start Err is contractually effect-free.
                    }
                    Err(_) => {
                        state.start_result = Some(Err(AuthError::ProviderFault));
                        state.terminate(Some(AuthError::ProviderFault), PeerAuthResult::Unknown);
                        // Unknown effects: keep retained capacity; never invent disposal.
                    }
                }
            }
            Work::Start(mut provider) => {
                let result = catch_unwind(AssertUnwindSafe(|| provider.poll_start(cx)))
                    .unwrap_or(Poll::Ready(Err(AuthError::ProviderFault)));
                let _authority = lock(&self.hub.state);
                let mut state = lock(&self.inner);
                state.provider = Some(provider);
                if let Poll::Ready(result) = result {
                    if let Err(error) = result {
                        state.terminate(Some(error), PeerAuthResult::Unknown);
                    }
                    state.start_result = Some(result);
                }
            }
            Work::Respond(mut provider, challenge) => {
                let result =
                    catch_unwind(AssertUnwindSafe(|| provider.poll_respond(cx, &challenge)))
                        .unwrap_or(Poll::Ready(Err(AuthError::ProviderFault)));
                let _authority = lock(&self.hub.state);
                let mut state = lock(&self.inner);
                state.provider = Some(provider);
                if !state.terminal {
                    match result {
                        Poll::Pending => state.challenge = Some(challenge),
                        Poll::Ready(result) => {
                            if let Err(error) = &result {
                                state.terminate(Some(*error), PeerAuthResult::Unknown);
                            }
                            state.response = Some(result);
                        }
                    }
                }
            }
            Work::Close(mut provider, cancelled) => {
                let result = catch_unwind(AssertUnwindSafe(|| provider.poll_close(cx, cancelled)))
                    .unwrap_or(Poll::Ready(Err(AuthError::ProviderFault)));
                let retired = {
                    let _authority = lock(&self.hub.state);
                    let mut state = lock(&self.inner);
                    if matches!(result, Poll::Ready(Ok(()))) {
                        state.cleanup = true;
                        Some(provider)
                    } else {
                        state.provider = Some(provider);
                        None
                    }
                };
                drop(retired);
            }
            Work::None => {}
        }
        let probe = lock(&self.inner).key_cleanup.clone();
        let key_disposed = probe.is_none_or(|probe| {
            catch_unwind(AssertUnwindSafe(|| probe.confirmed())).unwrap_or(false)
        });
        let (done, selected, discarded, wake) = {
            let mut authority = lock(&self.hub.state);
            let mut state = lock(&self.inner);
            state.external_active = false;
            if state.cleanup && key_disposed {
                *lock(&self.receipt.0) = CleanupStatus::Confirmed;
            }
            let discarded = if state.terminal {
                state.response.take()
            } else {
                None
            };
            let done = state.cleanup
                && key_disposed
                && state.outputs == 0
                && state.scopes == 0
                && !state.scope_pending
                && state.key_calls == 0;
            let selected = if done && !state.disposed {
                state.disposed = true;
                authority.children.remove(&self.id);
                authority.usage.conversations -= 1;
                authority.usage.retained_bytes -= self.record_bytes;
                lock(&self.operation.state).bytes -= self.record_bytes;
                state.selected.take()
            } else {
                None
            };
            (done, selected, discarded, state.waker.take())
        };
        drop(selected);
        drop(discarded);
        if let Some(wake) = wake {
            wake.wake();
        }
        if done { Poll::Ready(()) } else { Poll::Pending }
    }
}
