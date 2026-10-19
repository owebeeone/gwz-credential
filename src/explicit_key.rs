//! First real source: one admitted endpoint-local file, never an agent fallback.
use crate::{
    key_worker::{FileAcquisition, FileRead, MAX_KEY, NativeRead},
    provider::*,
    *,
};
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use zeroize::Zeroizing;

pub struct ExplicitKeyProvider {
    pub(crate) reader: Arc<dyn FileRead>,
}
impl ExplicitKeyProvider {
    pub fn new() -> Self {
        Self {
            reader: Arc::new(NativeRead),
        }
    }
}
impl Default for ExplicitKeyProvider {
    fn default() -> Self {
        Self::new()
    }
}
impl Provider for ExplicitKeyProvider {
    fn acquisition_charge(
        &self,
        selection: &ExchangeSelection,
        purpose: &AcquisitionPurpose,
    ) -> Result<AcquisitionCharge, AuthError> {
        if !matches!(
            (&selection.source, purpose),
            (Source::ExplicitKey(_), AcquisitionPurpose::BeforeLookup)
        ) {
            return Err(AuthError::Unsupported);
        }
        if !matches!(&selection.target, Target::Ssh { user, .. } if user.len() <= 128) {
            return Err(AuthError::InvalidInput);
        }
        Ok(AcquisitionCharge {
            snapshot_bytes: MAX_KEY + 1 + 512,
            // Include the fixed native stack as retained work, not ambient memory.
            work_bytes: (2 << 20) + 32768,
        })
    }
    fn start_acquire(
        &self,
        input: &CredentialInput,
        selection: &ExchangeSelection,
        purpose: &AcquisitionPurpose,
        control: SourceControl,
    ) -> Result<Box<dyn ProviderAcquisition>, AuthError> {
        self.acquisition_charge(selection, purpose)?;
        let Source::ExplicitKey(path) = &selection.source else {
            return Err(AuthError::Unsupported);
        };
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            input
                .native_base
                .as_ref()
                .ok_or(AuthError::InvalidInput)?
                .join(path)
        };
        if !resolved.is_absolute() || resolved.as_os_str().len() > 4096 {
            return Err(AuthError::InvalidInput);
        }
        FileAcquisition::start(resolved, control, self.reader.clone())
    }
}
pub(crate) struct KeySnapshot {
    pub bytes: Zeroizing<Vec<u8>>,
}
impl SelectedCredential for KeySnapshot {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn same_material(&self, other: &dyn SelectedCredential) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.bytes.as_slice() == other.bytes.as_slice())
    }
    fn retained_bytes(&self) -> usize {
        self.bytes.capacity() + std::mem::size_of::<Self>() + 128
    }
    fn conversation_charge(&self) -> usize {
        128
    }
    fn scope(&self) -> SecurityScope {
        SecurityScope::Request
    }
    fn private_key(&self) -> Option<SecretKey<'_>> {
        Some(SecretKey::from_provider(&self.bytes))
    }
    fn start(
        &self,
        peer: &PeerContext,
        _: u64,
        cancellation: Cancellation,
    ) -> Result<Box<dyn ProviderSession>, AuthError> {
        if cancellation.is_cancelled() {
            return Err(AuthError::Cancelled);
        }
        if !matches!(peer.target, Target::Ssh { .. }) {
            return Err(AuthError::Unsupported);
        }
        Ok(Box::new(KeySession))
    }
}
struct KeySession;
impl ProviderSession for KeySession {
    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), AuthError>> {
        Poll::Ready(Ok(()))
    }
    fn poll_respond(
        &mut self,
        _: &mut Context<'_>,
        _: &AuthChallenge,
    ) -> Poll<Result<ProviderStep, AuthError>> {
        Poll::Ready(Err(AuthError::Unsupported))
    }
    fn poll_close(&mut self, _: &mut Context<'_>, _: bool) -> Poll<Result<(), AuthError>> {
        Poll::Ready(Ok(()))
    }
}
