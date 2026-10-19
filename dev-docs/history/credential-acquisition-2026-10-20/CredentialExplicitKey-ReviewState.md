# Controlled acquisition and explicit file-key source — STATE-AXIS REVIEW

**Review object:** Controlled acquisition API and first endpoint-local explicit SSH file-key source at `gwz-credential` `e4a080b3d99a4ab98478d0b06c8d0d104e4180a0`, controlled by `dev-docs/Implementation.md` and `CredentialExplicitKey-Plan.md`. Initial implementation gate, separate from accepted design.

**Baseline:** Root authority `a80515f364194b04e38c4a6d59597cee0d8347cc`; member `e4a080b3d99a4ab98478d0b06c8d0d104e4180a0`; production/test candidate `3e995edd53d315f639cbfe167fd4562778ee42b9`; private evidence `cd4acf0af1aa662356680bcb8a649c5b5e518e23`. Reviewed pinned Git objects and the member delta from `648aae24e6ba00e27b65a56314739a67ee1ccc33`. Concurrent root advancement excluded.

**Date:** 2026-10-20

**Axis:** State — lifecycle legality, races, deadlines/cancellation, retained native ownership, accounting, publication and fail-closed behavior. Independent, adversarial and read-only. No current implementation peer report was read. Filed verbatim by the lane owner.

**Verdict: NO-GO** — one P2 deadline-propagation defect blocks acceptance; one P3 observation-attribution omission is nonblocking. I pre-commit to GO on a revision resolving P2-1 as specified, absent new blocking evidence.

**NEW ARCHITECTURAL ROOT: NONE.** Both findings are local implementation omissions within the accepted design. This implementation gate has established zero architectural roots.

---

## 0. Evidence base

Inspected all new production files: acquisition API/state/driver/observation, explicit provider, file worker, native file admission and bounded container parser. Followed the changed existing API, authentication driver, state, futures, values and provider ports, together with their unchanged previously reviewed ownership/publication paths.

Read the acquisition, actual file-source and source-admission regressions; worker blocked-read, panic/join, stored-result/delayed-exit and lock-callback tests; migrated lifecycle, publication, scope, key-pin, observation, selection, bounds and seeded-schedule changes; test support; README; package/dependency/CI files; controlling implementation/plan and public platform results.

Checked accepted acquisition revision 2 and applicable Credential/Connection boundary requirements at the pinned root. Prior design GO is treated as design authority, not implementation acceptance.

Read the private evidence instructions, campaign README, three result JSON files and selected native gate log output. Those records report pinned Rust 1.95.0 test/fmt/clippy/doc success on Mac ARM64, Windows MSVC E: Healthy/ReFS and Linux Pi aarch64. Verified that production/tests/package/CI have no delta from the qualified `3e995edd` candidate to the reviewed member.

At start and end the member HEAD matched, tracked source was clean, and only the two authorized implementation prompts were untracked. Pinned root and evidence commits remained resolvable commit objects. No current-round peer report was read.

No builds, tests, writes, network operations, mutations or additional agents were performed. Native results are inspected recorded evidence, not independently rerun gates.

## Problem, requirement provenance, and scope

The operator authorized a standalone public library, first real explicit-file source and independent native source qualification, then explicitly corrected and simplified acquisition before implementation. The accepted design requires source bytes before every explicit-key lookup, retained bounded work before effects, inherited absolute controls and separate actual-peer authentication.

The implementation uses the existing ownership route rather than a new runtime: an admitted source record owns one file-session thread handle and private result, progressed by the supplied retainer. Pure selection/publication follows joined acquisition. This directly addresses the demonstrated inline-source stall.

The exact base64 0.22.1 dependency reuses the existing bounded container-decoding route; zeroize remains 1.9.0. No dependency upgrade, general supervisor, Connection port, wire protocol or application migration was introduced. Public tests and CI do not require private evidence or adjacent GWZ crates.

The findings require only member-local fixes and public fake regressions. In particular, P2-1 concerns the deadline passed through an existing portable provider port; it does not demand implementing a deferred native provider. Real SSH authentication, production Connection composition and endpoint placement remain separate gates.

## 1. Findings

### [P2-1] Authentication providers receive an extended deadline despite the retained source deadline

**Locations:** `src/api.rs:132–156`; `src/driver.rs:67–73`; `src/state.rs:225–239`; regression `tests/acquisition.rs:100–116`.

**Invariant:** Acquisition’s original absolute deadline must survive preparation and authentication; a later begin control cannot extend it. Checking publication against the old deadline is insufficient if the effectful provider receives a longer native deadline.

The implementation retains source controls and checks them in `Child::error`. However, `begin` accepts a later deadline unchanged, and the authentication driver passes `self.control.deadline_ms` directly to `SelectedCredential::start`, without taking the earlier source deadline into account.

Concrete trace:

