#![allow(dead_code)]
use gwz_credential::{provider::*, *};
use std::{
    future::Future,
    pin::pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

pub struct TestClock(pub AtomicU64);
impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}
pub struct TestProgress;
impl Progress for TestProgress {
    fn wake(&self) {}
}
type Entry = (Arc<dyn RetainedWork>, Box<dyn Reservation>);
pub struct TestRetainer {
    entries: Mutex<Vec<Entry>>,
    count: Arc<AtomicUsize>,
    budget: Arc<Mutex<(usize, usize)>>,
    pub limit: usize,
}
struct Permit {
    count: Arc<AtomicUsize>,
    budget: Arc<Mutex<(usize, usize)>>,
    charge: RetentionCharge,
}
impl Reservation for Permit {}
impl Drop for Permit {
    fn drop(&mut self) {
        let mut budget = self.budget.lock().unwrap();
        budget.0 -= self.charge.records;
        budget.1 -= self.charge.bytes;
        self.count.fetch_sub(1, Ordering::SeqCst);
    }
}
impl TestRetainer {
    pub fn new(limit: usize) -> Self {
        Self {
            entries: Mutex::default(),
            count: Arc::default(),
            budget: Arc::default(),
            limit,
        }
    }
    pub fn observations(&self) -> Vec<AuthObservation> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .map(|(work, _)| work.observation())
            .collect()
    }
    pub fn pending(&self) -> usize {
        self.entries.lock().unwrap().len()
    }
    pub fn reservations(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
    pub fn tick(&self) {
        let entries = std::mem::take(&mut *self.entries.lock().unwrap());
        let mut pending = Vec::new();
        let waker = Waker::from(Arc::new(Noop));
        let mut cx = Context::from_waker(&waker);
        for (work, permit) in entries {
            if work.advance(&mut cx).is_pending() {
                pending.push((work, permit));
            }
        }
        self.entries.lock().unwrap().extend(pending);
    }
}
impl RetentionPort for TestRetainer {
    fn reserve(&self, charge: RetentionCharge) -> Result<Box<dyn Reservation>, AuthError> {
        let mut budget = self.budget.lock().unwrap();
        if budget.0 + charge.records > self.limit || budget.1 + charge.bytes > 64 * 1024 * 1024 {
            return Err(AuthError::Capacity);
        }
        budget.0 += charge.records;
        budget.1 += charge.bytes;
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(Permit {
            count: self.count.clone(),
            budget: self.budget.clone(),
            charge,
        }))
    }
    fn retain(&self, work: Arc<dyn RetainedWork>, reservation: Box<dyn Reservation>) {
        self.entries.lock().unwrap().push((work, reservation));
    }
}
pub struct Noop;
impl Wake for Noop {
    fn wake(self: Arc<Self>) {}
}
pub fn poll<F: Future>(future: std::pin::Pin<&mut F>) -> Poll<F::Output> {
    let waker = Waker::from(Arc::new(Noop));
    future.poll(&mut Context::from_waker(&waker))
}
pub struct Caller(pub u128);
impl CapturedCaller for Caller {
    fn retained_bytes(&self) -> usize {
        16
    }
    fn identity(&self) -> u128 {
        self.0
    }
    fn same_identity(&self, other: &dyn CapturedCaller) -> bool {
        (other as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .is_some_and(|other| self.0 == other.0)
    }
}
pub fn selection() -> ExchangeSelection {
    ExchangeSelection {
        target: Target::Https {
            origin: "https://git.example".into(),
        },
        source: Source::HttpHelper,
    }
}
pub fn input() -> CredentialInput {
    CredentialInput {
        endpoint: "endpoint-1".into(),
        caller: Arc::new(Caller(7)),
        allowed: vec![selection()],
    }
}
pub fn peer(generation: u64) -> PeerContext {
    PeerContext {
        endpoint: "endpoint-1".into(),
        target: selection().target,
        generation,
        verified_peer: b"verified-server".to_vec(),
        channel_binding: b"tls-binding".to_vec(),
    }
}
pub struct FakeProvider {
    pub cleanup_ready: Arc<AtomicBool>,
    pub response_ready: Arc<AtomicBool>,
    pub start_ready: Arc<AtomicBool>,
    pub starts: Arc<AtomicUsize>,
    pub key: Mutex<Vec<u8>>,
    pub panic_response: AtomicBool,
    pub fail_start: AtomicBool,
}
impl FakeProvider {
    pub fn new() -> Self {
        Self {
            cleanup_ready: Arc::new(AtomicBool::new(true)),
            response_ready: Arc::new(AtomicBool::new(true)),
            start_ready: Arc::new(AtomicBool::new(true)),
            starts: Arc::default(),
            key: Mutex::new(b"key-a".to_vec()),
            panic_response: AtomicBool::new(false),
            fail_start: AtomicBool::new(false),
        }
    }
}
struct Selected {
    cleanup: Arc<AtomicBool>,
    respond: Arc<AtomicBool>,
    start: Arc<AtomicBool>,
    starts: Arc<AtomicUsize>,
    key: zeroize::Zeroizing<Vec<u8>>,
    scope: SecurityScope,
    panic_response: bool,
    fail_start: bool,
}
impl Provider for FakeProvider {
    fn selection_charge(&self, _: &ExchangeSelection) -> Result<usize, AuthError> {
        Ok(64)
    }
    fn select(
        &self,
        _: &CredentialInput,
        selection: &ExchangeSelection,
    ) -> Result<Arc<dyn SelectedCredential>, AuthError> {
        Ok(Arc::new(Selected {
            cleanup: self.cleanup_ready.clone(),
            respond: self.response_ready.clone(),
            start: self.start_ready.clone(),
            starts: self.starts.clone(),
            key: zeroize::Zeroizing::new(self.key.lock().unwrap().clone()),
            scope: if selection.source == Source::WindowsLogon {
                SecurityScope::Connection
            } else {
                SecurityScope::Request
            },
            panic_response: self.panic_response.load(Ordering::SeqCst),
            fail_start: self.fail_start.load(Ordering::SeqCst),
        }))
    }
}
impl SelectedCredential for Selected {
    fn retained_bytes(&self) -> usize {
        self.key.len()
    }
    fn conversation_charge(&self) -> usize {
        256
    }
    fn scope(&self) -> SecurityScope {
        self.scope
    }
    fn private_key(&self) -> Option<SecretKey<'_>> {
        Some(SecretKey::from_provider(&self.key))
    }
    fn start(
        &self,
        _: &PeerContext,
        _: u64,
        _: Cancellation,
    ) -> Result<Box<dyn ProviderSession>, AuthError> {
        self.starts.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(FakeSession {
            cleanup: self.cleanup.clone(),
            respond: self.respond.clone(),
            start: self.start.clone(),
            panic_response: self.panic_response,
            fail_start: self.fail_start,
        }))
    }
}
struct FakeSession {
    cleanup: Arc<AtomicBool>,
    respond: Arc<AtomicBool>,
    start: Arc<AtomicBool>,
    panic_response: bool,
    fail_start: bool,
}
impl ProviderSession for FakeSession {
    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), AuthError>> {
        if self.fail_start {
            Poll::Ready(Err(AuthError::ProviderRejected))
        } else if self.start.load(Ordering::SeqCst) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
    fn poll_respond(
        &mut self,
        _: &mut Context<'_>,
        _: &AuthChallenge,
    ) -> Poll<Result<ProviderStep, AuthError>> {
        assert!(!self.panic_response, "synthetic provider failure");
        if self.respond.load(Ordering::SeqCst) {
            Poll::Ready(Ok(ProviderStep::Response(
                SecretBytes::new(b"token".to_vec()).unwrap(),
            )))
        } else {
            Poll::Pending
        }
    }
    fn poll_close(&mut self, _: &mut Context<'_>, _: bool) -> Poll<Result<(), AuthError>> {
        if self.cleanup.load(Ordering::SeqCst) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}
pub struct Harness {
    pub clock: Arc<TestClock>,
    pub provider: Arc<FakeProvider>,
    pub retainer: Arc<TestRetainer>,
}
impl Harness {
    pub fn new() -> Self {
        Self {
            clock: Arc::new(TestClock(AtomicU64::new(0))),
            provider: Arc::new(FakeProvider::new()),
            retainer: Arc::new(TestRetainer::new(64)),
        }
    }
    pub fn build(&self, config: CredentialConfig) -> (CredentialOwner, CredentialHandle) {
        build(
            config,
            ProviderSet(self.provider.clone()),
            CredentialPorts {
                clock: self.clock.clone(),
                progress: Arc::new(TestProgress),
                retention: self.retainer.clone(),
            },
        )
        .unwrap()
    }
    pub fn drive(&self) {
        for _ in 0..8 {
            self.retainer.tick();
        }
    }
    pub fn run<F: Future>(&self, future: F) -> F::Output {
        let mut future = pin!(future);
        for _ in 0..100 {
            self.drive();
            if let Poll::Ready(value) = poll(future.as_mut()) {
                return value;
            }
        }
        panic!("fake work did not settle");
    }
    pub fn open(&self, operation: &OperationCredentials, generation: u64) -> AuthSession {
        let opening = operation
            .context()
            .select(selection())
            .unwrap()
            .begin(peer(generation), AuthControl::new(1000));
        match self.run(opening.wait()) {
            Ok(session) => session,
            Err(error) => panic!("open: {error}"),
        }
    }
    pub fn respond(&self, session: &mut AuthSession) -> Result<AuthStep, AuthError> {
        self.run(session.respond(AuthChallenge::new(b"challenge".to_vec()).unwrap()))
    }
}
