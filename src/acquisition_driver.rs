use crate::{acquisition_state::*, provider::*, state::*, *};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::Ordering,
    task::{Context, Poll},
};

enum SourceWork {
    Start(AcquisitionPurpose),
    Acquire(Box<dyn ProviderAcquisition>),
    Close(Box<dyn ProviderAcquisition>, bool),
    None,
}
struct Driving<'a>(&'a SourceRecord);
impl Drop for Driving<'_> {
    fn drop(&mut self) {
        self.0.driving.store(false, Ordering::Release);
    }
}
impl RetainedWork for SourceRecord {
    fn observation(&self) -> WorkObservation {
        WorkObservation::Acquisition(self.observation.clone())
    }
    fn advance(&self, cx: &mut Context<'_>) -> Poll<()> {
        if self.driving.swap(true, Ordering::AcqRel) {
            return Poll::Pending;
        }
        let _driving = Driving(self);
        let now = self.control.clock.now_ms();
        let work = {
            let authority = lock(&self.operation.hub.state);
            let mut state = lock(&self.state);
            if let Some(error) = self.error(&authority, &state, now) {
                self.seal(&mut state, error);
            }
            if state.reason.is_some() || state.material.is_some() {
                let cancelled = state.reason.is_some();
                if let Some(provider) = state.provider.take() {
                    SourceWork::Close(provider, cancelled)
                } else {
                    if !state.started {
                        state.cleanup = true;
                    }
                    SourceWork::None
                }
            } else if !state.started {
                state.started = true;
                SourceWork::Start(state.purpose.take().unwrap())
            } else {
                state
                    .provider
                    .take()
                    .map_or(SourceWork::None, SourceWork::Acquire)
            }
        };
        match work {
            SourceWork::Start(purpose) => {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    self.operation.hub.provider.start_acquire(
                        &self.operation.input,
                        &self.choice,
                        &purpose,
                        self.control.clone(),
                    )
                }));
                let _authority = lock(&self.operation.hub.state);
                let mut state = lock(&self.state);
                state.purpose = Some(purpose);
                match result {
                    Ok(Ok(provider)) => state.provider = Some(provider),
                    Ok(Err(error)) => {
                        self.seal(&mut state, error);
                        state.cleanup = true;
                    }
                    Err(_) => {
                        // Unknown start effects cannot be declared disposed.
                        self.seal(&mut state, AuthError::ProviderFault);
                    }
                }
            }
            SourceWork::Acquire(mut provider) => {
                let result = catch_unwind(AssertUnwindSafe(|| provider.poll_acquire(cx)))
                    .unwrap_or(Poll::Ready(Err(AuthError::ProviderFault)));
                // Foreign metadata callbacks must not run under either kernel lock.
                let result = result.map(|result| {
                    result.map(|material| {
                        let valid = catch_unwind(AssertUnwindSafe(|| material.retained_bytes()))
                            .map_err(|_| AuthError::ProviderFault)
                            .and_then(|bytes| {
                                if bytes <= self.provider_bytes {
                                    Ok(())
                                } else {
                                    Err(AuthError::InputBudget)
                                }
                            });
                        (material, valid)
                    })
                });
                let _authority = lock(&self.operation.hub.state);
                let mut state = lock(&self.state);
                state.provider = Some(provider);
                if let Poll::Ready(result) = result {
                    match result {
                        Ok((material, validation)) => {
                            // Even invalid material stays owned until confirmed closure.
                            state.material = Some(material);
                            if let Err(error) = validation {
                                self.seal(&mut state, error);
                            }
                        }
                        Err(error) => self.seal(&mut state, error),
                    }
                }
            }
            SourceWork::Close(mut provider, cancelled) => {
                let result = catch_unwind(AssertUnwindSafe(|| provider.poll_close(cx, cancelled)))
                    .unwrap_or(Poll::Ready(Err(AuthError::ProviderFault)));
                let retired = {
                    let _authority = lock(&self.operation.hub.state);
                    let mut state = lock(&self.state);
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
            SourceWork::None => {}
        }
        let now = self.control.clock.now_ms();
        let (done, discarded, purpose, permit, wake) = {
            let mut authority = lock(&self.operation.hub.state);
            let mut state = lock(&self.state);
            if let Some(error) = self.error(&authority, &state, now) {
                self.seal(&mut state, error);
            }
            if state.cleanup {
                lock(&self.observation.0.facts).cleanup = true;
            }
            let done = state.cleanup && (state.reason.is_some() || state.transferred);
            let mut discarded = None;
            let mut purpose = None;
            let mut permit = None;
            if done && !state.disposed {
                state.disposed = true;
                authority.sources.remove(&self.id);
                authority.usage.acquisitions -= 1;
                self.release_bytes(&mut authority, self.work_bytes);
                discarded = state.material.take();
                purpose = state.purpose.take();
                permit = state.snapshot_charge.take();
            }
            (done, discarded, purpose, permit, state.waker.take())
        };
        drop(discarded);
        drop(purpose);
        drop(permit);
        if let Some(waker) = wake {
            waker.wake();
        }
        if done { Poll::Ready(()) } else { Poll::Pending }
    }
}
