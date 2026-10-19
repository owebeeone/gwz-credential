# CredentialExplicitKey — CODE-AXIS REVIEW — CORRECTION 1

**Review object:** Controlled acquisition API and endpoint-local explicit SSH file-key source at `gwz-credential 0d01d6af401d7ef5e0da09d5180b01098eb09a72`. Production/test candidate: `29b8838b9fb3bf60cda58ab259e7d85039a2da88`. Originating implementation correction-1 closure.
**Baseline:** Initial reviewed member `e4a080b3d99a4ab98478d0b06c8d0d104e4180a0`; original implementation baseline `648aae24e6ba00e27b65a56314739a67ee1ccc33`. Root authority remains `a80515f364194b04e38c4a6d59597cee0d8347cc`; private evidence is `a3f9a37d7f0a9805ed4abb5af253466766c8aced`. Read pinned Git objects; concurrent root HEAD is excluded.
**Date:** 2026-10-20
**Axis:** Code, including Surface. Independent, adversarial, read-only. Prior reports and merged remediation are legitimate inputs; no current-round peer report was read. Filed verbatim by the lane owner.

**Verdict: GO** — originating P2-1 and P3-1 are closed. No new P0–P3 findings. This accepts the bounded source/library implementation only.

**NEW ARCHITECTURAL ROOT: NONE.** The correction implements the existing accepted contract locally. Architectural-root count for this implementation object remains zero; this is correction 1 of the bounded implementation gate, separate from completed design remediation.

---

## Prior-finding closure table

| Finding | Original counterexample retraced | Correction and regression evidence | Disposition |
| --- | --- | --- | --- |
| P2-1: provider receives an extended begin deadline | Acquire with deadline 50, begin with 1000, then inspect the actual argument to `SelectedCredential::start`. Previously it received 1000 despite kernel Timeout at 50. | `src/api.rs:138–145` clamps control to the earlier source/begin deadline before admission. Child stores that control at line 222; unchanged `src/driver.rs:67–73` forwards it. `tests/acquisition.rs:213–245` verifies both deadline orderings produce 50, refusal at time 50, Pending cleanup before disposal, Confirmed afterward and eventual zero usage. | **Closed.** Recorded red assertion was `[1000]` versus `[50]`; both cases pass in the corrected native logs. |
| P3-1: source observations cannot identify their operation | Obtain retainer-only A1/A2/B1 observations for two otherwise identical operations, then lose the public callers and operation owners. Previously only acquisition identity was exposed. | `src/acquisition_observation.rs:54–57` returns the existing namespace-qualified operation ID without adding authority/storage. `tests/acquisition.rs:248–289` verifies grouping A1/A2 versus B1, distinct acquisition IDs, stable attribution after caller loss/cleanup and final accounting release. | **Closed.** Recorded red output reports the missing accessor; corrected tests pass on all three native platforms. |

## Changed-range analysis

Reviewed the full `e4a080b3..0d01d6af` correction. Production changes are confined to deadline normalization in `src/api.rs` and a read-only accessor in `src/acquisition_observation.rs`. Test changes add the two public-boundary regressions and instrument fake provider start deadlines/retainer observations. Remaining changes file the original reports/prompts, merged plan and qualification/checkpoint updates.

The minimum calculation uses existing absolute timestamps, introduces no clock conversion or arithmetic overflow, and preserves checks of both cancellation controls. It applies before retained authentication admission and provider effects. The accessor reads immutable existing fields through the observation's retained Operation; it allocates nothing and adds no cancellation, publication or release capability. Existing acquisition identity remains unchanged.

There are no worker, parser, dependency, CI, wire or source-provider implementation changes. Production/tests/package/CI are identical between qualified candidate `29b8838b` and reviewed member `0d01d6af`. The changed call path and Surface were reassessed; unchanged source ownership and publication proofs remain applicable.

## 0. Evidence base

- Read the complete canonical correction prompt. Revisited the unchanged public README/API before the correction plan, then read both original reports and `CredentialExplicitKey-RemPlan-1.md`.
- Inspected the entire correction diff, current `src/api.rs:1–265`, `acquisition_observation.rs:28–103`, affected provider invocation in `driver.rs:26–92`, inherited control checks in `state.rs:214–241`, new tests at `tests/acquisition.rs:212–289`, and all fake-provider/retainer support changes.
- Retained the initial whole-implementation review of acquisition, native file worker, file admission, bounded container decoder, material/observation accounting, authentication lifecycle and standalone tests. The correction diff confirms those other production paths are unchanged. Accepted Acquisition, Credential, Connection and boundary authorities remain at the same pinned root.
- Read corrected public implementation/platform summaries and private evidence instructions. Inspected the private correction-run README, both red logs, Mac result JSON/metadata/test output, Windows/Linux command snapshots and result JSON, and relevant native gate output at `campaigns/credential-qualification/runs/2026-10-20-explicit-source-correction1`.
- Evidence records both named regressions passing and ordinary fmt/test/clippy with `-D warnings`/doc success on pinned Rust 1.95.0: Mac ARM64, Windows MSVC E: Healthy/ReFS and Linux Pi aarch64. Windows/Linux logs identify exact source `29b8838b`; ordinary command sequences exit zero. These are inspected recorded results, not gates independently rerun by this reviewer.
- `git diff --check` passed for the correction and full implementation range. The source/tests/package/CI diff from the qualified production candidate to the reviewed member is empty.
- Verified root authority, member and evidence pinned objects at start and end. Member HEAD remained exactly `0d01d6af401d7ef5e0da09d5180b01098eb09a72`; tracked state remained clean, with only the two declared current generated prompts untracked. No discrepancy occurred.
- No builds, tests, writes, network operations, Git mutations or additional agents were performed.

