mod support;
use gwz_credential::*;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::Ordering,
};
use support::*;

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn seeded_publication_cancellation_and_cleanup_schedules_replay() {
    let seed_text =
        std::env::var("GWZ_CREDENTIAL_SEED").unwrap_or_else(|_| "47575a43524544".into());
    let seed = u64::from_str_radix(seed_text.trim_start_matches("0x"), 16).expect("hex seed");
    assert_ne!(seed, 0, "xorshift seed must be nonzero");
    let cases: usize = std::env::var("GWZ_CREDENTIAL_CASES")
        .map_or(Ok(500), |n| n.parse())
        .expect("case count");
    let source = option_env!("GWZ_CREDENTIAL_SOURCE").unwrap_or("unrecorded-working-tree");
    let mut rng = seed;
    for case in 0..cases {
        let schedule = next(&mut rng);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let h = Harness::new();
            let (owner, handle) = h.build(CredentialConfig::new(case as u128));
            let operation = handle.prepare(input()).unwrap();
            if schedule & 1 == 0 {
                let opening = h
                    .acquire(&operation, selection())
                    .unwrap()
                    .begin(peer(1), AuthControl::new(1000));
                if schedule & 2 != 0 {
                    h.drive();
                }
                drop(opening);
            } else {
                let mut session = h.open(&operation, 1);
                let mut output = match h.respond(&mut session).unwrap() {
                    AuthStep::Response(output) => output,
                    _ => panic!("missing response"),
                };
                let mut queue = EndpointSendQueue::new(peer(1), 1).unwrap();
                h.provider.cleanup_ready.store(false, Ordering::SeqCst);
                if schedule & 2 == 0 {
                    drop(session.cancel());
                    assert_eq!(output.commit(&mut queue), Err(AuthError::Cancelled));
                } else {
                    output.commit(&mut queue).unwrap();
                    drop(session.finish(PeerAuthResult::Accepted));
                }
                h.drive();
                assert_eq!(handle.snapshot().conversations, 1);
                drop(output);
                drop(queue);
                if schedule & 4 != 0 {
                    drop(operation.close());
                } else {
                    drop(operation);
                }
                h.provider.cleanup_ready.store(true, Ordering::SeqCst);
                h.drive();
                drop(owner);
                h.drive();
                assert_eq!(handle.snapshot(), Snapshot::default());
                assert_eq!(h.retainer.reservations(), 0);
                return;
            }
            drop(operation);
            drop(owner);
            h.drive();
            assert_eq!(handle.snapshot(), Snapshot::default());
            assert_eq!(h.retainer.reservations(), 0);
        }));
        assert!(
            result.is_ok(),
            "seed={seed:#x} case={case} schedule={schedule:#x} cases={cases} source={source}; replay with the same source tuple and GWZ_CREDENTIAL_SEED/GWZ_CREDENTIAL_CASES"
        );
    }
}
