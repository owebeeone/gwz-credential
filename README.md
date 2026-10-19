# gwz-credential

Portable implementation of the reviewed GWZ Credential API and the
Connection–Credential boundary. The source is public at
[owebeeone/gwz-credential](https://github.com/owebeeone/gwz-credential);
the crate is not published to a package registry.

Scope: explicit provider/operation/conversation ownership, bounded accounting,
deadline/cancellation, endpoint-local response capabilities and retained cleanup.
Providers, verified peer facts, clocks and retention/progress services are injected.
There are no sockets, Git/core/Connection dependencies, OS identity lookups,
credential helpers or application integration here. `ExplicitKeyProvider` supplies
one real file source with a bounded retained native read worker. Offer parsing,
SSH cryptography and the actual native SSH authentication adapter are not supplied.

## Lifecycle

1. The composition root calls `build(config, ProviderSet(provider), ports)` and
   retains the unique `CredentialOwner`. `CredentialHandle` is operation admission
   authority; it cannot close the provider.
2. `handle.prepare(input)` captures charged, owned endpoint policy and caller
   identity. Retain the unique `OperationCredentials`; its cloneable `context()`
   admits controlled source acquisitions. One operation can issue many independent
   acquisitions and authentication conversations.
3. `context.acquire(selection, purpose, control).await?` yields one immutable
   `PreparedCredential`. The returned unique `CredentialAcquiring` is a Send/Unpin
   Future; its Output is `Result<PreparedCredential, AuthError>`. Work starts only
   through the independent retainer after admission, even before the first poll.
   Failed admission gives a ready error and no observation or source effects.
   For explicit keys use `AcquisitionPurpose::BeforeLookup` before EVERY pool
   lookup. Compare `prepared.source_identity()` using `same_material`; equality
   is exact admitted bytes and required lineage, not permission to reuse a connection.
4. `prepared.begin(peer, control).wait().await` starts actual-peer authentication.
   Supply verified current endpoint/target/generation facts and absolute controls
   in the injected clock's units. A later control cannot extend the original
   acquisition deadline. Other source providers can require `VerifiedRequest`
   acquisition with the actual origin/peer/challenge; do not contact them early.
5. `session.respond(challenge).await` returns `AwaitPeer`, `ProviderComplete` or
   an opaque `AuthOutput`. The latter commits once into a bounded endpoint send
   queue. Only the trusted local network adapter drains that queue. Provider
   completion never establishes peer acceptance.
6. `session.finish(actual_peer_result).await` reports disposition and a separate
   cleanup receipt. `session.cancel().await`, operation close and provider close
   revoke their own scope. Dropping an opening/response/session also cancels its
   conversation. Dropping terminal observation never stops retained cleanup.
Acquisition has its own peer-free `AcquisitionObservation` and cleanup receipt.
`acquiring.observation()` is available before polling. `acquiring.cancel()` returns
an immediate `AcquisitionEnd`: a failure reason, publication flag and optional
cleanup receipt. Pending/unpolled Drop seals unpublished work. After successful
manual/borrowed polling, cancel only reports the fixed success and does not revoke
the transferred prepared credential; completed Drop releases its view only.
Re-poll returns WrongState without a second transfer. A stalled read remains
charged with Pending cleanup until actual native exit/join, even after caller loss.

The retainer obtains `WorkObservation::Acquisition` or a cloneable
`WorkObservation::Authentication(AuthObservation)` from `RetainedWork` before
provider effects; an opening/session can also supply it. It identifies the
conversation and physical generation, exposes a fixed terminal record and
matching cleanup receipt, and has no release or scope-transfer authority.
Dropping finish/cancel observation does not hide the terminal outcome. A potential
scope transfer remains undecided until transfer, cancellation, deadline or Drop;
once recorded, terminal facts do not change. Later provider revocation is a
separate monotonic `revoked()` fact that requires retiring any transferred scope.
Observation allocations remain charged until their last clone disappears.

7. A permitted connection-bound authentication yields a `ScopeLease`. It grants
   reuse only to matching captured caller/policy/peer/generation, with a fresh
   operation control and checked publication. Basic/helper credentials are not
   connection-scoped.

Explicit SSH keys use `LocalSshKeyPort`: the selected key snapshot is borrowed
only by the trusted endpoint-native port, stays pinned across `NeedNetwork`, and
requires `KeyCleanup` confirmation before release. No key path is reopened.

## Integration obligations

`provider` contains privileged adapter interfaces, not an untrusted application
API. Providers must declare retained charges, return promptly from admission/start/polls, and not start native authentication
during acquisition. Source acquisition and internal immutable selection are
distinct from native SSH authentication. `start` errors must
be effect-free; registered native failures return a session that can be closed.
Explicit-key source paths use native `PathBuf` without UTF-8 conversion. Kernel
selection metadata is charged separately from declared provider storage.
Private-key borrows and the local output sink must remain at the executing
endpoint. In-process malicious code is outside this trust boundary.

The composition root supplies one shared retention budget/service. It reserves
before effects and retains every registration until `advance` returns Ready,
even if all public handles disappear. It must schedule bounded periodic polls
in addition to native/wake notifications so timeouts and cancellation flags are
observed even if the provider stalls without waking. The explicit-file source creates one admitted native worker per source job;
there is no internal executor or reaper. Its fixed 2 MiB stack is included in the
work reservation. Shutdown of the external service must preserve pending work;
this crate cannot make a broken or dropped service continue executing.

Terminal reports are immediate and can report Pending. Native cleanup errors
or missing key-disposal proof retain capacity; they never imply disposal. A
Confirmed provider/operation report covers source and conversation records, while still-held
input/selection handles remain charged until dropped. An output committed before
revocation can already be queued/sent; cancellation cannot unsend it. Physical
connection retirement remains the network adapter's responsibility.

Defaults: 72 operations, 64 combined source/authentication records, 16 MiB per operation, 64 MiB per owner,
8 rounds, maximum terminal-report allowance 5 seconds (reports here are immediate).
Shared retention admission also bounds replacement owners. Opaque response and
challenge buffers are capped at 64 KiB each; the selected provider declares and
is responsible for its additional retained allocation. Captured caller equality
uses the provider's full identity comparison, not just its identifier hint.

## Standalone verification

Run `cargo test --offline`, `cargo fmt --check`, and
`cargo clippy --offline --all-targets -- -D warnings`.
The fake-provider suite needs no other GWZ crate, wire, host credentials or native
worker. `tests/support` is an illustrative manual retention driver, not a
production scheduling service. The schedule test prints replay seed and case on
failure; set `GWZ_CREDENTIAL_SEED` (hex) and `GWZ_CREDENTIAL_CASES` to replay.

See [implementation status](dev-docs/Implementation.md). Passing portable tests
does not qualify native authentication, endpoint placement or connection reuse
through CLI/Python. No release or publishing is authorized by this implementation.

## Explicit file source

Use `ProviderSet(Arc::new(ExplicitKeyProvider::new()))` at the executing endpoint.
`CredentialInput.native_base` captures an absolute native base for relative key
paths; an absolute key path needs no base. The worker never resolves against its
ambient cwd, converts native filenames to UTF-8 or falls back to another source.
Each acquisition reads a fresh regular-file snapshot, capped at 1 MiB. Empty,
oversized, encrypted, malformed/multiple-key and unsupported native forms return
sanitized typed refusals. Container checks are framing/policy checks; the trusted
native SSH port still determines cryptographic validity. Synthetic test armor is
not a usable signing key.

The source retains exact bytes, including line endings. Snapshot/identity pins
stay charged through their final owner and are zeroized before accounting release.
Source cleanup confirms native job disposal; it does not imply that a successfully
transferred snapshot or identity view has been dropped. On qualified Mac/Linux,
nonblocking opens also fence a regular-file-to-FIFO replacement. On Windows,
namespace/device forms are screened and the opened object must be a regular file.
A filesystem call can still stall indefinitely: cancellation reports promptly,
while the retained worker occupies shared capacity until actual exit.

See the [current bounded task](dev-docs/CredentialExplicitKey-Plan.md) and
[implementation status](dev-docs/Implementation.md) for the separate review and
native qualification gates. This source does not establish real SSH authentication,
endpoint placement or Connection/application reuse.