## Problem, requirement provenance, and scope

The operator authorized a public standalone Credential library, first real explicit file-key source and independent native qualification, then expressly required correction and simplification of acquisition before implementation. The accepted lifecycle requires controlled source work before explicit-key lookup, retained ownership before effects, immutable material transfer, inherited absolute controls and separate actual-peer authentication.

This patch addresses the demonstrated provider-cutoff defect and observation-attribution omission using the existing supported boundaries. Neither remedy depends on a new runtime, supervisor, registry, native provider or dependency. The exact existing `base64=0.22.1` pin and unchanged zeroize dependency remain justified by the narrowly reused bounded decoding/storage route inspected during the initial review.

Authorized editable components remain this member's library/provider/tests/CI/docs and scoped qualification evidence. Core, Connection, transport, Git, SSPI, CLI, Python and existing supervisors remain reference-only. The correction stays within the authorized member API/observation/test boundary. No unsupported prerequisite or unauthorized component change was established.

Other source providers, native SSH cryptography/authentication, production retention composition, endpoint placement/reuse and application integration remain separately authorized outcomes. Their absence does not block this bounded gate, and the current documents do not claim those outcomes passed.

## 1. Findings

No new findings. Both originating findings are closed as recorded above.

## 2. Invariant analysis

**Surface.** Acquisition remains one directly awaited `CredentialAcquiring` Future yielding `Result<PreparedCredential, AuthError>`, with optional observation and immediate consuming cancel. No extra wait/Acquired/select stage was introduced. The additive operation accessor completes the promised attribution without changing acquisition IDs or lifecycle rights. README construction, required controls, defaults and cleanup obligations remain consistent with the corrected implementation.

**Controls and provider boundary.** For source50/begin1000 and source1000/begin50, Child now stores deadline50 and the actual provider call receives50. Already-expired controls still fail before provider effects. The original source and begin cancellation authorities remain checked. At cutoff, output/progression refusal does not release retained native work; the regression holds cleanup Pending until the provider confirms disposal.

**Acquisition completion and ownership.** Unchanged guarded transfer still requires live provider/operation/source control and confirmed source cleanup. Failed admission produces no source effects. Pending/unconsumed Drop seals unpublished work; successful manual/borrowed polling transfers once. Completed cancel/Drop/re-poll preserve terminal facts and do not revoke or duplicate the transferred material. The correction changes none of those states or ownership relationships.

**Retention and material.** Source/authentication records still share pre-effect admission and the existing finite allowance. Private result readiness remains separate from native exit/join and public transfer. Snapshot, identity, authentication/native pins and observations retain their respective charges until the final legitimate owner disappears. Zeroization precedes matching material-accounting release. The new accessor adds no surviving allocation. Replacement owners still cannot bypass pending shared work.

**Source and authentication separation.** Captured caller/policy/native-base inputs, fresh exact explicit-key bytes before lookup, opaque equality without checkout authority, VerifiedRequest peer binding and actual-peer authentication remain intact. Native key use retains the admitted snapshot across NeedNetwork without reopening the path. Provider completion remains distinct from remote acceptance.

**Native/source and test isolation.** Worker ownership, fixed charged stack/buffer, regular-file/native-path handling, refusal policy and bounded decoding are unchanged. The full native source/library gates include those existing tests. APFS native refusal and Linux actual nonUTF8 creation/read remain separately described; framing tests do not establish cryptographic validity. Product tests retain no private-evidence or adjacent-core dependency.

## 3. Risks and next action

The external retainer must continue bounded periodic progress and preserve pending registrations through shutdown. An uninterruptible filesystem operation can retain its charged worker indefinitely; Pending remains an honest cleanup result. Real SSH/native authentication and Connection/application composition remain separate qualification gates.

The Code+Surface implementation gate is GO at the exact reviewed tuple. The next action is the lane owner's merge with the other independent verdict at this same tuple, followed by scope-accurate acceptance recording if both are GO.
