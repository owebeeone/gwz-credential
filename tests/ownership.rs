mod support;
use gwz_credential::*;
use support::*;

#[test]
fn one_operation_issues_independent_conversations() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(1));
    let operation = handle.prepare(input()).unwrap();
    let mut a = h.open(&operation, 10);
    let mut b = h.open(&operation, 11);
    drop(a.respond(AuthChallenge::new(b"challenge".to_vec()).unwrap()));
    assert!(matches!(h.respond(&mut a), Err(AuthError::Cancelled)));
    assert!(h.respond(&mut b).is_ok());
    let mut c = h.open(&operation, 12);
    assert!(h.respond(&mut c).is_ok());
}

#[test]
fn unpolled_owner_close_revokes_publication_and_retains_pending_work() {
    let h = Harness::new();
    let (owner, handle) = h.build(CredentialConfig::new(2));
    let operation = handle.prepare(input()).unwrap();
    let mut session = h.open(&operation, 20);
    let mut output = match h.respond(&mut session).unwrap() {
        AuthStep::Response(output) => output,
        _ => panic!("expected response"),
    };
    let mut queue = EndpointSendQueue::new(peer(20), 1).unwrap();
    h.provider
        .cleanup_ready
        .store(false, std::sync::atomic::Ordering::SeqCst);
    drop(owner.close());
    assert_eq!(output.commit(&mut queue), Err(AuthError::ProviderClosed));
    drop(output);
    drop(session);
    drop(operation);
    drop(handle);
    h.drive();
    assert_eq!(h.retainer.pending(), 1);
    h.provider
        .cleanup_ready
        .store(true, std::sync::atomic::Ordering::SeqCst);
    h.drive();
    assert_eq!(h.retainer.pending(), 0);
    assert_eq!(h.retainer.reservations(), 0);
}
