//! Contract counterexample: inline source admission has no retained pending state.
//! This deliberately stalled adapter is not a conforming production provider.
mod support;
use gwz_credential::{provider::*, *};
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};
use support::*;

struct StalledSource {
    entered: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}
impl Provider for StalledSource {
    fn selection_charge(&self, _: &ExchangeSelection) -> Result<usize, AuthError> {
        Ok(1024)
    }
    fn select(
        &self,
        _: &CredentialInput,
        _: &ExchangeSelection,
    ) -> Result<Arc<dyn SelectedCredential>, AuthError> {
        // Model a source read that has entered its blocking OS operation. There is
        // no provider session yet and select receives no cancellation/deadline.
        self.entered.send(()).unwrap();
        self.release.lock().unwrap().recv().unwrap();
        Err(AuthError::ProviderRejected)
    }
}

#[test]
fn inline_source_stall_has_no_retained_work_and_cannot_observe_close() {
    let h = Harness::new();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (owner, handle) = build(
        CredentialConfig::new(1),
        ProviderSet(Arc::new(StalledSource {
            entered: entered_tx,
            release: Mutex::new(release_rx),
        })),
        CredentialPorts {
            clock: h.clock.clone(),
            progress: Arc::new(TestProgress),
            retention: h.retainer.clone(),
        },
    )
    .unwrap();
    let selection = ExchangeSelection {
        target: Target::Ssh {
            host: "git.example".into(),
            user: "git".into(),
        },
        source: Source::ExplicitKey("synthetic-endpoint-file".into()),
    };
    let mut captured = input();
    captured.allowed = vec![selection.clone()];
    let operation = handle.prepare(captured).unwrap();
    let context = operation.context();
    let (done_tx, done_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let result = context.select(selection).err();
        done_tx.send(result).unwrap();
    });
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let charged = handle.snapshot().retained_bytes;
    drop(owner.close());
    drop(operation.close());
    h.drive();
    // Purely structural assertions after a barrier; no sleep/timing threshold.
    let returned_early = done_rx.try_recv();
    let registered = h.retainer.pending();
    let retained = handle.snapshot().retained_bytes;
    // Always unblock/join this owned probe thread before checking observations.
    release_tx.send(()).unwrap();
    worker.join().unwrap();
    assert!(matches!(returned_early, Err(mpsc::TryRecvError::Empty)));
    assert_eq!(registered, 0);
    assert_eq!(retained, charged);
    assert_eq!(done_rx.recv().unwrap(), Some(AuthError::ProviderRejected));
    assert_eq!(handle.snapshot().retained_bytes, 0);
}
