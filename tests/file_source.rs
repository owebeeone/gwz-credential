mod support;
use gwz_credential::{provider::*, *};
use std::{
    fs,
    future::Future,
    path::PathBuf,
    pin::pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use support::*;

// Synthetic armor accepted for source framing, not a real private signing key.
const KEY_A: &[u8] = b"-----BEGIN RSA PRIVATE KEY-----\nAQID\n-----END RSA PRIVATE KEY-----\n";
struct Fixture(PathBuf);
impl Fixture {
    fn new(label: &str) -> Self {
        let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        fs::create_dir_all(&base).unwrap();
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = base.join(format!("key-{label}-{}-{tick}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run_native<F: Future>(h: &Harness, future: F) -> F::Output {
    let mut future = pin!(future);
    let until = Instant::now() + Duration::from_secs(10);
    let waker = Waker::from(Arc::new(Noop));
    loop {
        h.drive();
        if let Poll::Ready(result) = future.as_mut().poll(&mut Context::from_waker(&waker)) {
            return result;
        }
        assert!(Instant::now() < until, "owned source did not complete");
        std::thread::park_timeout(Duration::from_millis(1));
    }
}
#[test]
fn real_source_reads_once_and_returns_exact_opaque_identity() {
    let h = Harness::new();
    let (_owner, handle) = build(
        CredentialConfig::new(1),
        ProviderSet(Arc::new(ExplicitKeyProvider::new())),
        CredentialPorts {
            clock: h.clock.clone(),
            progress: Arc::new(TestProgress),
            retention: h.retainer.clone(),
        },
    )
    .unwrap();
    let fixture = Fixture::new("snapshot");
    let path = fixture.0.join("key file é");
    fs::write(&path, KEY_A).unwrap();
    let choice = ExchangeSelection {
        target: Target::Ssh {
            host: "git.example".into(),
            user: "git".into(),
        },
        source: Source::ExplicitKey(path),
    };
    let mut captured = input();
    captured.allowed = vec![choice.clone()];
    let operation = handle.prepare(captured).unwrap();
    let prepared = run_native(
        &h,
        operation.context().acquire(
            choice,
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        ),
    )
    .unwrap();
    let identity = prepared.source_identity();
    assert!(identity.same_material(&prepared.source_identity()));
    drop(prepared);
    drop(identity);
    drop(operation);
    h.drive();
    assert_eq!(handle.snapshot(), Snapshot::default());
}

fn choice(path: PathBuf) -> ExchangeSelection {
    ExchangeSelection {
        target: Target::Ssh {
            host: "git.example".into(),
            user: "git".into(),
        },
        source: Source::ExplicitKey(path),
    }
}
fn source_handle(h: &Harness, namespace: u128) -> (CredentialOwner, CredentialHandle) {
    build(
        CredentialConfig::new(namespace),
        ProviderSet(Arc::new(ExplicitKeyProvider::new())),
        CredentialPorts {
            clock: h.clock.clone(),
            progress: Arc::new(TestProgress),
            retention: h.retainer.clone(),
        },
    )
    .unwrap()
}
struct Probe(std::sync::atomic::AtomicBool);
impl KeyCleanup for Probe {
    fn confirmed(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}
struct NativePort {
    peer: PeerContext,
    probe: Arc<Probe>,
    seen: Vec<Vec<u8>>,
}
impl LocalSshKeyPort for NativePort {
    fn peer(&self) -> &PeerContext {
        &self.peer
    }
    fn key_cleanup(&self) -> Arc<dyn KeyCleanup> {
        self.probe.clone()
    }
    fn authenticate(&mut self, key: SecretKey<'_>) -> Result<SshProgress, AuthError> {
        self.seen.push(key.native_bytes().to_vec());
        Ok(SshProgress::NeedNetwork)
    }
}
#[test]
fn replacement_is_reread_for_next_lookup_while_native_reentry_keeps_the_original_snapshot() {
    use std::sync::atomic::{AtomicBool, Ordering};
    const KEY_B: &[u8] = b"-----BEGIN RSA PRIVATE KEY-----\nBAUG\n-----END RSA PRIVATE KEY-----\n";
    let h = Harness::new();
    let (_owner, handle) = source_handle(&h, 2);
    let fixture = Fixture::new("replacement");
    let path = fixture.0.join("selected key");
    let other = fixture.0.join("equal key at another path");
    fs::write(&path, KEY_A).unwrap();
    fs::write(&other, KEY_A).unwrap();
    let selected = choice(path.clone());
    let alternative = choice(other);
    let mut captured = input();
    captured.allowed = vec![selected.clone(), alternative.clone()];
    let operation = handle.prepare(captured).unwrap();
    let original = run_native(
        &h,
        operation.context().acquire(
            selected.clone(),
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        ),
    )
    .unwrap();
    let identity = original.source_identity();
    let mut actual_peer = peer(1);
    actual_peer.target = selected.target.clone();
    let mut session = h
        .run(original.begin(actual_peer.clone(), AuthControl::new(1000)))
        .unwrap();
    let probe = Arc::new(Probe(AtomicBool::new(false)));
    let mut port = NativePort {
        peer: actual_peer,
        probe: probe.clone(),
        seen: vec![],
    };
    session.use_ssh_key(&mut port).unwrap();
    fs::write(&path, KEY_B).unwrap();
    session.use_ssh_key(&mut port).unwrap();
    assert_eq!(port.seen, [KEY_A.to_vec(), KEY_A.to_vec()]);
    let replacement = run_native(
        &h,
        operation.context().acquire(
            selected,
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        ),
    )
    .unwrap();
    let equal_other_path = run_native(
        &h,
        operation.context().acquire(
            alternative,
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        ),
    )
    .unwrap();
    assert!(!identity.same_material(&replacement.source_identity()));
    assert!(identity.same_material(&equal_other_path.source_identity()));
    fs::remove_file(&path).unwrap();
    session.use_ssh_key(&mut port).unwrap(); // no reopen, even after removal
    assert_eq!(port.seen.last().unwrap().as_slice(), KEY_A);
    let end = h.run(session.cancel());
    assert_eq!(end.cleanup.status(), CleanupStatus::Pending);
    drop(replacement);
    drop(equal_other_path);
    drop(identity);
    drop(operation);
    h.drive();
    assert_eq!(handle.snapshot().conversations, 1);
    probe.0.store(true, Ordering::SeqCst);
    h.drive();
    assert_eq!(end.cleanup.status(), CleanupStatus::Confirmed);
    drop(end);
    assert_eq!(handle.snapshot(), Snapshot::default());
}

#[test]
fn relative_path_uses_captured_endpoint_base_and_missing_source_never_falls_back() {
    let h = Harness::new();
    let (_owner, handle) = source_handle(&h, 3);
    let fixture = Fixture::new("base");
    fs::write(fixture.0.join("relative key"), KEY_A).unwrap();
    let relative = choice("relative key".into());
    let missing = choice(fixture.0.join("missing"));
    let agent = ExchangeSelection {
        source: Source::Agent,
        target: relative.target.clone(),
    };
    let mut captured = input();
    captured.native_base = Some(fixture.0.clone());
    captured.allowed = vec![relative.clone(), missing.clone(), agent.clone()];
    let operation = handle.prepare(captured).unwrap();
    assert!(
        run_native(
            &h,
            operation.context().acquire(
                relative,
                AcquisitionPurpose::BeforeLookup,
                AuthControl::new(1000)
            )
        )
        .is_ok()
    );
    assert!(matches!(
        run_native(
            &h,
            operation.context().acquire(
                missing,
                AcquisitionPurpose::BeforeLookup,
                AuthControl::new(1000)
            )
        ),
        Err(AuthError::Source(SourceFailure::NotFound))
    ));
    let refused = operation.context().acquire(
        agent,
        AcquisitionPurpose::BeforeLookup,
        AuthControl::new(1000),
    );
    assert!(refused.observation().is_none());
    assert!(matches!(h.run(refused), Err(AuthError::Unsupported)));
}

#[test]
fn empty_oversize_encrypted_and_malformed_sources_are_typed_refusals() {
    let h = Harness::new();
    let (_owner, handle) = source_handle(&h, 4);
    let fixture = Fixture::new("refusals");
    let path = fixture.0.join("key");
    let selected = choice(path.clone());
    let mut captured = input();
    captured.allowed = vec![selected.clone()];
    let operation = handle.prepare(captured).unwrap();
    let cases = [
        (vec![], SourceFailure::EmptyOrOversize),
        (vec![b'x'; (1 << 20) + 1], SourceFailure::EmptyOrOversize),
        (b"-----BEGIN ENCRYPTED PRIVATE KEY-----\nAQID\n-----END ENCRYPTED PRIVATE KEY-----\n".to_vec(), SourceFailure::EncryptedKey),
        (b"-----BEGIN RSA PRIVATE KEY-----\nProc-Type: 4,ENCRYPTED\nAQID\n-----END RSA PRIVATE KEY-----\n".to_vec(), SourceFailure::EncryptedKey),
        (b"-----BEGIN RSA PRIVATE KEY-----\n@@@\n-----END RSA PRIVATE KEY-----\n".to_vec(), SourceFailure::InvalidKey),
        ([KEY_A, KEY_A].concat(), SourceFailure::InvalidKey),
        (b"not a key\0".to_vec(), SourceFailure::InvalidKey),
    ];
    for (bytes, expected) in cases {
        fs::write(&path, bytes).unwrap();
        let acquiring = operation.context().acquire(
            selected.clone(),
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        );
        let observation = acquiring.observation().unwrap();
        assert!(
            matches!(run_native(&h, acquiring), Err(AuthError::Source(actual)) if actual == expected)
        );
        h.drive();
        assert_eq!(observation.cleanup().status(), CleanupStatus::Confirmed);
        assert!(!observation.terminal().unwrap().material_published);
    }
    let directory = choice(fixture.0.clone());
    let mut captured = input();
    captured.allowed = vec![directory.clone()];
    let directory_operation = handle.prepare(captured).unwrap();
    assert!(matches!(
        run_native(
            &h,
            directory_operation.context().acquire(
                directory,
                AcquisitionPurpose::BeforeLookup,
                AuthControl::new(1000)
            )
        ),
        Err(AuthError::Source(SourceFailure::NonRegular))
    ));
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, process::Command};
    #[cfg(target_os = "macos")]
    mod native_names {
        pub(super) fn unsupported(error: &std::io::Error) -> bool {
            error.raw_os_error() == Some(92)
        }
    }
    #[cfg(not(target_os = "macos"))]
    mod native_names {
        pub(super) fn unsupported(_: &std::io::Error) -> bool {
            false
        }
    }
    #[test]
    fn actual_non_utf8_newline_and_fifo_paths_remain_native() {
        let h = Harness::new();
        let (_owner, handle) = source_handle(&h, 5);
        let fixture = Fixture::new("native-unix");
        for name in [b"key-\xff".to_vec(), b"key\nname".to_vec()] {
            let path = fixture.0.join(OsString::from_vec(name));
            let creation = fs::write(&path, KEY_A);
            if let Err(error) = creation {
                assert!(
                    native_names::unsupported(&error),
                    "unexpected fixture failure: {error}"
                );
                // APFS refuses invalid UTF-8 with EILSEQ. An existing lossy-name
                // sentinel must NOT be read in place of the original native path.
                fs::write(PathBuf::from(path.to_string_lossy().into_owned()), KEY_A).unwrap();
                let native_error = fs::metadata(&path).unwrap_err();
                let expected = if native_error.kind() == std::io::ErrorKind::NotFound {
                    SourceFailure::NotFound
                } else {
                    SourceFailure::Io
                };
                let selected = choice(path);
                let mut captured = input();
                captured.allowed = vec![selected.clone()];
                let operation = handle.prepare(captured).unwrap();
                assert!(matches!(
                    run_native(
                        &h,
                        operation.context().acquire(
                            selected,
                            AcquisitionPurpose::BeforeLookup,
                            AuthControl::new(1000)
                        )
                    ),
                    Err(AuthError::Source(actual)) if actual == expected
                ));
                continue;
            }
            let selected = choice(path);
            let mut captured = input();
            captured.allowed = vec![selected.clone()];
            let operation = handle.prepare(captured).unwrap();
            let prepared = run_native(
                &h,
                operation.context().acquire(
                    selected,
                    AcquisitionPurpose::BeforeLookup,
                    AuthControl::new(1000),
                ),
            )
            .unwrap();
            drop(prepared);
            drop(operation);
            h.drive();
        }
        let fifo = fixture.0.join("fifo");
        assert!(
            Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        let selected = choice(fifo);
        let mut captured = input();
        captured.allowed = vec![selected.clone()];
        let operation = handle.prepare(captured).unwrap();
        assert!(matches!(
            run_native(
                &h,
                operation.context().acquire(
                    selected,
                    AcquisitionPurpose::BeforeLookup,
                    AuthControl::new(1000)
                )
            ),
            Err(AuthError::Source(SourceFailure::NonRegular))
        ));
    }
}

#[test]
fn exact_identity_does_not_normalize_line_endings() {
    let h = Harness::new();
    let (_owner, handle) = source_handle(&h, 6);
    let fixture = Fixture::new("exact-bytes");
    let path = fixture.0.join("key");
    let selected = choice(path.clone());
    let mut captured = input();
    captured.allowed = vec![selected.clone()];
    let operation = handle.prepare(captured).unwrap();
    fs::write(&path, KEY_A).unwrap();
    let first = run_native(
        &h,
        operation.context().acquire(
            selected.clone(),
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        ),
    )
    .unwrap();
    fs::write(
        &path,
        std::str::from_utf8(KEY_A).unwrap().replace('\n', "\r\n"),
    )
    .unwrap();
    let next = run_native(
        &h,
        operation.context().acquire(
            selected,
            AcquisitionPurpose::BeforeLookup,
            AuthControl::new(1000),
        ),
    )
    .unwrap();
    assert!(
        !first
            .source_identity()
            .same_material(&next.source_identity())
    );
}
