//! Trusted endpoint-local provider and retention ports. No network/runtime types.
use crate::*;
use std::{
    sync::Arc,
    task::{Context, Poll},
};

pub trait Clock: Send + Sync {
    fn now_ms(&self) -> u64;
}
/// Wakes the independently owned retention service; never runs provider work inline.
pub trait Progress: Send + Sync {
    fn wake(&self);
}
pub trait RetainedWork: Send + Sync {
    /// Obtain before effects; no cancellation, scope-transfer or release rights.
    fn observation(&self) -> WorkObservation;
    /// The service must keep polling until Ready, independently of caller futures.
    /// It must schedule bounded periodic polls as well as native/wake notifications:
    /// cancellation flags and absolute deadlines do not depend on a provider wake.
    /// Calls must be nonblocking, including on shutdown; providers use retained
    /// native workers for blocking calls. Ready releases the registration once.
    fn advance(&self, cx: &mut Context<'_>) -> Poll<()>;
}
pub trait Reservation: Send + Sync {}
#[derive(Clone, Copy, Debug)]
pub struct RetentionCharge {
    pub bytes: usize,
    pub records: usize,
}
pub trait RetentionPort: Send + Sync {
    /// Refuse before effects. The reservation's Drop releases the actual allowance.
    fn reserve(&self, charge: RetentionCharge) -> Result<Box<dyn Reservation>, AuthError>;
    /// A reserved registration cannot be refused or silently discarded.
    fn retain(&self, work: Arc<dyn RetainedWork>, reservation: Box<dyn Reservation>);
}
pub struct CredentialPorts {
    pub clock: Arc<dyn Clock>,
    pub progress: Arc<dyn Progress>,
    pub retention: Arc<dyn RetentionPort>,
}
pub struct ProviderSet(pub Arc<dyn Provider>);
#[derive(Clone)]
pub enum WorkObservation {
    Acquisition(AcquisitionObservation),
    Authentication(AuthObservation),
}
#[derive(Clone, Copy)]
pub struct AcquisitionCharge {
    pub snapshot_bytes: usize,
    pub work_bytes: usize,
}
pub trait Provider: Send + Sync {
    /// Pure upper bound; reserve BEFORE any source access or worker allocation.
    fn acquisition_charge(
        &self,
        selection: &ExchangeSelection,
        purpose: &AcquisitionPurpose,
    ) -> Result<AcquisitionCharge, AuthError>;
    /// Called only from registered retained work, outside authority locks.
    /// Err is effect-free; registered native failures belong in the session.
    fn start_acquire(
        &self,
        input: &CredentialInput,
        selection: &ExchangeSelection,
        purpose: &AcquisitionPurpose,
        control: SourceControl,
    ) -> Result<Box<dyn ProviderAcquisition>, AuthError>;
}
pub trait ProviderAcquisition: Send {
    fn poll_acquire(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Arc<dyn SelectedCredential>, AuthError>>;
    /// Ready confirms actual source-job disposal, never just cancellation.
    fn poll_close(&mut self, cx: &mut Context<'_>, cancelled: bool) -> Poll<Result<(), AuthError>>;
}
pub trait SelectedCredential: std::any::Any + Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    /// Full admitted material equality, not a path or identifier hint.
    fn same_material(&self, other: &dyn SelectedCredential) -> bool;
    fn retained_bytes(&self) -> usize;
    fn conversation_charge(&self) -> usize;
    fn scope(&self) -> SecurityScope;
    /// Privileged native-provider borrow; never given to application/control code.
    fn private_key(&self) -> Option<SecretKey<'_>> {
        None
    }
    /// Err must be effect-free. Registered failure/cleanup belongs in the returned session.
    fn start(
        &self,
        peer: &PeerContext,
        deadline_ms: u64,
        cancellation: Cancellation,
    ) -> Result<Box<dyn ProviderSession>, AuthError>;
}
pub struct SecretKey<'a>(&'a [u8]);
impl<'a> SecretKey<'a> {
    pub fn from_provider(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
    pub fn native_bytes(&self) -> &'a [u8] {
        self.0
    }
}
pub trait KeyCleanup: Send + Sync {
    /// True only when the native/network owner no longer needs the key pin.
    fn confirmed(&self) -> bool;
}
pub trait LocalSshKeyPort {
    fn peer(&self) -> &PeerContext;
    fn key_cleanup(&self) -> Arc<dyn KeyCleanup>;
    fn authenticate(&mut self, key: SecretKey<'_>) -> Result<SshProgress, AuthError>;
}
pub enum ProviderStep {
    AwaitPeer,
    Complete,
    Response(SecretBytes),
}
pub trait ProviderSession: Send {
    fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), AuthError>>;
    fn poll_respond(
        &mut self,
        cx: &mut Context<'_>,
        challenge: &AuthChallenge,
    ) -> Poll<Result<ProviderStep, AuthError>>;
    /// Must remain pollable after cancellation; Ready confirms native disposal.
    fn poll_close(&mut self, cx: &mut Context<'_>, cancelled: bool) -> Poll<Result<(), AuthError>>;
}
