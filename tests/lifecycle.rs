mod support;
use gwz_credential::*;
use std::{pin::pin, sync::atomic::Ordering};
use support::*;

#[test]
fn selected_before_owner_close_cannot_start_work() {
    let h = Harness::new();
    let (owner, handle) = h.build(CredentialConfig::new(1));
    let operation = handle.prepare(input()).unwrap();
    let prepared = h.acquire(&operation, selection()).unwrap();
    drop(owner.close());
    assert!(matches!(
        h.run(prepared.begin(peer(1), AuthControl::new(1000)).wait()),
        Err(AuthError::ProviderClosed)
    ));
    assert_eq!(h.provider.starts.load(Ordering::SeqCst), 0);
    assert_eq!(h.retainer.pending(), 0);
}

#[test]
fn lost_opening_never_publishes_a_late_session() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(2));
    let operation = handle.prepare(input()).unwrap();
    let opening = h
        .acquire(&operation, selection())
        .unwrap()
        .begin(peer(1), AuthControl::new(1000));
    h.drive(); // Ready provider, but application has not consumed the opening.
    drop(opening);
    h.drive();
    assert_eq!(h.retainer.pending(), 0);
    assert_eq!(handle.snapshot().conversations, 0);
}

#[test]
fn dropped_response_retains_native_work_until_actual_disposal() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(3));
    let operation = handle.prepare(input()).unwrap();
    let mut session = h.open(&operation, 1);
    h.provider.response_ready.store(false, Ordering::SeqCst);
    h.provider.cleanup_ready.store(false, Ordering::SeqCst);
    {
        let mut response = pin!(session.respond(AuthChallenge::new(vec![]).unwrap()));
        h.drive();
        assert!(poll(response.as_mut()).is_pending());
    }
    h.drive();
    assert_eq!(handle.snapshot().conversations, 1);
    h.provider.response_ready.store(true, Ordering::SeqCst);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 1); // Late reply is not cleanup.
    h.provider.cleanup_ready.store(true, Ordering::SeqCst);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 0);
    assert!(matches!(h.respond(&mut session), Err(AuthError::Cancelled)));
}

#[test]
fn peer_acceptance_and_pending_native_cleanup_are_separate() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(4));
    let operation = handle.prepare(input()).unwrap();
    let session = h.open(&operation, 1);
    h.provider.cleanup_ready.store(false, Ordering::SeqCst);
    let end = h.run(session.finish(PeerAuthResult::Accepted));
    assert_eq!(end.peer_result, PeerAuthResult::Accepted);
    assert_eq!(end.cleanup.status(), CleanupStatus::Pending);
    assert_eq!(end.disposition, PhysicalDisposition::RequestSettled);
    assert_eq!(handle.snapshot().conversations, 1);
    h.provider.cleanup_ready.store(true, Ordering::SeqCst);
    h.drive();
    assert_eq!(end.cleanup.status(), CleanupStatus::Confirmed);
    assert_eq!(handle.snapshot().conversations, 0);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 0); // No double release.
}

#[test]
fn deadline_is_absolute_across_multiple_rounds() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(5));
    let operation = handle.prepare(input()).unwrap();
    let mut session = h.open(&operation, 1);
    h.clock.0.store(999, Ordering::SeqCst);
    drop(h.respond(&mut session).unwrap());
    h.clock.0.store(1000, Ordering::SeqCst);
    assert!(matches!(h.respond(&mut session), Err(AuthError::Timeout)));
    let end = h.run(session.finish(PeerAuthResult::Accepted));
    assert_eq!(end.reason, Some(AuthError::Timeout));
    assert_eq!(end.disposition, PhysicalDisposition::Retire);
}

#[test]
fn cancellation_of_one_operation_does_not_touch_another() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(6));
    let a = handle.prepare(input()).unwrap();
    let b = handle.prepare(input()).unwrap();
    let mut first = h.open(&a, 1);
    let mut second = h.open(&b, 2);
    drop(a.close());
    assert!(matches!(
        h.respond(&mut first),
        Err(AuthError::OperationClosed)
    ));
    assert!(h.respond(&mut second).is_ok());
}

#[test]
fn provider_panic_does_not_escape_or_fabricate_peer_success() {
    let h = Harness::new();
    h.provider.panic_response.store(true, Ordering::SeqCst);
    let (_owner, handle) = h.build(CredentialConfig::new(7));
    let operation = handle.prepare(input()).unwrap();
    let mut session = h.open(&operation, 1);
    assert!(matches!(
        h.respond(&mut session),
        Err(AuthError::ProviderFault)
    ));
    let end = h.run(session.cancel());
    assert_eq!(end.peer_result, PeerAuthResult::Unknown);
    assert_eq!(end.disposition, PhysicalDisposition::Retire);
    assert_eq!(end.cleanup.status(), CleanupStatus::Confirmed);
}

#[test]
fn later_owner_shutdown_does_not_mask_a_recorded_provider_failure() {
    let h = Harness::new();
    h.provider.panic_response.store(true, Ordering::SeqCst);
    let (owner, handle) = h.build(CredentialConfig::new(8));
    let operation = handle.prepare(input()).unwrap();
    let mut session = h.open(&operation, 3);
    let observation = session.observation();
    assert!(matches!(
        h.respond(&mut session),
        Err(AuthError::ProviderFault)
    ));
    let fixed = observation.terminal().unwrap();
    drop(operation);
    drop(owner);
    assert!(matches!(
        h.respond(&mut session),
        Err(AuthError::ProviderFault)
    ));
    let end = h.run(session.finish(PeerAuthResult::Accepted));
    assert_eq!(end.reason, Some(AuthError::ProviderFault));
    assert_eq!(end.peer_result, PeerAuthResult::Unknown);
    assert_eq!(end.disposition, PhysicalDisposition::Retire);
    assert_eq!(observation.terminal(), Some(fixed));
    assert!(observation.revoked());
}
