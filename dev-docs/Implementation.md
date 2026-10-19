# Credential implementation checkpoints

Current status: **accepted first explicit-file source/library implementation**,
production/tests29b8838b9fb3bf60cda58ab259e7d85039a2da88, after dual GO at
member0d01d6af. See [current merged verdict](CredentialExplicitKey-Verdict.md)
and native platform results. The historical portable checkpoint follows.

Status: **accepted portable implementation at
`d6fc64d6a38457018377020765229405920a4833` after
[Code](CredentialPortable-ReviewCode-2.md) and
[State](CredentialPortable-ReviewState-2.md) both reported GO.**
This accepts the isolated portable kernel only; it is not integrated or released.
See [merged verdict](CredentialPortable-Verdict.md).

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
No native/provider or adjacent code was changed. Round-1 status was pending; final acceptance is recorded above.

## Round 2 correction evidence

At edf0d776, State returned GO and Code closed every original finding but found
P2-5: the new long-lived observation omitted its separately allocated fixed peer
payload. The correction adds that payload and explicit two-Arc overhead to its
retained charge. A public-port regression retains completed observations to
budget refusal, checks owner/shared surviving storage charges and last-clone
release. No API, lifecycle, dependencies, native or adjacent sources changed.
All standalone tests, formatter, clippy -D warnings and cargo doc pass. Final
Code/State closure is GO/GO at the accepted tuple above; architectural roots
remain one, closed.

## Resume boundary

The isolated portable chunk is finished. Stop here: no further work on core,
Connection, Python, native providers or builder closeout is authorized by this
checkpoint. Next provider work must separately select which existing provider
semantics to implement and its focused qualification; composition needs its own
small bridge proof. Do not report the full Credential system, transport release,
credential placement or persistent connection reuse as complete from these fakes.
The retainer/progress implementation remains an explicit composition obligation.

Intermediate prompts, superseded reviews and correction plans have moved into
history unchanged at this stable checkpoint. Their historical references are
unchanged. The accepted design authorities remain in root at 0de87d56.

## First real source implementation — pending acceptance

The operator expanded scope to correct acquisition, implement one explicit-file
source and qualify it independently on Mac/Windows/Linux. The joint acquisition
design at root5dd59da1 passed Code and State after two bounded corrections; see
CredentialAcquisition-Verdict.md. That accepted design supersedes the earlier
portable synchronous selection seam. The historical portable acceptance above
remains evidence for its exact checkpoint, not the current changed source.

Current source implements one directly awaited controlled acquisition returning
PreparedCredential, separate source observations, shared admission/retained
cleanup, exact opaque source identity and an endpoint-local explicit file provider.
The file worker owns its native JoinHandle and result until actual join. Snapshot
bytes use zeroize1.9.0; bounded container decoding uses base64=0.22.1, the existing
core version. No dependency was upgraded. No new runtime or general supervisor.

Tests preserve the earlier lifecycle suite and add source caller-loss, completed
Future cancel/Drop/re-poll, original controls, private-ready fencing, retained
material/observation accounting, actual blocked/panicking native workers,
result-before-exit, file replacement, exact line-ending identity, native paths and
file/container refusal cases. Tests use temporary synthetic source data and fake
native authentication ports. Native SSH authentication and Connection placement/
reuse remain unimplemented separate gates. New source acceptance and native
results will be recorded at the settled implementation gate.

The settled production candidate is3e995edd. Native Mac/Windows/Linux gates
all pass; see CredentialPlatform-Results.md. Its separate Code+Surface/State
implementation review is pending. Measured candidate delta:17 production/package
files; approximately2,100 new/modified production lines excluding unit-test
modules (under2,400 ceiling), under1,800 new/modified test lines including unit
tests, and zero wire changes. Production and tests remain standalone.

Initial implementation review e4a080b3 returned NO-GO on both axes. They
independently found the same local P2 (native provider received later begin
deadline) and P3 (source observation lacked operation attribution), with zero
architectural roots. Correction1 normalizes the stored/forwarded deadline to
min(original source, begin) and exposes the existing read-only operation_id.
Both smallest regressions were observed red before their production correction,
then pass; see CredentialExplicitKey-RemPlan-1.md. The source/kernel controls
and retained cleanup remain unchanged. Native correction qualification and
originating review closure are pending.

Correction1 production/tests29b8838b pass fmt/test/clippy/doc natively on
MacARM64, WindowsMSVC/E:ReFS and LinuxPi/aarch64 using1.95.0. Source and
native evidence are settled for originating closure; no implementation GO is
claimed before both reviewers return their correction verdicts.
