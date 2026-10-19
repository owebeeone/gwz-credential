# Controlled acquisition and explicit file-key source correction 1 — STATE-AXIS REVIEW

**Review object:** Controlled acquisition API and endpoint-local explicit SSH file-key source at `gwz-credential` `0d01d6af401d7ef5e0da09d5180b01098eb09a72`, controlled by `dev-docs/Implementation.md` and `CredentialExplicitKey-Plan.md`. Implementation correction 1, separate from design acceptance.

**Baseline:** Previous member `e4a080b3d99a4ab98478d0b06c8d0d104e4180a0`; current member `0d01d6af401d7ef5e0da09d5180b01098eb09a72`; corrected production/tests `29b8838b9fb3bf60cda58ab259e7d85039a2da88`; immutable root authority `a80515f364194b04e38c4a6d59597cee0d8347cc`; private evidence `a3f9a37d7f0a9805ed4abb5af253466766c8aced`. Sources read through pinned Git objects; concurrent root advancement excluded.

**Date:** 2026-10-20

**Axis:** State — lifecycle legality, cancellation/deadlines, retained native ownership, accounting, publication and fail-closed behavior. Independent, adversarial and read-only. No current-round peer report was read. Filed verbatim by the lane owner.

**Verdict: GO** — original P2-1 and P3-1 are closed. No new P0–P3 finding was established.

**NEW ARCHITECTURAL ROOT: NONE.** Cumulative architectural-root count for this implementation gate remains zero.

---

## Prior-finding closure table

| Original finding | Result and evidence |
| --- | --- |
| P2-1: authentication provider receives extended begin deadline | **Closed.** `src/api.rs:132–162` normalizes control to `min(begin, source)` before admission and storage in Child. `src/driver.rs:67–73` therefore forwards the effective cutoff. `tests/acquisition.rs:212–245` records the actual provider argument for both deadline orders, checks time-50 refusal, Pending cleanup and eventual balanced release. |
| P3-1: source observation omits operation identity | **Closed.** `src/acquisition_observation.rs:54–57` exposes the existing namespace-qualified operation ID without authority or allocation. `tests/acquisition.rs:247–289` obtains retainer-only A1/A2/B1 observations, verifies grouping and distinct acquisition IDs, then verifies stable attribution after caller loss and cleanup. |

Both original counterexamples were re-traced on the corrected source. Recorded red evidence shows the former provider argument `[1000]` instead of `[50]` and the missing `operation_id` compile failure; corrected native logs show both regressions passing.

## Changed-range analysis

The complete correction changes two production files: deadline normalization in authentication registration and a read-only observation accessor. The test delta adds actual provider-argument capture and the two focused regressions.

The normalized control is stored in the existing Child and reaches the existing provider start path unchanged. Both original source cancellation and begin cancellation checks remain present. A tighter begin deadline remains tighter; neither control order extends the native cutoff. No new clock callback or lock acquisition was added.

Operation attribution reads immutable existing namespace/operation fields. It creates no registry, material copy, owner or lifecycle permission and does not change observation charging or destruction.

The file worker, parser, acquisition publication/completion state machine, dependencies and CI have no correction delta. Production/tests/package/CI are identical between qualified candidate `29b8838b` and the reviewed member.

## 0. Evidence base

Read the canonical correction prompt, both legitimate original implementation reports, merged `CredentialExplicitKey-RemPlan-1.md`, the complete correction diff, affected registration/provider forwarding paths, observation implementation and complete new regressions/support changes.

The prior complete implementation inspection remains applicable to unchanged source: acquisition registration/state/driver, completed-Future behavior, explicit-file worker/join ownership, native regular-file admission, bounded container checks, authentication/publication, scope and key-pin cleanup, accounting and standalone tests. Accepted root requirements and scope remain unchanged.

Inspected the scoped private correction run README, red regression logs, Mac result metadata, Windows/Linux result files and relevant native gate log output. They record pinned Rust 1.95.0 test/fmt/clippy/doc PASS on Mac ARM64, Windows MSVC E: Healthy/ReFS and Linux Pi aarch64 at `29b8838b`. Both new regressions appear passing on all three platforms.

At start and end:

- Member HEAD matched `0d01d6af401d7ef5e0da09d5180b01098eb09a72`.
- Tracked source was clean; only the two authorized correction prompts were untracked.
- Root authority and evidence pins resolved exactly and remained commit objects.

No builds, tests, writes, network operations, mutations or additional agents were performed. Qualification results are inspected recorded evidence, not independently rerun gates. No current-round peer report was read.

## Problem, requirement provenance, and scope

The operator authorized the standalone Credential library, first explicit-file source and independent source qualification after the accepted acquisition redesign. Original deadlines must survive into effectful provider work; retained source observations must expose their promised operation attribution.

The correction directly implements those existing requirements. It stays within the member API, observation and focused-test boundary. No accepted contract amendment, dependency change, supervisor/runtime, Connection port or application migration was introduced. Core, Connection, transports, Git, SSPI, CLI/Python and existing supervisors remain reference-only.

The existing bounded decoder/dependency route remains unchanged. No new infrastructure prerequisite is asserted. Native SSH authentication, production retention composition and endpoint placement/reuse remain separate gates, accurately distinguished from these source/library results.

## 2. Invariant analysis

The original deadline trace now produces provider cutoff 50 for source50/begin1000. Reversing the values also produces 50, preserving the stricter begin budget. At time 50 the existing error guard rejects further response work. Disposal remains Pending until the provider confirms; dropping the completed observation and operation then balances usage. This closes the difference between caller-visible timeout and the cutoff admitted to provider work.

The original attribution trace now groups A1 and A2 by the same issuer-qualified operation ID and distinguishes B1 despite identical endpoint/target/source inputs. Acquisition IDs remain separate. Dropping acquiring handles and operation owners cannot change those immutable values; cleanup reaches Confirmed through the retainer and final observation release balances usage.

The narrow patch preserves the other reviewed protections:

- Pre-effect shared reservations, combined source/auth record limits and replacement-owner refusal.
- Independent periodic retention progress after caller/owner loss.
- Private readiness distinct from guarded one-time transfer and actual joined native disposal.
- Completed Future cancel/Drop/re-poll preserving terminal facts and unique prepared ownership.
- Provider/clock/equality callbacks outside kernel authority locks.
- Snapshot, native-key and observation pins retaining matching charges through actual disposal or final ownership release.
- Fresh exact-byte acquisition before explicit-key lookup, verified-request binding, captured native paths, no fallback and no key export.
- Existing authentication, output, transferred-scope and connection-before-key-pin ownership requirements.

No new persistent mutation or restart grammar is introduced. No new blocking evidence was found in the affected paths.

## 3. Risks and next action

An uninterruptible filesystem call can still retain a charged native worker indefinitely; Pending and bounded refusal remain the truthful outcomes. Real SSH cryptography/authentication, production scheduler/Connection composition and endpoint placement require their separately authorized qualification.

**Next action:** merge the two implementation verdicts at this exact tuple. If both are GO, accept the bounded acquisition/explicit-file source implementation and record the source/library qualification scope. No further State-axis correction is required.
