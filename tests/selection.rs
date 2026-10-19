mod support;
use gwz_credential::{provider::*, *};
use std::{
    path::PathBuf,
    sync::Arc,
    task::{Context, Poll},
};
use support::*;

#[test]
fn equal_selection_with_inflated_target_or_native_path_is_charged_before_retention() {
    let h = Harness::new();
    let mut config = CredentialConfig::new(1);
    config.max_operation_bytes = 65536;
    let (_owner, handle) = h.build(config);
    let operation = handle.prepare(input()).unwrap();
    let before = handle.snapshot();
    let mut origin = String::with_capacity(128 * 1024);
    origin.push_str("https://git.example");
    assert!(matches!(
        h.acquire(
            &operation,
            ExchangeSelection {
                target: Target::Https { origin },
                source: Source::HttpHelper
            }
        ),
        Err(AuthError::InputBudget)
    ));
    assert_eq!(handle.snapshot(), before);
    let choice = ExchangeSelection {
        target: Target::Ssh {
            host: "git.example".into(),
            user: "git".into(),
        },
        source: Source::ExplicitKey("key".into()),
    };
    let mut captured = input();
    captured.allowed = vec![choice.clone()];
    let operation = handle.prepare(captured).unwrap();
    let before = handle.snapshot();
    let mut path = PathBuf::with_capacity(128 * 1024);
    path.push("key");
    assert!(matches!(
        h.acquire(
            &operation,
            ExchangeSelection {
                source: Source::ExplicitKey(path),
                ..choice
            }
        ),
        Err(AuthError::InputBudget)
    ));
    assert_eq!(handle.snapshot(), before);
}
struct EmptyProvider;
struct EmptySelection;
impl Provider for EmptyProvider {
    fn acquisition_charge(
        &self,
        _: &ExchangeSelection,
        _: &AcquisitionPurpose,
    ) -> Result<AcquisitionCharge, AuthError> {
        Ok(AcquisitionCharge {
            snapshot_bytes: 0,
            work_bytes: 0,
        })
    }
    fn start_acquire(
        &self,
        input: &CredentialInput,
        selection: &ExchangeSelection,
        _: &AcquisitionPurpose,
        _: SourceControl,
    ) -> Result<Box<dyn ProviderAcquisition>, AuthError> {
        assert!(input.allowed.contains(selection));
        Ok(Box::new(EmptyAcquisition))
    }
}
struct EmptyAcquisition;
impl ProviderAcquisition for EmptyAcquisition {
    fn poll_acquire(
        &mut self,
        _: &mut Context<'_>,
    ) -> Poll<Result<Arc<dyn SelectedCredential>, AuthError>> {
        Poll::Ready(Ok(Arc::new(EmptySelection)))
    }
    fn poll_close(&mut self, _: &mut Context<'_>, _: bool) -> Poll<Result<(), AuthError>> {
        Poll::Ready(Ok(()))
    }
}
impl SelectedCredential for EmptySelection {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn same_material(&self, other: &dyn SelectedCredential) -> bool {
        other.as_any().is::<Self>()
    }
    fn retained_bytes(&self) -> usize {
        0
    }
    fn conversation_charge(&self) -> usize {
        0
    }
    fn scope(&self) -> SecurityScope {
        SecurityScope::Request
    }
    fn start(
        &self,
        _: &PeerContext,
        _: u64,
        _: Cancellation,
    ) -> Result<Box<dyn ProviderSession>, AuthError> {
        Err(AuthError::Unsupported)
    }
}
#[test]
fn zero_charge_provider_cannot_create_unbounded_free_selections() {
    let h = Harness::new();
    let mut config = CredentialConfig::new(2);
    config.max_operation_bytes = 65536;
    let (_owner, handle) = build(
        config,
        ProviderSet(Arc::new(EmptyProvider)),
        CredentialPorts {
            clock: h.clock.clone(),
            progress: Arc::new(TestProgress),
            retention: h.retainer.clone(),
        },
    )
    .unwrap();
    let operation = handle.prepare(input()).unwrap();
    let baseline = handle.snapshot();
    let mut held = Vec::new();
    loop {
        match h.acquire(&operation, selection()) {
            Ok(selected) => held.push(selected),
            Err(AuthError::InputBudget) => break,
            Err(error) => panic!("{error}"),
        }
        assert!(held.len() < 1000, "library metadata was not charged");
    }
    assert!(!held.is_empty());
    assert!(handle.snapshot().retained_bytes <= 65536);
    drop(held);
    assert_eq!(handle.snapshot(), baseline);
    drop(operation);
    assert_eq!(handle.snapshot(), Snapshot::default());
    assert_eq!(h.retainer.reservations(), 0);
}

// Enclosing platform module preserves disabled-branch scope.
#[cfg(unix)]
mod unix {
    use super::*;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    #[test]
    fn native_non_utf8_and_newline_key_names_remain_exact() {
        let h = Harness::new();
        let (_owner, handle) = build(
            CredentialConfig::new(3),
            ProviderSet(Arc::new(EmptyProvider)),
            CredentialPorts {
                clock: h.clock.clone(),
                progress: Arc::new(TestProgress),
                retention: h.retainer.clone(),
            },
        )
        .unwrap();
        for bytes in [b"key-\xff".to_vec(), b"key\nname".to_vec()] {
            let path = PathBuf::from(OsString::from_vec(bytes));
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
            drop(h.acquire(&operation, choice).unwrap());
            drop(operation);
            assert_eq!(handle.snapshot(), Snapshot::default());
        }
    }
}
