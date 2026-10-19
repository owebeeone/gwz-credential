use crate::{provider::*, state::*, *};
use std::{
    sync::{Arc, Mutex, atomic::AtomicBool},
    task::Waker,
};

pub(crate) struct SourceAuthority {
    pub control: AuthControl,
    pub purpose: AcquisitionPurpose,
}
pub(crate) struct SourceRecord {
    pub operation: Arc<Operation>,
    pub id: u64,
    pub choice: ExchangeSelection,
    pub control: SourceControl,
    pub provider_bytes: usize,
    pub work_bytes: usize,
    pub observation: AcquisitionObservation,
    pub driving: AtomicBool,
    pub state: Mutex<SourceState>,
}
pub(crate) struct SourceState {
    pub purpose: Option<AcquisitionPurpose>,
    pub snapshot_charge: Option<SelectedCharge>,
    pub material: Option<Arc<dyn SelectedCredential>>,
    pub provider: Option<Box<dyn ProviderAcquisition>>,
    pub started: bool,
    pub reason: Option<AuthError>,
    pub cleanup: bool,
    pub transferred: bool,
    pub disposed: bool,
    pub waker: Option<Waker>,
}
impl SourceRecord {
    pub fn error(&self, authority: &Authority, state: &SourceState, now: u64) -> Option<AuthError> {
        if state.transferred {
            return None;
        }
        state
            .reason
            .or_else(|| self.operation.hub.check(authority, &self.operation).err())
            .or_else(|| self.control.check_at(now).err())
    }
    pub fn seal(&self, state: &mut SourceState, reason: AuthError) {
        if !state.transferred {
            state.reason.get_or_insert(reason);
            self.control.stop();
            let mut facts = lock(&self.observation.0.facts);
            if facts.terminal.is_none() {
                facts.terminal = Some(AcquisitionTerminal {
                    reason: state.reason,
                    material_published: false,
                });
            }
        }
    }
    pub fn cancel(&self, reason: AuthError) {
        let wake = {
            let _authority = lock(&self.operation.hub.state);
            let mut state = lock(&self.state);
            self.seal(&mut state, reason);
            state.waker.take()
        };
        if let Some(waker) = wake {
            waker.wake();
        }
        self.operation.hub.ports.progress.wake();
    }
    pub fn release_bytes(&self, authority: &mut Authority, bytes: usize) {
        authority.usage.retained_bytes -= bytes;
        lock(&self.operation.state).bytes -= bytes;
    }
}
