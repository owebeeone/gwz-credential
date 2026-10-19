# Credential acquisition boundary amendment, round 2 — CODE-AXIS REVIEW

**Review object:** Whole affected Credential lifecycle and Connection seam under `dev-docs/GwzCredentialAcquisitionAmendment.md`, revision 2, at root `5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9`. Design gate only.

**Baseline:** Root `5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9`; portable reference member `50838c44f8faf4bc6eeb618370ee6ddc10cdd2fc`. Previous design root `dc7d6bbdd1611838badda78b9f891a74cc4d2106`; accepted pre-amendment authority `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`. Sources inspected through pinned Git objects; concurrent live-root advancement is excluded.

**Date:** 2026-10-19

**Axis:** Code, including Surface. Independent, adversarial and read-only. Prior reports and merged remediation were legitimate inputs; no current-round peer report was read. Filed verbatim by the lane owner.

**Verdict: GO** — both Code findings are closed; no new P0–P3 finding was established. This accepts the acquisition design boundary, not its implementation or native qualification.

**Architectural-root classification:** **NEW ARCHITECTURAL ROOT: NONE.** No architectural root has been established for this amendment.

---

## Prior-finding closure table

| Finding/input | Closure |
| --- | --- |
| Original Code P2-1: undefined AcquisitionClosing/report | **Closed and preserved.** AcquisitionClosing remains removed. Immediate AcquisitionEnd, its cleanup receipt/status and report Drop are explicitly defined. The stalled-source cancellation sequence remains complete. |
| Code P2-2 / prior State P2-1: completed-Future cancel omitted | **Closed.** Amendment lines 83–130 define truthful successful/error reports, completed cancel/Drop/observation/re-poll behavior and retained ownership. Lines 259–277 provide the borrowed-await witness and required completion traces. |
| Operator request for one directly awaited acquisition | **Preserved.** `acquire(...).await` yields PreparedCredential directly. No public wait, AcquiredCredential or select stage is restored. |

## Changed-range analysis

Read the revised public surface first, then the full amendment delta and affected lifecycle.

AcquisitionEnd now represents either outcome using `Option<AuthError>` and `material_published`. Cancel before transfer still cancels; cancel after successful transfer reports success observationally. Completed error retains its original cause. Explicit Unpin makes the borrowed-await example valid. Completed Drop releases only the completed view/observation pin, and re-poll returns WrongState without changing source facts or transferring twice.

The correction adds no lifecycle stage, dependency, supervisor or production source change. Provider acquisition, source controls, actual-peer authentication and retained native ownership are otherwise unchanged.

## 0. Evidence base

Read the canonical round-2 prompt, revised amendment, complete delta from the prior design, both prior-round reports and merged `CredentialAcquisition-RemPlan-2.md`.

Focused inspection covered:

- Public signatures and completed-state grammar: amendment lines 69–180.
- First-use, errors, cancellation, Drop and borrowed-await examples: lines 182–277.
- Source observation, accounting and retained cleanup: lines 279–320.
- Provider/native worker ownership: lines 322–364.
- Explicit-file trace, mandatory conformance cases and exact supersession: lines 366–434.

Earlier inspection of accepted Credential/Connection/boundary requirements and supporting core ownership code remains applicable. Those accepted authority documents have no new semantic delta. Member production source, tests, package files and README have no delta between the two portable references.

Both pinned commits resolved exactly and were verified as commit objects at start and end. No tracked member modification was reported; only authorized round-2 prompts were untracked. No current-round peer contents were read.

No builds, tests, writes, network operations or additional agents were used. Closure is by design trace, not executed implementation evidence.

## Problem, requirement provenance, and scope

The operator authorized fixing source acquisition before the first real explicit-file provider and requested a simpler directly awaited public surface. The revised design satisfies that preference while preserving internal acquisition, immutable selection and guarded publication as distinct phases.

The original necessity remains grounded in the controlled-stall source counterexample and existing reserve/work/join/publication route. Source identity must exist before explicit-key pool lookup; actual-peer authentication occurs later. Constructor preloading or delaying source acquisition until authentication would not satisfy those combined requirements.

The correction is confined to the authorized design boundary. No general supervisor, dependency upgrade, Connection implementation change or application migration is necessary. Native providers, production composition and actual SSH/placement/reuse qualification remain separate gates.

## 1. Findings

No open findings.

## 2. Invariant analysis

The original completed-Future counterexample now has an explicit consistent result:

1. Construct acquisition and retain its optional observation.
2. Borrow/manual-poll it to Ready(Ok(prepared)); source success and publication become fixed.
3. Cancel the still-existing completed Future.
4. Receive AcquisitionEnd with `reason = None`, `material_published = true` and the matching cleanup receipt.
5. PreparedCredential remains the sole transferred material authority; neither its authority nor native/material charges are revoked or released by this observational cancel.

The alternative completed paths are also closed:

| Path | Specified result |
| --- | --- |
| Success followed by completed-Future Drop | Only its completed view/observation pin is released; transferred prepared authority and source success remain unchanged. |
| Error followed by cancel | Original error and matching receipt are returned; publication stays false. Pre-admission refusal retains no receipt/record. |
| Error followed by Drop | No new terminal transition or replacement cause. |
| Re-poll after either completion | WrongState only; no second PreparedCredential, terminal rewrite or early resource release. Later cancel still observes the fixed original outcome. |

The original pre-transfer cancellation trace remains valid: a stalled read can be cancelled synchronously, its immediate report says Cancelled/unpublished with Pending cleanup, report Drop leaves retention intact, and actual worker release/join later confirms matching cleanup without late publication.

Whole-boundary attacks remain satisfied by the text:

- Admission precedes source effects; work progresses independently of caller polling.
- Ready-but-untransferred material remains private and races close/deadline under the publication guard.
- Captured native endpoint base and original caller remain authoritative; no ambient recapture or fallback is permitted.
- Every explicit-key lookup acquires fresh exact bytes; SourceIdentity remains equality rather than checkout authorization or a secret accessor.
- VerifiedRequest retains actual origin/peer/generation/challenge binding; BeforeLookup does not permit premature helper/native access.
- Acquisitions and authentication share the total record allowance and retainer; replacement owners cannot bypass stalled native work.
- Stored results do not prove native exit. The registered session retains its handle through actual joined disposal; matching material, observation and native pins keep their charges.
- Actual-peer authentication, key borrowing over NeedNetwork, physical disposal before key-pin release, independent trust/possession checks and retained-scope revocation remain unchanged.
- No fifth HostPort, new wire carrier, general signing capability or production integration is introduced.

## 3. Risks and next action

The later implementation gate must compile and exercise these completion/cancellation traces and qualify the scoped file worker, native filenames, regular-file admission, delayed exit, panic/error cleanup and last-pin accounting on supported platforms. This review does not claim those outcomes exist or pass.

**Next action:** merge both independent verdicts at this exact tuple. If both are GO, accept this acquisition amendment and proceed only with the already authorized bounded Credential API/explicit-file implementation and its separate qualification gates.
