# Credential acquisition boundary amendment revision 2 — STATE-AXIS REVIEW

**Review object:** Whole affected Credential lifecycle and Connection seam, controlled by DRAFT `dev-docs/GwzCredentialAcquisitionAmendment.md` at root `5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9`.

**Baseline:** Previous root `dc7d6bbdd1611838badda78b9f891a74cc4d2106`; current root `5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9`; portable reference member `50838c44f8faf4bc6eeb618370ee6ddc10cdd2fc`. Accepted pre-amendment authority remains `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`. Inspection used pinned Git objects; concurrent root advancement excluded.

**Date:** 2026-10-19

**Axis:** State — lifecycle legality, cancellation/deadlines, retained ownership, bounded admission, publication and fail-closed recovery. Independent, adversarial and read-only. No current-round peer report was read. Filed verbatim by the lane owner.

**Verdict: GO** — State P2-1 is closed, prior closures are preserved, and no new P0–P3 finding was established. This accepts the design only, not production implementation or native/integration qualification.

**NEW ARCHITECTURAL ROOT: NONE.** No architectural root has been established in this amendment review.

---

## Prior-finding closure table

| Prior input | Result and evidence |
| --- | --- |
| Original Code P2-1: undefined acquisition closing/report | **Closed, preserved.** `cancel(self) -> AcquisitionEnd` remains immediate, with declared outcome fields, matching optional cleanup and report Drop behavior (amendment lines 79–130). |
| State P2-1 / Code P2-2: completed-Future cancellation undefined | **Closed.** Success and error completion now have explicit cancel, Drop, observation and re-poll rules. Success reports `reason=None, material_published=true`; error preserves `Some(original_error)`. Neither changes transferred ownership or fixed terminal facts (lines 117–130,259–277). |
| Operator request for one directly awaited acquisition | **Preserved.** The unique Send/Unpin Future yields PreparedCredential directly; no public wait/Acquired/select staging returns (lines 69–108). |
| Earlier whole-boundary State conclusions | **Preserved.** Admission, caller loss, exact-byte prelookup identity, absolute controls, source/auth separation and shared retained cleanup remain explicit. |

## Changed-range analysis

Read the complete revision-2 delta and affected lifecycle. `AcquisitionEnd` can now represent either fixed success or failure. Explicit Unpin makes the borrowed-await counterexample part of the declared supported surface.

The completed Future keeps fixed terminal/view state and an optional charged observation, not another material owner. Cancel after transfer moves observation into a truthful report without revoking the prepared credential or releasing its material/native charge. Re-poll returns WrongState without changing the original source outcome. Completed Drop adds no terminal transition.

Pre-transfer cancellation and ready-but-untransferred private results retain their earlier behavior. No public stage, runtime, supervisor, dependency or source implementation was added. The three accepted root API/boundary documents and member production source/tests/package files have no delta in this correction.

## 0. Evidence base

Read the canonical round-2 prompt, both legitimate round-1 reports, merged `CredentialAcquisition-RemPlan-2.md`, complete amendment delta and the revised public surface, examples, terminal table, worker ownership, conformance and supersession clauses through line 434.

Re-traced the original completed-Future counterexample and success/error cancel, Drop and re-poll variants. Rechecked their interaction with unchanged admission, publication, control, cleanup and capacity requirements.

The prior complete authority/source inspection remains applicable: accepted Credential §§4–5 and consumer traces; Connection–Credential boundary; Connection identity/reuse/timing requirements; portable source-admission counterexample; core ownership feasibility source at `461ddf4bc6892373a00b600da9778efb08ffc5d6`.

Both exact root/member commits resolved and remained commit objects at start and end. Root live HEAD was not used as authority. No current-round peer report was read.

No tests, builds, writes, network operations, mutations or additional agents were performed. These are design traces; no implemented or executed closure is claimed.

## Problem, requirement provenance, and scope

The operator authorized fixing the acquisition boundary before the real explicit-file provider, then requested fewer caller chores. The directly awaited Future satisfies that preference while retaining necessary internal acquisition, immutable selection and publication phases.

The concrete original problem remains the synchronous source seam’s inability to represent retained blocked acquisition before pool lookup. The inspected controlled-stall source and existing reserve/work/join route support the selected remedy. Delaying acquisition until authentication or constructor preloading would lose required timing or controls.

This correction resolves a local completed-state omission within the authorized design. It does not expand implementation into Connection, core, transports, Git, SSPI, CLI/Python or other providers. No dependency or infrastructure prerequisite is introduced. Native source implementation and integration remain separate authorized gates.

## 2. Invariant analysis

| Trace re-run | Result |
| --- | --- |
| Borrow-await success, retain the completed Future, then cancel | The original prepared credential remains the unique material authority. Cancel returns success/publication plus the matching source receipt; source observation remains unchanged. No native/material pin is released by that call (lines 122,125–130,261–271). |
| Completed success followed by Drop or re-poll | Drop releases only the completed view/observation pin. Re-poll returns WrongState and cannot transfer another credential or rewrite source success (lines 122,274–277). |
| Completed registered error followed by cancel, Drop or re-poll | Cancel reports the original error and matching receipt. Drop adds no transition. Re-poll reports WrongState separately from the fixed original source error; pending cleanup remains owned (lines 123,274–277). |
| Admission error followed by cancel, Drop or re-poll | No registration, source effect or observation is invented. Cancel preserves the original error with unpublished material and Confirmed/no receipt; subsequent polling cannot rewrite it (lines 105–106,123,255–257). |
| Cancel/Drop before first poll or while private success is ready | Acquisition is sealed before publication. Private readiness remains distinct from `Poll::Ready(Ok(prepared))`; late material is discarded and retained cleanup continues (lines 144–149,251–254,292–311). |
| Stalled read, explicit cancel/owner close, all callers lost | Immediate terminal failure and Pending cleanup remain observable. The session retains its handle, result and reservation until actual joined/probed disposal; report or channel loss is not proof (lines 302–320,342–364). |
| Deadline/external cancellation races transfer | Required periodic retainer progress observes controls without worker wakes; the publication guard checks again. Prepared authority preserves original request controls and verified-request binding after transfer (lines 158–169,302–308). |
| Replace owners or sustain retained material/observations | Source and auth share the combined finite record limit. Pending work and surviving pins retain matching shared charges; completed Future observation adds no second material owner or early release (lines 125–130,281–320). |
| Replace explicit-key bytes between lookups | Every lookup acquires anew; exact admitted bytes and required lineage determine compatibility. Identity equality remains separate from caller/policy, possession, fresh trust and live checkout authorization (lines 171–180,368–385). |

Captured native base/caller authority, source-specific verified-challenge timing, endpoint-local secret storage, actual-peer authentication, NeedNetwork pinning and connection-before-key-pin disposal remain unchanged. Source observation still invents no physical peer/generation and grants no selection or release authority.

No persistent mutation or restart protocol is introduced. Process loss cannot publish an unproved in-memory source result; restart requires fresh acquisition.

## 3. Risks and next action

Implementation still must prove the declared worker/result/exit/join ordering, regular-file and native-path handling, panic/error cleanup, completed-Future behavior and last-pin accounting. A permanently blocked read remains a truthful bounded Pending/refusal state. This design review supplies no native SSH or production bridge evidence.

**Next action:** merge the two verdicts at this exact tuple. If both are GO, accept revision 2 and proceed only with the authorized Credential implementation and its separate qualification gates.
