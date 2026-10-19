mod support;
use gwz_credential::{provider::*, *};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicBool, Ordering},
};
use support::*;

struct Probe(AtomicBool);
impl KeyCleanup for Probe {
    fn confirmed(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
struct KeyPort {
    peer: PeerContext,
    probe: Arc<Probe>,
    seen: Vec<Vec<u8>>,
    barrier: Option<Arc<Barrier>>,
}
impl LocalSshKeyPort for KeyPort {
    fn peer(&self) -> &PeerContext {
        &self.peer
    }
    fn key_cleanup(&self) -> Arc<dyn KeyCleanup> {
        self.probe.clone()
    }
    fn authenticate(&mut self, key: SecretKey<'_>) -> Result<SshProgress, AuthError> {
        self.seen.push(key.native_bytes().to_vec());
        if let Some(barrier) = &self.barrier {
            barrier.wait();
            barrier.wait();
        }
        Ok(SshProgress::NeedNetwork)
    }
}
fn explicit() -> ExchangeSelection {
    ExchangeSelection {
        target: Target::Ssh {
            host: "git.example".into(),
            user: "git".into(),
        },
        source: Source::ExplicitKey("endpoint-key".into()),
    }
}
fn setup(
    h: &Harness,
    handle: &CredentialHandle,
) -> (OperationCredentials, AuthSession, PeerContext) {
    let mut captured = input();
    captured.allowed = vec![explicit()];
    let operation = handle.prepare(captured).unwrap();
    let mut actual_peer = peer(10);
    actual_peer.target = explicit().target;
    let session = match h.run(
        operation
            .context()
            .select(explicit())
            .unwrap()
            .begin(actual_peer.clone(), AuthControl::new(1000))
            .wait(),
    ) {
        Ok(session) => session,
        Err(error) => panic!("{error}"),
    };
    (operation, session, actual_peer)
}

#[test]
fn key_snapshot_survives_reentry_and_cancel_waits_for_the_native_disposal_probe() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(1));
    let (_operation, mut session, actual_peer) = setup(&h, &handle);
    let probe = Arc::new(Probe(AtomicBool::new(false)));
    let mut port = KeyPort {
        peer: actual_peer,
        probe: probe.clone(),
        seen: Vec::new(),
        barrier: None,
    };
    session.use_ssh_key(&mut port).unwrap();
    *h.provider.key.lock().unwrap() = b"replacement".to_vec();
    session.use_ssh_key(&mut port).unwrap();
    assert_eq!(port.seen, [b"key-a".to_vec(), b"key-a".to_vec()]);
    let end = h.run(session.cancel());
    assert_eq!(end.cleanup.status(), CleanupStatus::Pending);
    assert_eq!(handle.snapshot().conversations, 1);
    probe.0.store(true, Ordering::SeqCst); // Network owner confirms safe disposal.
    h.drive();
    assert_eq!(end.cleanup.status(), CleanupStatus::Confirmed);
    assert_eq!(handle.snapshot().conversations, 0);
    let (_next, mut next, actual_peer) = setup(&h, &handle);
    let mut port = KeyPort {
        peer: actual_peer,
        probe: Arc::new(Probe(AtomicBool::new(true))),
        seen: Vec::new(),
        barrier: None,
    };
    next.use_ssh_key(&mut port).unwrap();
    assert_eq!(port.seen, [b"replacement".to_vec()]);
}

#[test]
fn provider_close_during_native_key_call_retains_the_pin_and_revokes_late_success() {
    let h = Harness::new();
    let (owner, handle) = h.build(CredentialConfig::new(2));
    let (_operation, mut session, actual_peer) = setup(&h, &handle);
    let barrier = Arc::new(Barrier::new(2));
    let probe = Arc::new(Probe(AtomicBool::new(false)));
    let mut port = KeyPort {
        peer: actual_peer,
        probe: probe.clone(),
        seen: Vec::new(),
        barrier: Some(barrier.clone()),
    };
    let thread = std::thread::spawn(move || {
        let result = session.use_ssh_key(&mut port);
        (result, session)
    });
    barrier.wait();
    drop(owner.close());
    h.drive();
    assert_eq!(handle.snapshot().conversations, 1);
    barrier.wait();
    let (result, session) = thread.join().unwrap();
    assert_eq!(result, Err(AuthError::ProviderClosed));
    drop(session);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 1);
    probe.0.store(true, Ordering::SeqCst);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 0);
}
