//! Charged observation without cancellation, scope-transfer or release authority.
use crate::{provider::Reservation, state::*, *};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalRecord {
    pub reason: Option<AuthError>,
    pub peer_result: PeerAuthResult,
    pub publication_committed: bool,
    pub disposition: PhysicalDisposition,
}
#[derive(Default)]
pub(crate) struct Facts {
    pub terminal: Option<TerminalRecord>,
    pub cleanup: Option<CleanupStatus>,
    pub revoked: bool,
}
pub(crate) struct Observed {
    pub operation: Arc<Operation>,
    pub id: u64,
    pub peer: Arc<PeerContext>,
    pub bytes: usize,
    pub _reservation: Box<dyn Reservation>,
    pub facts: Mutex<Facts>,
}
impl Drop for Observed {
    fn drop(&mut self) {
        let mut authority = lock(&self.operation.hub.state);
        lock(&self.operation.state).bytes -= self.bytes;
        authority.usage.retained_bytes -= self.bytes;
    }
}
#[derive(Clone)]
pub struct AuthObservation(pub(crate) Arc<Observed>);
impl AuthObservation {
    pub fn id(&self) -> (u128, u64) {
        (self.0.operation.hub.config.namespace, self.0.id)
    }
    pub fn peer(&self) -> &PeerContext {
        &self.0.peer
    }
    /// The immutable decision appears when the outcome is fixed. An unconsumed
    /// potential scope stays pending until transfer, cancellation, deadline or Drop.
    pub fn terminal(&self) -> Option<TerminalRecord> {
        lock(&self.0.facts).terminal
    }
    /// Provider revocation can invalidate an already transferred scope without
    /// rewriting its original terminal cause or accepted peer fact.
    pub fn revoked(&self) -> bool {
        lock(&self.0.operation.hub.state).closed || lock(&self.0.facts).revoked
    }
    pub fn cleanup(&self) -> CleanupReceipt {
        CleanupReceipt(self.0.clone())
    }
}
#[derive(Clone)]
pub struct CleanupReceipt(pub(crate) Arc<Observed>);
impl CleanupReceipt {
    pub fn status(&self) -> CleanupStatus {
        lock(&self.0.facts)
            .cleanup
            .unwrap_or(CleanupStatus::Pending)
    }
    pub fn observation(&self) -> AuthObservation {
        AuthObservation(self.0.clone())
    }
}
impl std::fmt::Debug for CleanupReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.status().fmt(f)
    }
}
