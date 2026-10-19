mod support;
use gwz_credential::*;
use support::*;

#[test]
fn acquired_result_is_one_future_and_completed_cancel_preserves_transfer() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(1));
    let operation = handle.prepare(input()).unwrap();
    let mut acquiring = operation.context().acquire(
        selection(),
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    let observation = acquiring.observation().unwrap();
    let prepared = h.run(&mut acquiring).unwrap();
    let terminal = observation.terminal().unwrap();
    assert_eq!(terminal.reason, None);
    assert!(terminal.material_published);
    let end = acquiring.cancel();
    assert_eq!(end.reason, None);
    assert!(end.material_published);
    assert_eq!(observation.terminal(), Some(terminal));
    let session = h
        .run(prepared.begin(peer(1), AuthControl::new(1000)))
        .unwrap();
    drop(h.run(session.cancel()));
}

#[test]
fn completed_error_and_repoll_preserve_original_decision_without_a_second_transfer() {
    use std::pin::Pin;
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(2));
    let operation = handle.prepare(input()).unwrap();
    let mut denied = selection();
    denied.target = Target::Https {
        origin: "https://other.example".into(),
    };
    let mut acquiring = operation.context().acquire(
        denied,
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    assert!(acquiring.observation().is_none());
    assert!(matches!(h.run(&mut acquiring), Err(AuthError::Unsupported)));
    assert!(matches!(
        poll(Pin::new(&mut acquiring)),
        std::task::Poll::Ready(Err(AuthError::WrongState))
    ));
    let end = acquiring.cancel();
    assert_eq!(end.reason, Some(AuthError::Unsupported));
    assert!(!end.material_published);
    assert!(end.cleanup.is_none());
    assert_eq!(end.cleanup_status(), CleanupStatus::Confirmed);
    let mut success = operation.context().acquire(
        selection(),
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    let observation = success.observation().unwrap();
    let prepared = h.run(&mut success).unwrap();
    assert!(matches!(
        poll(Pin::new(&mut success)),
        std::task::Poll::Ready(Err(AuthError::WrongState))
    ));
    let terminal = observation.terminal();
    drop(success);
    assert_eq!(observation.terminal(), terminal);
    assert!(terminal.unwrap().material_published);
    drop(prepared);
}

#[test]
fn private_ready_material_is_cancelled_on_unconsumed_future_drop() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(3));
    let operation = handle.prepare(input()).unwrap();
    let acquiring = operation.context().acquire(
        selection(),
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    let observation = acquiring.observation().unwrap();
    h.drive(); // Provider completed and joined; result is still private/untransferred.
    assert!(observation.terminal().is_none());
    assert_eq!(observation.cleanup().status(), CleanupStatus::Confirmed);
    drop(acquiring);
    let terminal = observation.terminal().unwrap();
    assert_eq!(terminal.reason, Some(AuthError::Cancelled));
    assert!(!terminal.material_published);
    h.drive();
    assert_eq!(handle.snapshot().acquisitions, 0);
    drop(observation);
    drop(operation);
    assert_eq!(handle.snapshot(), Snapshot::default());
}

#[test]
fn original_source_deadline_and_verified_request_binding_survive_transfer() {
    use std::sync::atomic::Ordering;
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(4));
    let operation = handle.prepare(input()).unwrap();
    let prepared = h
        .run(operation.context().acquire(
            selection(),
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(50),
        ))
        .unwrap();
    let mut session = h
        .run(prepared.begin(peer(1), AuthControl::new(1000)))
        .unwrap();
    h.clock.0.store(50, Ordering::SeqCst);
    assert!(matches!(h.respond(&mut session), Err(AuthError::Timeout)));
    drop(session);
    h.drive();
    h.clock.0.store(0, Ordering::SeqCst);
    let prepared = h
        .run(operation.context().acquire(
            selection(),
            AcquisitionPurpose::VerifiedRequest {
                peer: peer(2),
                challenge: AuthChallenge::new(b"offer".to_vec()).unwrap(),
            },
            AuthControl::new(1000),
        ))
        .unwrap();
    assert!(matches!(
        h.run(prepared.begin(peer(3), AuthControl::new(1000))),
        Err(AuthError::PeerMismatch)
    ));
}

