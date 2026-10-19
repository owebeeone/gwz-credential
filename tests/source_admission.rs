//! Permanent regression for the formerly unowned synchronous-source stall.
//! This port deliberately remains Pending; actual file-worker barriers are separate.
mod support;
use gwz_credential::*;
use std::sync::{Arc, atomic::Ordering};
use support::*;

#[test]
fn stalled_source_close_is_observed_retained_and_blocks_replacement_without_late_material() {
    let mut h = Harness::new();
    h.retainer = Arc::new(TestRetainer::new(1));
    h.provider.acquisition_ready.store(false, Ordering::SeqCst);
    h.provider
        .acquisition_cleanup_ready
        .store(false, Ordering::SeqCst);
    let (owner, handle) = h.build(CredentialConfig::new(1));
    let operation = handle.prepare(input()).unwrap();
    let acquiring = operation.context().acquire(
        selection(),
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    let observation = acquiring.observation().unwrap();
    h.drive();
    assert_eq!(h.provider.acquisitions.load(Ordering::SeqCst), 1);
    drop(owner.close());
    drop(operation.close());
    drop(acquiring);
    h.drive();
    let terminal = observation.terminal().unwrap();
    assert_eq!(terminal.reason, Some(AuthError::ProviderClosed));
    assert!(!terminal.material_published);
    assert_eq!(observation.cleanup().status(), CleanupStatus::Pending);
    assert_eq!(handle.snapshot().acquisitions, 1);
    let (_replacement, other) = h.build(CredentialConfig::new(2));
    let next = other.prepare(input()).unwrap();
    let refused = next.context().acquire(
        selection(),
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    assert!(refused.observation().is_none());
    assert!(matches!(h.run(refused), Err(AuthError::Capacity)));
    assert_eq!(h.provider.acquisitions.load(Ordering::SeqCst), 1);
    h.provider.acquisition_ready.store(true, Ordering::SeqCst);
    h.drive();
    assert_eq!(observation.cleanup().status(), CleanupStatus::Pending);
    h.provider
        .acquisition_cleanup_ready
        .store(true, Ordering::SeqCst);
    h.drive();
    assert_eq!(observation.terminal(), Some(terminal));
    assert_eq!(observation.cleanup().status(), CleanupStatus::Confirmed);
    assert_eq!(handle.snapshot().acquisitions, 0);
    drop(observation);
    assert_eq!(handle.snapshot(), Snapshot::default());
}