1. Acquire successfully with source deadline 50.
2. Before time 50, call `prepared.begin(peer, AuthControl::new(1000))`.
3. Drive authentication creation. The provider receives deadline 1000.
4. A provider that fixes its native operation deadline from this argument consequently admits work under a longer deadline than the original request.
5. At time 50 the kernel can report Timeout and fence publication, but the native operation was already configured with the extended deadline. Later cancellation/cleanup cannot retroactively correct that admitted deadline.

The existing regression deliberately supplies 1000 after source deadline 50, then asserts a later response Timeout. It proves caller/publication fencing, not correct provider deadline propagation. The current explicit key session ignores this argument because actual SSH authentication is injected; that does not exempt the portable provider boundary from its accepted control contract.

**Impact:** A conforming provider using the supplied fixed deadline can retain or execute native work beyond the request’s original budget, despite the caller-visible timeout appearing correct.

**Correction:** Normalize the effective authentication deadline to the earlier source/exchange deadline, or refuse an extending control before effects. Ensure the actual value passed to provider start cannot extend the original source deadline. Preserve existing cancellation and cleanup ownership.

**Smallest regression:** A fake SelectedCredential records the deadline received by start. Acquire at deadline 50, begin with 1000, and assert either pre-effect refusal or a provider deadline of 50. Also test a tighter begin deadline and timeout/publication behavior. No native provider is needed.

**Classification:** Local inherited-control propagation defect. **NEW ARCHITECTURAL ROOT: NO.**

### [P3-1] Source observation omits the promised operation identity

**Locations:** `src/acquisition_observation.rs:32–38,49–68`; accepted acquisition amendment at root `:287–294`.

The accepted observation contract identifies provider namespace, operation and acquisition serial. The public `id()` exposes only namespace and acquisition serial. The operation ID is retained privately through SourceObserved.operation but is not observable.

For two operations with identical endpoint/target/source, a generic retainer receiving their WorkObservation values can distinguish acquisitions but cannot group them by owning operation from the declared observation alone. It must retain separate caller-side association metadata to obtain the promised attribution after caller loss. This is a bounded observation/diagnostic omission; no wrong cancellation or capacity release was established.

**Correction:** Expose the existing issuer-qualified operation identity alongside the acquisition identity, without adding authority or a new registry.

**Smallest regression:** Two acquisitions from one operation expose the same operation identity and distinct acquisition IDs; an acquisition from another otherwise identical operation exposes a different operation identity. Verify attribution using only public retained observations after ordinary caller views disappear.

**Classification:** Local observation-field omission. **NEW ARCHITECTURAL ROOT: NO.**

## 2. Invariant analysis

The following attacks held:

- Admission reserves snapshot, work, observation and a shared record before effects. Source/auth counts share the existing bound; replacement owners cannot bypass pending native work.
- Unpolled/Pending Drop and explicit cancel seal unpublished acquisition synchronously. Independent progress retains stalled work and rejects late material publication.
- Private success readiness remains distinct from guarded transfer. File material is exposed only after actual thread exit/join; a stored result alone cannot establish disposal.
- Completed Future success/error cancel, Drop and re-poll preserve fixed terminal facts. The Future drops its source-record reference after completion and retains observation rather than a second material owner.
- Provider calls, clock callbacks, equality and native work occur outside kernel authority/payload locks. Driving guards prevent concurrent stepping of one record.
- Native panic is joined before failure/disposal. Cleanup errors or uncertain work retain accounting; result and material disposal precede matching snapshot-charge release.
- Stable snapshot identity compares exact bytes and lineage. Changed files are reread for subsequent acquisition; ongoing key calls retain the original bytes over NeedNetwork and removal/replacement.
- Relative paths use captured absolute native base. No ambient cwd lookup, source fallback or lossy path substitution is used for opening. Qualified Unix opens fence FIFO replacement; Windows device/native-container policy is explicit.
- Bounded container decoding uses fixed scratch and periodic control checks. Framing acceptance does not claim cryptographic key validity; synthetic armor tests are labelled accordingly.
- Original key-fault, terminal-provenance, publication, transferred-scope and pending-disposal protections remain present. P2-1 identifies the narrower difference between kernel deadline enforcement and the provider’s admitted deadline.

The native evidence distinguishes actual Linux non-UTF8 creation/read from APFS refusal/no-lossy-fallback behavior, and source qualification from real SSH or Connection acceptance. No persistent mutation/restart protocol is introduced.

## 3. Risks and next action

An uninterruptible filesystem operation can retain a charged worker indefinitely. The implementation represents this as Pending and bounded refusal rather than invented disposal. Production retainer scheduling, actual native SSH and Connection composition remain explicit later obligations.

**Next action:** correct P2-1 and add the provider-argument regression in one bounded member patch, rerun the focused standalone/native gates and return the settled tuple for closure. P3-1 is nonblocking and can be corrected in the same authorized member scope.
