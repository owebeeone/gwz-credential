//! Source terminal facts, independent of physical-peer authentication.
use crate::{provider::Reservation, state::*, *};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    ExplicitKey,
    Agent,
    HttpHelper,
    WindowsLogon,
}
impl SourceKind {
    pub(crate) fn of(source: &Source) -> Self {
        match source {
            Source::ExplicitKey(_) => Self::ExplicitKey,
            Source::Agent => Self::Agent,
            Source::HttpHelper => Self::HttpHelper,
            Source::WindowsLogon => Self::WindowsLogon,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcquisitionTerminal {
    pub reason: Option<AuthError>,
    pub material_published: bool,
}
#[derive(Default)]
pub(crate) struct SourceFacts {
    pub terminal: Option<AcquisitionTerminal>,
    pub cleanup: bool,
}
pub(crate) struct SourceObserved {
    pub operation: Arc<Operation>,
    pub id: u64,
    pub target: Target,
    pub kind: SourceKind,
    pub bytes: usize,
    pub _reservation: Box<dyn Reservation>,
    pub facts: Mutex<SourceFacts>,
}
impl Drop for SourceObserved {
    fn drop(&mut self) {
        let mut authority = lock(&self.operation.hub.state);
        lock(&self.operation.state).bytes -= self.bytes;
        authority.usage.retained_bytes -= self.bytes;
    }
}
#[derive(Clone)]
pub struct AcquisitionObservation(pub(crate) Arc<SourceObserved>);
impl AcquisitionObservation {
    pub fn id(&self) -> (u128, u64) {
        (self.0.operation.hub.config.namespace, self.0.id)
    }
    /// Read-only issuer-qualified attribution, without lifecycle authority.
    pub fn operation_id(&self) -> (u128, u64) {
        (self.0.operation.hub.config.namespace, self.0.operation.id)
    }
    pub fn endpoint(&self) -> &str {
        &self.0.operation.input.endpoint
    }
    pub fn target(&self) -> &Target {
        &self.0.target
    }
    pub fn source_kind(&self) -> SourceKind {
        self.0.kind
    }
    pub fn terminal(&self) -> Option<AcquisitionTerminal> {
        lock(&self.0.facts).terminal
    }
    pub fn revoked(&self) -> bool {
        let authority = lock(&self.0.operation.hub.state);
        authority.closed || lock(&self.0.operation.state).closed
    }
    pub fn cleanup(&self) -> AcquisitionCleanupReceipt {
        AcquisitionCleanupReceipt(self.0.clone())
    }
}
#[derive(Clone)]
pub struct AcquisitionCleanupReceipt(pub(crate) Arc<SourceObserved>);
impl AcquisitionCleanupReceipt {
    pub fn status(&self) -> CleanupStatus {
        if lock(&self.0.facts).cleanup {
            CleanupStatus::Confirmed
        } else {
            CleanupStatus::Pending
        }
    }
    pub fn observation(&self) -> AcquisitionObservation {
        AcquisitionObservation(self.0.clone())
    }
}
pub struct AcquisitionEnd {
    pub reason: Option<AuthError>,
    pub material_published: bool,
    pub cleanup: Option<AcquisitionCleanupReceipt>,
}
impl AcquisitionEnd {
    pub fn cleanup_status(&self) -> CleanupStatus {
        self.cleanup
            .as_ref()
            .map_or(CleanupStatus::Confirmed, |r| r.status())
    }
}
