use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use zeroize::Zeroizing;

/// Sanitized failure classes; provider diagnostics and secrets are never copied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthError {
    InvalidInput,
    InputBudget,
    Capacity,
    ProviderClosed,
    OperationClosed,
    Cancelled,
    Timeout,
    PeerMismatch,
    Unsupported,
    ProviderRejected,
    ProviderFault,
    WrongState,
    WouldBlock,
}
impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "credential: {self:?}")
    }
}
impl std::error::Error for AuthError {}
pub type AuthFailure = AuthError;
pub type BuildError = AuthError;
pub type PrepareError = AuthError;
pub type SelectError = AuthError;

/// Local bytes for trusted provider ports, never serializable or publicly readable.
pub struct SecretBytes(pub(crate) Zeroizing<Vec<u8>>);
impl SecretBytes {
    pub fn new(bytes: Vec<u8>) -> Result<Self, AuthError> {
        let owner = Zeroizing::new(bytes);
        if owner.capacity() > 65536 {
            return Err(AuthError::InputBudget);
        }
        Ok(Self(owner))
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl std::fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretBytes([redacted])")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Ssh { host: String, user: String },
    Https { origin: String },
}
impl Target {
    pub(crate) fn validate(&self) -> Result<(), AuthError> {
        let valid = |s: &str| !s.is_empty() && s.len() <= 4096 && !s.chars().any(char::is_control);
        match self {
            Self::Ssh { host, user } if valid(host) && valid(user) => Ok(()),
            Self::Https { origin }
                if valid(origin)
                    && origin.starts_with("https://")
                    && !origin.contains(['@', '?', '#']) =>
            {
                Ok(())
            }
            _ => Err(AuthError::InvalidInput),
        }
    }
    pub(crate) fn bytes(&self) -> usize {
        match self {
            Self::Ssh { host, user } => host.capacity() + user.capacity(),
            Self::Https { origin } => origin.capacity(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    ExplicitKey(String),
    Agent,
    HttpHelper,
    WindowsLogon,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExchangeSelection {
    pub target: Target,
    pub source: Source,
}
impl ExchangeSelection {
    pub(crate) fn validate(&self) -> Result<(), AuthError> {
        self.target.validate()?;
        match (&self.target, &self.source) {
            (Target::Ssh { .. }, Source::Agent) => Ok(()),
            (Target::Ssh { .. }, Source::ExplicitKey(path))
                if !path.is_empty()
                    && path.len() <= 4096
                    && !path.chars().any(char::is_control) =>
            {
                Ok(())
            }
            (Target::Https { .. }, Source::HttpHelper | Source::WindowsLogon) => Ok(()),
            _ => Err(AuthError::InvalidInput),
        }
    }
}

/// Supplied original-entry provenance. Providers interpret its opaque local data.
pub trait CapturedCaller: std::any::Any + Send + Sync {
    fn retained_bytes(&self) -> usize;
    /// Index/hint only, not sufficient evidence of identity equality.
    fn identity(&self) -> u128;
    /// Trusted provider comparison of complete captured identity, not a hash match.
    fn same_identity(&self, other: &dyn CapturedCaller) -> bool;
}
pub struct CredentialInput {
    pub endpoint: String,
    pub caller: Arc<dyn CapturedCaller>,
    pub allowed: Vec<ExchangeSelection>,
}
impl CredentialInput {
    pub(crate) fn charge(&self) -> Result<usize, AuthError> {
        if self.endpoint.is_empty()
            || self.endpoint.len() > 4096
            || self.endpoint.chars().any(char::is_control)
            || self.allowed.is_empty()
            || self.allowed.capacity() > 64
        {
            return Err(AuthError::InvalidInput);
        }
        let mut bytes = self
            .endpoint
            .capacity()
            .checked_add(self.caller.retained_bytes())
            .and_then(|n| {
                n.checked_add(self.allowed.capacity() * std::mem::size_of::<ExchangeSelection>())
            })
            .ok_or(AuthError::InputBudget)?;
        for selection in &self.allowed {
            selection.validate()?;
            bytes = bytes
                .checked_add(selection.target.bytes() + 128)
                .ok_or(AuthError::InputBudget)?;
            if let Source::ExplicitKey(path) = &selection.source {
                bytes = bytes
                    .checked_add(path.capacity())
                    .ok_or(AuthError::InputBudget)?;
            }
        }
        Ok(bytes)
    }
}

/// Trusted network adapter's verified facts; deliberately no Debug/serialization.
#[derive(Clone, PartialEq, Eq)]
pub struct PeerContext {
    pub endpoint: String,
    pub target: Target,
    pub generation: u64,
    pub verified_peer: Vec<u8>,
    pub channel_binding: Vec<u8>,
}
impl PeerContext {
    pub(crate) fn validate(&self) -> Result<(), AuthError> {
        self.target.validate()?;
        if self.endpoint.is_empty()
            || self.endpoint.len() > 4096
            || self.generation == 0
            || self.verified_peer.is_empty()
            || self.verified_peer.capacity() > 65536
            || self.channel_binding.capacity() > 65536
        {
            return Err(AuthError::InvalidInput);
        }
        Ok(())
    }
}

pub struct AuthChallenge(pub(crate) SecretBytes);
impl AuthChallenge {
    pub fn new(bytes: Vec<u8>) -> Result<Self, AuthError> {
        Ok(Self(SecretBytes::new(bytes)?))
    }
    /// Privileged provider view, not a response/key export API.
    pub fn provider_bytes(&self) -> &[u8] {
        &self.0.0
    }
}

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
pub struct AuthControl {
    pub deadline_ms: u64,
    pub cancellation: Cancellation,
}
impl AuthControl {
    pub fn new(deadline_ms: u64) -> Self {
        Self {
            deadline_ms,
            cancellation: Cancellation::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerAuthResult {
    Accepted,
    Rejected,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleanupStatus {
    Confirmed,
    Pending,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicalDisposition {
    RequestSettled,
    BoundScope,
    Retire,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SshProgress {
    NeedNetwork,
    Accepted,
    Rejected,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityScope {
    Request,
    Connection,
}

/// Explicit finite limits. The namespace is supplied by the composition root.
#[derive(Clone)]
pub struct CredentialConfig {
    pub namespace: u128,
    pub max_operations: usize,
    pub max_conversations: usize,
    pub max_operation_bytes: usize,
    pub max_bytes: usize,
    pub max_rounds: usize,
    pub cleanup_report_ms: u64,
}
impl CredentialConfig {
    pub fn new(namespace: u128) -> Self {
        Self {
            namespace,
            max_operations: 72,
            max_conversations: 64,
            max_operation_bytes: 16 * 1024 * 1024,
            max_bytes: 64 * 1024 * 1024,
            max_rounds: 8,
            cleanup_report_ms: 5000,
        }
    }
    pub(crate) fn validate(&self) -> Result<(), AuthError> {
        if self.max_operations == 0
            || self.max_conversations == 0
            || self.max_rounds == 0
            || self.max_operation_bytes < 65536
            || self.max_bytes < self.max_operation_bytes
            || self.cleanup_report_ms == 0
            || self.cleanup_report_ms > 5000
        {
            return Err(AuthError::InvalidInput);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub operations: usize,
    pub conversations: usize,
    pub retained_bytes: usize,
}
