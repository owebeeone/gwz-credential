//! File-only projection of core's retained Entry/Exit pattern. No reaper or executor.
use crate::{explicit_key::KeySnapshot, provider::*, state::lock, *};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    task::{Context, Poll},
    thread::{self, JoinHandle},
};
use zeroize::Zeroizing;

pub(crate) const MAX_KEY: usize = 1 << 20;
pub(crate) trait FileRead: Send + Sync {
    fn read(
        &self,
        path: &Path,
        buffer: &mut [u8],
        control: &SourceControl,
    ) -> Result<usize, AuthError>;
}
pub(crate) struct NativeRead;
impl FileRead for NativeRead {
    fn read(
        &self,
        path: &Path,
        buffer: &mut [u8],
        control: &SourceControl,
    ) -> Result<usize, AuthError> {
        crate::key_file::read(path, buffer, control)
    }
}
type ResultSlot = Arc<Mutex<Option<Result<Arc<dyn SelectedCredential>, AuthError>>>>;
pub(crate) trait NativeExit: Send {
    fn finished(&self) -> bool;
    fn join(self: Box<Self>) -> bool;
}
impl NativeExit for JoinHandle<()> {
    fn finished(&self) -> bool {
        self.is_finished()
    }
    fn join(self: Box<Self>) -> bool {
        (*self).join().is_ok()
    }
}
pub(crate) struct FileAcquisition {
    join: Option<Box<dyn NativeExit>>,
    result: ResultSlot,
    control: SourceControl,
}
impl FileAcquisition {
    pub(crate) fn start(
        path: PathBuf,
        control: SourceControl,
        reader: Arc<dyn FileRead>,
    ) -> Result<Box<dyn ProviderAcquisition>, AuthError> {
        let result: ResultSlot = Arc::new(Mutex::new(None));
        let mut owner = Box::new(Self {
            join: None,
            result: result.clone(),
            control: control.clone(),
        });
        control.check()?;
        // One slot was reserved by the shared retainer before this standard
        // native thread. A fixed stack avoids ambient RUST_MIN_STACK changes.
        let join = thread::Builder::new()
            .name("credential-key-read".into())
            .stack_size(2 << 20)
            .spawn(move || {
                let outcome = load(&path, &control, reader.as_ref());
                *lock(&result) = Some(outcome);
                control.wake();
            })
            .map_err(|_| AuthError::ProviderFault)?;
        owner.join = Some(Box::new(join));
        Ok(owner)
    }
    fn joined(&mut self) -> Option<bool> {
        if self.join.as_ref().is_some_and(|join| !join.finished()) {
            return None;
        }
        Some(self.join.take().is_none_or(|join| join.join()))
    }
}
fn load(
    path: &Path,
    control: &SourceControl,
    reader: &dyn FileRead,
) -> Result<Arc<dyn SelectedCredential>, AuthError> {
    control.check()?;
    let mut bytes = Zeroizing::new(vec![0; MAX_KEY + 1]);
    let used = reader.read(path, &mut bytes, control)?;
    control.check()?;
    if used == 0 || used > MAX_KEY {
        return Err(AuthError::Source(SourceFailure::EmptyOrOversize));
    }
    bytes.truncate(used);
    let text =
        std::str::from_utf8(&bytes).map_err(|_| AuthError::Source(SourceFailure::InvalidKey))?;
    crate::key_container::check(text, control)?;
    crate::key_file::check_native_form(text)?;
    control.check()?;
    Ok(Arc::new(KeySnapshot { bytes }))
}
impl ProviderAcquisition for FileAcquisition {
    fn poll_acquire(
        &mut self,
        _: &mut Context<'_>,
    ) -> Poll<Result<Arc<dyn SelectedCredential>, AuthError>> {
        match self.joined() {
            None => Poll::Pending,
            Some(false) => Poll::Ready(Err(AuthError::ProviderFault)),
            Some(true) => Poll::Ready(
                lock(&self.result)
                    .take()
                    .unwrap_or(Err(AuthError::ProviderFault)),
            ),
        }
    }
    fn poll_close(&mut self, _: &mut Context<'_>, cancelled: bool) -> Poll<Result<(), AuthError>> {
        if cancelled {
            self.control.stop();
        }
        if self.joined().is_none() {
            return Poll::Pending;
        }
        let discarded = lock(&self.result).take();
        drop(discarded);
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::Future,
        pin::Pin,
        sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            mpsc,
        },
        task::Waker,
        time::{Duration, Instant},
    };
    struct Fixed;
    impl Clock for Fixed {
        fn now_ms(&self) -> u64 {
            0
        }
    }
    struct Wake;
    impl Progress for Wake {
        fn wake(&self) {}
    }
    struct Caller;
    impl CapturedCaller for Caller {
        fn retained_bytes(&self) -> usize {
            1
        }
        fn identity(&self) -> u128 {
            1
        }
        fn same_identity(&self, other: &dyn CapturedCaller) -> bool {
            (other as &dyn std::any::Any).is::<Self>()
        }
    }
    type Entry = (Arc<dyn RetainedWork>, Box<dyn Reservation>);
    struct Retainer {
        entries: Mutex<Vec<Entry>>,
        slots: Arc<AtomicUsize>,
    }
    struct Permit {
        slots: Arc<AtomicUsize>,
        records: usize,
    }
    impl Reservation for Permit {}
    impl Drop for Permit {
        fn drop(&mut self) {
            self.slots.fetch_sub(self.records, Ordering::SeqCst);
        }
    }
    impl RetentionPort for Retainer {
        fn reserve(&self, charge: RetentionCharge) -> Result<Box<dyn Reservation>, AuthError> {
            if self.slots.fetch_add(charge.records, Ordering::SeqCst) + charge.records > 1 {
                self.slots.fetch_sub(charge.records, Ordering::SeqCst);
                return Err(AuthError::Capacity);
            }
            Ok(Box::new(Permit {
                slots: self.slots.clone(),
                records: charge.records,
            }))
        }
        fn retain(&self, work: Arc<dyn RetainedWork>, reservation: Box<dyn Reservation>) {
            lock(&self.entries).push((work, reservation));
        }
    }
    impl Retainer {
        fn tick(&self) {
            let entries = std::mem::take(&mut *lock(&self.entries));
            for (work, permit) in entries {
                if work
                    .advance(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
                {
                    lock(&self.entries).push((work, permit));
                }
            }
        }
    }
    struct BlockedRead {
        entered: mpsc::SyncSender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl FileRead for BlockedRead {
        fn read(&self, _: &Path, _: &mut [u8], _: &SourceControl) -> Result<usize, AuthError> {
            self.entered.send(()).unwrap();
            // Deliberately cannot observe cancellation while inside the OS boundary.
            let _ = lock(&self.release).recv();
            Err(AuthError::Source(SourceFailure::Io))
        }
    }
    fn control() -> SourceControl {
        SourceControl {
            deadline: 1000,
            request_cancel: Cancellation::default(),
            stopped: Cancellation::default(),
            clock: Arc::new(Fixed),
            progress: Arc::new(Wake),
        }
    }
    #[test]
    fn actual_blocked_native_worker_survives_owner_loss_until_joined_and_never_publishes() {
        let retainer = Arc::new(Retainer {
            entries: Mutex::default(),
            slots: Arc::default(),
        });
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::channel();
        let provider = Arc::new(ExplicitKeyProvider {
            reader: Arc::new(BlockedRead {
                entered: entered_tx,
                release: Mutex::new(release_rx),
            }),
        });
        let (owner, handle) = build(
            CredentialConfig::new(1),
            ProviderSet(provider),
            CredentialPorts {
                clock: Arc::new(Fixed),
                progress: Arc::new(Wake),
                retention: retainer.clone(),
            },
        )
        .unwrap();
        let choice = ExchangeSelection {
            target: Target::Ssh {
                host: "git.example".into(),
                user: "git".into(),
            },
            source: Source::ExplicitKey(
                std::env::current_dir()
                    .unwrap()
                    .join("synthetic-unopened-file"),
            ),
        };
        let operation = handle
            .prepare(CredentialInput {
                endpoint: "endpoint".into(),
                caller: Arc::new(Caller),
                native_base: None,
                allowed: vec![choice.clone()],
            })
            .unwrap();
        let mut acquiring = operation.context().acquire(
            choice,
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        );
        let observation = acquiring.observation().unwrap();
        retainer.tick();
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        drop(owner.close());
        drop(operation);
        retainer.tick();
        let terminal = observation.terminal();
        let disposal_before_exit = observation.cleanup().status();
        let native_slots_before_exit = retainer.slots.load(Ordering::SeqCst);
        let future_result = Pin::new(&mut acquiring).poll(&mut Context::from_waker(Waker::noop()));
        // Release and drive the owned thread to actual join BEFORE assertions.
        release_tx.send(()).unwrap();
        let until = Instant::now() + Duration::from_secs(10);
        while retainer.slots.load(Ordering::SeqCst) != 0 {
            retainer.tick();
            assert!(Instant::now() < until, "native source did not join");
            std::thread::yield_now();
        }
        assert_eq!(terminal.unwrap().reason, Some(AuthError::ProviderClosed));
        assert!(!terminal.unwrap().material_published);
        assert_eq!(disposal_before_exit, CleanupStatus::Pending);
        assert_eq!(native_slots_before_exit, 1);
        assert!(matches!(
            future_result,
            Poll::Ready(Err(AuthError::ProviderClosed))
        ));
        assert_eq!(observation.terminal(), terminal);
        assert_eq!(observation.cleanup().status(), CleanupStatus::Confirmed);
        drop(acquiring);
        drop(observation);
        assert_eq!(handle.snapshot(), Snapshot::default());
    }
    struct ProbeClock(Mutex<Option<std::sync::Weak<crate::acquisition_state::SourceRecord>>>);
    impl Clock for ProbeClock {
        fn now_ms(&self) -> u64 {
            if let Some(record) = lock(&self.0).as_ref().and_then(std::sync::Weak::upgrade) {
                assert!(
                    record.operation.hub.state.try_lock().is_ok(),
                    "clock called under authority lock"
                );
                assert!(
                    record.state.try_lock().is_ok(),
                    "clock called under source lock"
                );
            }
            0
        }
    }
    #[test]
    fn source_clock_callbacks_are_outside_kernel_locks() {
        let retainer = Arc::new(Retainer {
            entries: Mutex::default(),
            slots: Arc::default(),
        });
        let clock = Arc::new(ProbeClock(Mutex::default()));
        let (owner, handle) = build(
            CredentialConfig::new(2),
            ProviderSet(Arc::new(ExplicitKeyProvider::new())),
            CredentialPorts {
                clock: clock.clone(),
                progress: Arc::new(Wake),
                retention: retainer.clone(),
            },
        )
        .unwrap();
        let choice = ExchangeSelection {
            target: Target::Ssh {
                host: "git.example".into(),
                user: "git".into(),
            },
            source: Source::ExplicitKey(
                std::env::current_dir()
                    .unwrap()
                    .join("synthetic-absent-key"),
            ),
        };
        let operation = handle
            .prepare(CredentialInput {
                endpoint: "endpoint".into(),
                caller: Arc::new(Caller),
                native_base: None,
                allowed: vec![choice.clone()],
            })
            .unwrap();
        let mut acquiring = operation.context().acquire(
            choice,
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        );
        *lock(&clock.0) = Some(Arc::downgrade(acquiring.record.as_ref().unwrap()));
        assert!(
            Pin::new(&mut acquiring)
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        retainer.tick();
        let end = acquiring.cancel();
        let until = Instant::now() + Duration::from_secs(10);
        while retainer.slots.load(Ordering::SeqCst) != 0 {
            retainer.tick();
            assert!(Instant::now() < until);
            std::thread::yield_now();
        }
        assert_eq!(end.cleanup_status(), CleanupStatus::Confirmed);
        drop(end);
        drop(operation);
        drop(owner);
        assert_eq!(handle.snapshot(), Snapshot::default());
    }
    struct PanickingRead;
    impl FileRead for PanickingRead {
        fn read(&self, _: &Path, _: &mut [u8], _: &SourceControl) -> Result<usize, AuthError> {
            panic!("synthetic native read failure");
        }
    }
    #[test]
    fn native_panic_is_joined_before_failure_and_cleanup() {
        let mut work =
            FileAcquisition::start(PathBuf::from("unused"), control(), Arc::new(PanickingRead))
                .unwrap();
        let until = Instant::now() + Duration::from_secs(10);
        let mut cx = Context::from_waker(Waker::noop());
        let outcome = loop {
            if let Poll::Ready(outcome) = work.poll_acquire(&mut cx) {
                break outcome;
            }
            assert!(Instant::now() < until, "panicking worker did not finish");
            std::thread::yield_now();
        };
        assert!(matches!(outcome, Err(AuthError::ProviderFault)));
        assert!(matches!(
            work.poll_close(&mut cx, true),
            Poll::Ready(Ok(()))
        ));
    }
    struct DelayedExit {
        exited: Arc<AtomicBool>,
        joins: Arc<AtomicUsize>,
    }
    impl NativeExit for DelayedExit {
        fn finished(&self) -> bool {
            self.exited.load(Ordering::SeqCst)
        }
        fn join(self: Box<Self>) -> bool {
            self.joins.fetch_add(1, Ordering::SeqCst);
            true
        }
    }
    #[test]
    fn stored_result_is_not_native_exit_or_disposal_proof() {
        let exited = Arc::new(AtomicBool::new(false));
        let joins = Arc::new(AtomicUsize::new(0));
        let material = Arc::new(KeySnapshot {
            bytes: Zeroizing::new(vec![1, 2, 3]),
        });
        let pin = Arc::downgrade(&material);
        let mut work = FileAcquisition {
            join: Some(Box::new(DelayedExit {
                exited: exited.clone(),
                joins: joins.clone(),
            })),
            result: Arc::new(Mutex::new(Some(Ok(material)))),
            control: control(),
        };
        let mut cx = Context::from_waker(Waker::noop());
        assert!(work.poll_acquire(&mut cx).is_pending());
        assert!(work.poll_close(&mut cx, true).is_pending());
        assert!(pin.upgrade().is_some());
        assert_eq!(joins.load(Ordering::SeqCst), 0);
        exited.store(true, Ordering::SeqCst);
        assert!(matches!(
            work.poll_close(&mut cx, true),
            Poll::Ready(Ok(()))
        ));
        assert!(pin.upgrade().is_none());
        assert_eq!(joins.load(Ordering::SeqCst), 1);
    }
}
