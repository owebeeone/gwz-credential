mod support;
use gwz_credential::*;
use std::sync::atomic::Ordering;
use support::*;

#[test]
fn fake_bridge_observes_lost_and_failed_openings_without_caller_progress() {
    for fail in [false, true] {
        let h = Harness::new();
        h.provider.fail_start.store(fail, Ordering::SeqCst);
        h.provider.cleanup_ready.store(false, Ordering::SeqCst);
        let (owner, handle) = h.build(CredentialConfig::new(45));
        let operation = handle.prepare(input()).unwrap();
        let opening = operation
            .context()
            .select(selection())
            .unwrap()
            .begin(peer(40), AuthControl::new(1000));
        // The composition retainer obtains this through the public work port,
        // before provider/native effects; it needs no application-stage reference.
        let observation = h.retainer.observations().pop().unwrap();
        assert_eq!(observation.id().0, 45);
        assert_eq!(observation.peer().generation, 40);
        assert!(observation.terminal().is_none());
        h.drive();
        if fail {
            assert!(matches!(h.run(opening), Err(AuthError::ProviderRejected)));
        } else {
            drop(opening);
        }
        let fixed = observation.terminal().unwrap();
        assert_eq!(
            fixed.reason,
            Some(if fail {
                AuthError::ProviderRejected
            } else {
                AuthError::Cancelled
            })
        );
        assert_eq!(fixed.disposition, PhysicalDisposition::Retire);
        drop(operation);
        drop(owner);
        drop(handle);
        h.drive();
        assert_eq!(observation.terminal(), Some(fixed));
        assert_eq!(observation.cleanup().status(), CleanupStatus::Pending);
        assert_eq!(h.retainer.pending(), 1);
        h.provider.cleanup_ready.store(true, Ordering::SeqCst);
        h.drive();
        assert_eq!(observation.cleanup().status(), CleanupStatus::Confirmed);
        assert_eq!(h.retainer.pending(), 0);
        assert!(h.retainer.reservations() > 0); // Observation is charged, not a tombstone.
        drop(observation);
        h.drive();
        assert_eq!(h.retainer.reservations(), 0);
    }
}
fn native_input() -> CredentialInput {
    let mut input = input();
    input.allowed[0].source = Source::WindowsLogon;
    input
}
#[test]
fn lost_finish_or_cancel_reports_preserve_generation_and_accepted_fact() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(2));
    let operation = handle.prepare(native_input()).unwrap();
    let first = h
        .run(
            operation
                .context()
                .select(native_input().allowed.remove(0))
                .unwrap()
                .begin(peer(1), AuthControl::new(1000)),
        )
        .unwrap();
    let second = h
        .run(
            operation
                .context()
                .select(native_input().allowed.remove(0))
                .unwrap()
                .begin(peer(2), AuthControl::new(1000)),
        )
        .unwrap();
    let a = first.observation();
    let b = second.observation();
    h.provider.cleanup_ready.store(false, Ordering::SeqCst);
    drop(first.finish(PeerAuthResult::Accepted));
    drop(second.cancel());
    assert_ne!(a.id(), b.id());
    assert_eq!(a.peer().generation, 1);
    assert_eq!(b.peer().generation, 2);
    assert_eq!(a.terminal().unwrap().peer_result, PeerAuthResult::Accepted);
    assert_eq!(
        a.terminal().unwrap().disposition,
        PhysicalDisposition::Retire
    );
    assert_eq!(b.terminal().unwrap().reason, Some(AuthError::Cancelled));
    h.drive();
    assert_eq!(handle.snapshot().conversations, 2);
    h.provider.cleanup_ready.store(true, Ordering::SeqCst);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 0);
    assert_eq!(a.cleanup().status(), CleanupStatus::Confirmed);
    assert_eq!(b.cleanup().status(), CleanupStatus::Confirmed);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 0);
}
#[test]
fn pending_scope_transfer_is_fenced_by_cancel_and_deadline_but_transferred_scope_survives_operation()
 {
    for deadline in [false, true] {
        let h = Harness::new();
        let (_owner, handle) = h.build(CredentialConfig::new(3));
        let operation = handle.prepare(native_input()).unwrap();
        let control = AuthControl::new(1000);
        let cancel = control.cancellation.clone();
        let session = h
            .run(
                operation
                    .context()
                    .select(native_input().allowed.remove(0))
                    .unwrap()
                    .begin(peer(1), control),
            )
            .unwrap();
        let observation = session.observation();
        let finishing = session.finish(PeerAuthResult::Accepted);
        if deadline {
            h.clock.0.store(1000, Ordering::SeqCst);
        } else {
            cancel.cancel();
        }
        let end = h.run(finishing);
        assert_eq!(end.peer_result, PeerAuthResult::Accepted);
        assert_eq!(
            end.reason,
            Some(if deadline {
                AuthError::Timeout
            } else {
                AuthError::Cancelled
            })
        );
        assert!(end.scope.is_none());
        assert_eq!(end.disposition, PhysicalDisposition::Retire);
        assert_eq!(observation.terminal().unwrap().reason, end.reason);
    }
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(4));
    let operation = handle.prepare(native_input()).unwrap();
    let control = AuthControl::new(1000);
    let cancel = control.cancellation.clone();
    let session = h
        .run(
            operation
                .context()
                .select(native_input().allowed.remove(0))
                .unwrap()
                .begin(peer(1), control),
        )
        .unwrap();
    let end = h.run(session.finish(PeerAuthResult::Accepted));
    let scope = end.scope.unwrap();
    cancel.cancel();
    drop(operation);
    let next = handle.prepare(native_input()).unwrap();
    let mut queue = EndpointSendQueue::new(peer(1), 1).unwrap();
    scope
        .grant(&next.context(), peer(1), AuthControl::new(1000))
        .unwrap()
        .commit(&mut queue)
        .unwrap();
    assert_eq!(queue.len(), 1);
}
