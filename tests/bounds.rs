mod support;
use gwz_credential::*;
use std::sync::{Arc, atomic::Ordering};
use support::*;

#[test]
fn saturated_records_refuse_before_provider_work_and_survive_host_replacement() {
    let mut h = Harness::new();
    h.retainer = Arc::new(TestRetainer::new(1));
    let (owner, handle) = h.build(CredentialConfig::new(1));
    let operation = handle.prepare(input()).unwrap();
    let first = h.open(&operation, 1);
    h.provider.cleanup_ready.store(false, Ordering::SeqCst);
    drop(first);
    drop(operation);
    drop(owner);
    h.drive();
    let (_replacement, other) = h.build(CredentialConfig::new(2));
    let b = other.prepare(input()).unwrap();
    assert!(matches!(
        h.acquire(&b, selection()),
        Err(AuthError::Capacity)
    ));
    assert_eq!(h.provider.starts.load(Ordering::SeqCst), 1);
    h.provider.cleanup_ready.store(true, Ordering::SeqCst);
    h.drive();
    let second = h.open(&b, 3);
    assert_eq!(h.provider.starts.load(Ordering::SeqCst), 2);
    drop(second);
    h.drive();
}

#[test]
fn operation_and_byte_limits_apply_before_source_admission() {
    let h = Harness::new();
    let mut config = CredentialConfig::new(1);
    config.max_operations = 1;
    let (_owner, handle) = h.build(config);
    let operation = handle.prepare(input()).unwrap();
    assert!(matches!(handle.prepare(input()), Err(AuthError::Capacity)));
    let mut oversized = input();
    struct Large;
    impl CapturedCaller for Large {
        fn retained_bytes(&self) -> usize {
            usize::MAX
        }
        fn identity(&self) -> u128 {
            1
        }
        fn same_identity(&self, _: &dyn CapturedCaller) -> bool {
            false
        }
    }
    oversized.caller = Arc::new(Large);
    assert!(matches!(
        handle.prepare(oversized),
        Err(AuthError::InputBudget)
    ));
    drop(operation);
    assert_eq!(handle.snapshot(), Snapshot::default());
}

#[test]
fn ten_thousand_operations_retire_records_instead_of_accumulating_tombstones() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(1));
    for generation in 1..=10_000 {
        let operation = handle.prepare(input()).unwrap();
        let session = h.open(&operation, generation);
        let end = h.run(session.finish(PeerAuthResult::Accepted));
        assert_eq!(end.cleanup.status(), CleanupStatus::Confirmed);
        drop(end);
        drop(operation);
        h.drive();
        assert_eq!(handle.snapshot(), Snapshot::default());
        assert_eq!(h.retainer.reservations(), 0);
    }
}
