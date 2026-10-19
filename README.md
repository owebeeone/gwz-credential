# gwz-credential

Portable implementation of the reviewed GWZ Credential API and the
Connection–Credential boundary. This local member is unpublished.

Scope: explicit provider/operation/conversation ownership, bounded accounting,
deadline/cancellation, endpoint-local response capabilities and retained cleanup.
Providers, verified peer facts, clocks and retention/progress services are injected.
There are no sockets, Git/core/Connection dependencies, OS identity lookups,
credential helpers, native workers or production integration here. Real provider
implementations, offer parsing and SSH cryptography are not supplied by this chunk.

## Lifecycle

1. The composition root calls `build(config, ProviderSet(provider), ports)` and
   retains the unique `CredentialOwner`. `CredentialHandle` is selection admission
   authority; it cannot close the provider.
2. `handle.prepare(input)` captures charged, owned endpoint policy and caller
   identity. Retain the unique `OperationCredentials`; its cloneable `context()`
   is selection-only. One operation can issue many independent conversations.
3. `context.select(selection)?.begin(peer, control).wait().await` starts one
   conversation. The caller must provide verified, current endpoint/target/
   generation facts and an absolute deadline in the injected clock's units.
4. `session.respond(challenge).await` returns `AwaitPeer`, `ProviderComplete` or
   an opaque `AuthOutput`. The latter commits once into a bounded endpoint send
   queue. Only the trusted local network adapter drains that queue. Provider
   completion never establishes peer acceptance.
5. `session.finish(actual_peer_result).await` reports disposition and a separate
   cleanup receipt. `session.cancel().await`, operation close and provider close
   revoke their own scope. Dropping an opening/response/session also cancels its
   conversation. Dropping terminal observation never stops retained cleanup.
6. A permitted connection-bound authentication yields a `ScopeLease`. It grants
   reuse only to matching captured caller/policy/peer/generation, with a fresh
   operation control and checked publication. Basic/helper credentials are not
   connection-scoped.

Explicit SSH keys use `LocalSshKeyPort`: the selected key snapshot is borrowed
only by the trusted endpoint-native port, stays pinned across `NeedNetwork`, and
requires `KeyCleanup` confirmation before release. No key path is reopened.

## Integration obligations

`provider` contains privileged adapter interfaces, not an untrusted application
API. Providers must declare retained charges, return promptly from selection and
polls, and not start native authentication during selection. `start` errors must
be effect-free; registered native failures return a session that can be closed.
Private-key borrows and the local output sink must remain at the executing
endpoint. In-process malicious code is outside this trust boundary.

The composition root supplies one shared retention budget/service. It reserves
before effects and retains every registration until `advance` returns Ready,
even if all public handles disappear. It must schedule bounded periodic polls
in addition to native/wake notifications so timeouts and cancellation flags are
observed even if the provider stalls without waking. No hidden executor/thread
is created here. Shutdown of the external service must preserve pending work;
this crate cannot make a broken or dropped service continue executing.

Terminal reports are immediate and can report Pending. Native cleanup errors
or missing key-disposal proof retain capacity; they never imply disposal. A
Confirmed provider/operation report covers conversation records, while still-held
input/selection handles remain charged until dropped. An output committed before
revocation can already be queued/sent; cancellation cannot unsend it. Physical
connection retirement remains the network adapter's responsibility.

Defaults: 72 operations, 64 conversations, 16 MiB per operation, 64 MiB per owner,
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
