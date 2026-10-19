# Portable Credential implementation

Status: portable implementation checkpoint for dual Code/State review;
not accepted, integrated or released.

Authorized by the operator after the accepted Credential API and boundary
reviews. Editable: this new member's portable library, tests/docs and GWZ-managed
registration. All core/CLI/Python/Connection/transport/SSPI source and builder
closeout lanes are reference-only. No native provider or release is included.

Baseline root before registration: 5de3d3f8dedcb8319435cc27163e2809fdbcd5e5.
Root authorities: dev-docs/GwzCredentialApiDesign.md (joint review c0fe3cf8)
and dev-docs/GwzConnectionCredentialBoundary.md (review 0570a27e).

Deliver a standalone Rust library and deterministic fake-provider lifecycle,
multi-conversation, output-publication, local-key, deadline, capacity and retained
cleanup proofs. Pin zeroize to the already-used 1.9.0; no dependency upgrades.
Tests must build without core, Git, Connection, sockets, installed helpers or
native workers. Runtime/native behavior is not inferred from fake tests.

Stop after portable implementation, focused gates, settled review and report.
Production adapters, native providers, workspace/Python adoption, placement
qualification, repository hosting and publication are separate work.

## Checkpoint evidence

2026-10-19: `cargo test --offline`, `cargo fmt --check`, and
`cargo clippy --offline --all-targets -- -D warnings` pass from this member.
`cargo tree --offline` contains this package and zeroize 1.9.0 only.
The suite includes 10,000 sequential operations without retained tombstones,
500 seeded lifecycle schedules, barrier-controlled native-key cancellation,
multiple independent conversations, absolute deadlines, bounded admission,
publication/revocation ordering, connection-scope reuse checks and compile-fail
owner/secret interface checks. Fake providers supply all authentication material.

The injected retention service must progress independently and poll on a bounded
cadence even without native wakeups. Tests step it manually. Production scheduling,
source providers, HTTP offer parsing, SSH algorithms, platform adapters and the
composition bridge are not implemented; native/platform/placement or real
connection-reuse qualification is not claimed. The public ports are privileged
trusted adapter boundaries, not a sandbox against malicious in-process code.

Review: dual Code/State, Surface folded into Code. Exact member commit is recorded
in review prompts; root controlling documents are pinned at immutable commits,
not the concurrently advancing builder HEAD. Inspect only this member and those
authorities; adjacent builder changes are outside the object. File complete
reports verbatim here. At most two remediation rounds; classify architectural
roots and stop for operator decision on a third new architectural root.

## Round 1 correction evidence

Both initial reports are filed verbatim. The two axes independently identified
the same missing observation boundary (one architectural implementation root),
plus local transition, accounting and native-path defects. See RemPlan-1 for the
complete mapping. The member-only patch adds pre-effect charged observation,
immutable terminal facts with separate monotonic revocation, and the matching
cleanup receipt. Pending scope transfer is settled against cancellation/deadline;
errors cannot be replaced by later shutdown. Key failure/panic seals re-entry
without freeing its native pin. Selection allocation includes kernel capacities
and structural overhead; native paths remain PathBuf.

Regressions cover each reported sequence through the public ports, including
observation surviving all ordinary handles, lost finish/cancel futures, native
probe delay, both transfer/cancel orders, oversized equal string/path allocations,
zero-charge providers and Unix non-UTF-8/newline paths. The 10,000-operation test
now explicitly drops its charged terminal receipt before expecting zero usage.
No native/provider or adjacent code was changed. Reviewed status still pending.

## Round 2 correction evidence

At edf0d776, State returned GO and Code closed every original finding but found
P2-5: the new long-lived observation omitted its separately allocated fixed peer
payload. The correction adds that payload and explicit two-Arc overhead to its
retained charge. A public-port regression retains completed observations to
budget refusal, checks owner/shared surviving storage charges and last-clone
release. No API, lifecycle, dependencies, native or adjacent sources changed.
All standalone tests, formatter, clippy -D warnings and cargo doc pass. Final
Code/State closure is pending; cumulative architectural roots remain one.