#[test]
fn source_observations_keep_their_complete_storage_charged_to_last_clone() {
    let h = Harness::new();
    let mut config = CredentialConfig::new(5);
    config.max_operation_bytes = 65536;
    let (_owner, handle) = h.build(config);
    let operation = handle.prepare(input()).unwrap();
    let baseline = handle.snapshot();
    let mut held = Vec::new();
    loop {
        let acquiring = operation.context().acquire(
            selection(),
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        );
        let observation = acquiring.observation();
        match h.run(acquiring) {
            Ok(prepared) => {
                drop(prepared);
                held.push(observation.unwrap());
            }
            Err(AuthError::InputBudget) => {
                assert!(observation.is_none());
                break;
            }
            Err(error) => {
                panic!("unexpected admission: {error}");
            }
        }
        h.drive();
        assert!(held.len() < 1000, "uncharged source-observation allocation");
    }
    h.drive();
    assert!(!held.is_empty());
    assert_eq!(handle.snapshot().acquisitions, 0);
    assert!(handle.snapshot().retained_bytes > baseline.retained_bytes);
    assert!(handle.snapshot().retained_bytes <= 65536);
    let last = held[0].clone();
    drop(held);
    assert!(handle.snapshot().retained_bytes > baseline.retained_bytes);
    drop(last);
    assert_eq!(handle.snapshot(), baseline);
}

#[test]
fn an_unpolled_failed_future_keeps_both_source_choice_and_observation_payload_charged() {
    let h = Harness::new();
    let (owner, handle) = h.build(CredentialConfig::new(6));
    let host = "h".repeat(4096);
    let user = "u".repeat(4096);
    let payload_floor = 2 * (host.capacity() + user.capacity());
    let choice = ExchangeSelection {
        target: Target::Ssh { host, user },
        source: Source::ExplicitKey("synthetic".into()),
    };
    let mut captured = input();
    captured.allowed = vec![choice.clone()];
    let operation = handle.prepare(captured).unwrap();
    let baseline = handle.snapshot().retained_bytes;
    let acquiring = operation.context().acquire(
        choice,
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    let observation = acquiring.observation().unwrap();
    h.drive();
    drop(owner);
    h.drive();
    assert_eq!(handle.snapshot().acquisitions, 0);
    assert_eq!(observation.cleanup().status(), CleanupStatus::Confirmed);
    assert!(handle.snapshot().retained_bytes - baseline >= payload_floor);
    drop(acquiring);
    drop(observation);
    assert_eq!(handle.snapshot().retained_bytes, baseline);
}

#[test]
fn provider_receives_the_earlier_source_or_begin_cutoff_and_cleanup_stays_owned() {
    use std::sync::atomic::Ordering;
    for (source, begin) in [(50, 1000), (1000, 50)] {
        let h = Harness::new();
        let (_owner, handle) = h.build(CredentialConfig::new(7));
        let operation = handle.prepare(input()).unwrap();
        let prepared = h
            .run(operation.context().acquire(
                selection(),
                AcquisitionPurpose::BeforeLookup,
                AuthControl::new(source),
            ))
            .unwrap();
        let mut session = h
            .run(prepared.begin(peer(1), AuthControl::new(begin)))
            .unwrap();
        assert_eq!(*h.provider.start_deadlines.lock().unwrap(), vec![50]);
        let observation = session.observation();
        h.provider.cleanup_ready.store(false, Ordering::SeqCst);
        h.clock.0.store(50, Ordering::SeqCst);
        assert!(matches!(h.respond(&mut session), Err(AuthError::Timeout)));
        drop(session);
        h.drive();
        assert_eq!(observation.cleanup().status(), CleanupStatus::Pending);
        h.provider.cleanup_ready.store(true, Ordering::SeqCst);
        h.drive();
        assert_eq!(observation.cleanup().status(), CleanupStatus::Confirmed);
        drop(observation);
        drop(operation);
        h.drive();
        assert_eq!(handle.snapshot(), Snapshot::default());
    }
}

#[test]
fn retained_source_observations_attribute_the_original_operation_after_caller_loss() {
    let h = Harness::new();
    let (_owner, handle) = h.build(CredentialConfig::new(8));
    let a = handle.prepare(input()).unwrap();
    let b = handle.prepare(input()).unwrap();
    let a1 = a.context().acquire(
        selection(),
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    let a2 = a.context().acquire(
        selection(),
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    let b1 = b.context().acquire(
        selection(),
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    let observed = h.retainer.source_observations();
    let operations: Vec<_> = observed.iter().map(|o| o.operation_id()).collect();
    let acquisitions: Vec<_> = observed.iter().map(|o| o.id()).collect();
    assert_eq!(operations[0], operations[1]);
    assert_ne!(operations[0], operations[2]);
    assert_ne!(acquisitions[0], acquisitions[1]);
    assert_ne!(acquisitions[1], acquisitions[2]);
    drop(a1);
    drop(a2);
    drop(b1);
    drop(a);
    drop(b);
    h.drive();
    for (index, observation) in observed.iter().enumerate() {
        assert_eq!(observation.operation_id(), operations[index]);
        assert_eq!(observation.id(), acquisitions[index]);
        assert_eq!(observation.cleanup().status(), CleanupStatus::Confirmed);
        assert!(!observation.terminal().unwrap().material_published);
    }
    drop(observed);
    assert_eq!(handle.snapshot(), Snapshot::default());
}
