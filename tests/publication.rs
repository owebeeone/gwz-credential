mod support;
use gwz_credential::*;
use support::*;

fn response(h: &Harness, session: &mut AuthSession) -> AuthOutput {
    match h.respond(session).unwrap() {
        AuthStep::Response(output) => output,
        _ => panic!("response"),
    }
}
struct Sink(Vec<u8>, PeerContext);
impl LocalAuthSink for Sink {
    fn peer(&self) -> &PeerContext {
        &self.1
    }
    fn send(&mut self, bytes: &[u8]) -> Result<(), AuthError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

#[test]
fn commit_before_cancel_can_send_but_never_claims_peer_acceptance() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(1));
    let operation = handle.prepare(input()).unwrap();
    let mut session = h.open(&operation, 1);
    let mut output = response(&h, &mut session);
    let mut queue = EndpointSendQueue::new(peer(1), 1).unwrap();
    output.commit(&mut queue).unwrap();
    assert_eq!(output.commit(&mut queue), Err(AuthError::WrongState));
    let end = h.run(session.cancel());
    assert!(end.publication_committed);
    assert_eq!(end.peer_result, PeerAuthResult::Unknown);
    assert_eq!(end.disposition, PhysicalDisposition::Retire);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 1); // Queue still pins charged output.
    let mut wrong = Sink(Vec::new(), peer(99));
    assert_eq!(queue.deliver_next(&mut wrong), Err(AuthError::PeerMismatch));
    assert!(wrong.0.is_empty());
    let mut sink = Sink(Vec::new(), peer(1));
    assert!(queue.deliver_next(&mut sink).unwrap());
    assert_eq!(sink.0, b"token");
    h.drive();
    assert_eq!(handle.snapshot().conversations, 0);
}

#[test]
fn cancel_before_commit_never_exposes_material_and_wrong_peer_cannot_consume_it() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(2));
    let operation = handle.prepare(input()).unwrap();
    let mut session = h.open(&operation, 1);
    let mut output = response(&h, &mut session);
    let mut wrong = EndpointSendQueue::new(peer(2), 1).unwrap();
    assert_eq!(output.commit(&mut wrong), Err(AuthError::PeerMismatch));
    assert!(wrong.is_empty());
    let end = h.run(session.cancel());
    let mut queue = EndpointSendQueue::new(peer(1), 1).unwrap();
    assert_eq!(output.commit(&mut queue), Err(AuthError::Cancelled));
    assert!(queue.is_empty());
    assert!(!end.publication_committed);
    drop(output);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 0);
}

#[test]
fn response_backpressure_keeps_one_envelope_instead_of_growing_records() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(3));
    let operation = handle.prepare(input()).unwrap();
    let mut session = h.open(&operation, 1);
    let output = response(&h, &mut session);
    let bytes = handle.snapshot().retained_bytes;
    for _ in 0..100 {
        assert!(matches!(
            h.respond(&mut session),
            Err(AuthError::WouldBlock)
        ));
        assert_eq!(handle.snapshot().retained_bytes, bytes);
    }
    drop(output);
    assert!(h.respond(&mut session).is_ok());
}

#[test]
fn scope_grant_checks_new_operation_policy_and_revocation_at_commit() {
    let h = Harness::new();
    let (owner, handle) = h.build(CredentialConfig::new(4));
    let native = ExchangeSelection {
        source: Source::WindowsLogon,
        ..selection()
    };
    let mut a_input = input();
    a_input.allowed = vec![native.clone()];
    let a = handle.prepare(a_input).unwrap();
    let session = match h.run(
        a.context()
            .select(native.clone())
            .unwrap()
            .begin(peer(1), AuthControl::new(1000))
            .wait(),
    ) {
        Ok(session) => session,
        Err(error) => panic!("{error}"),
    };
    let end = h.run(session.finish(PeerAuthResult::Accepted));
    let scope = end.scope.unwrap();
    assert_eq!(end.disposition, PhysicalDisposition::BoundScope);
    drop(a); // Transferred scope survives its original operation.
    let helper_only = handle.prepare(input()).unwrap();
    assert!(matches!(
        scope.grant(&helper_only.context(), peer(1), AuthControl::new(1000)),
        Err(AuthError::PeerMismatch)
    ));
    let mut b_input = input();
    b_input.allowed = vec![native];
    let b = handle.prepare(b_input).unwrap();
    let grant = scope
        .grant(&b.context(), peer(1), AuthControl::new(1000))
        .unwrap();
    drop(owner.close());
    let mut queue = EndpointSendQueue::new(peer(1), 1).unwrap();
    assert_eq!(grant.commit(&mut queue), Err(AuthError::ProviderClosed));
    assert!(queue.is_empty());
    drop(scope);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 0);
}
