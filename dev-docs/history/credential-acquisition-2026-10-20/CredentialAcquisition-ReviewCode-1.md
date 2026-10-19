# Credential acquisition boundary amendment, round 1 — CODE-AXIS REVIEW

**Review object:** Whole affected Credential lifecycle and Connection seam under DRAFT `dev-docs/GwzCredentialAcquisitionAmendment.md`, revision 1, at root `dc7d6bbdd1611838badda78b9f891a74cc4d2106`.

**Baseline:** Root `dc7d6bbdd1611838badda78b9f891a74cc4d2106`; portable reference member `15b68ced3f123692662a634aba8c3b0450eaa129`. Prior design root `c9640db2881d6682cc06de58b4fa2464d0a4b706`; accepted pre-amendment authority `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`. Inspection used pinned Git objects; live root advancement is excluded.

**Date:** 2026-10-19

**Axis:** Code, including Surface. Independent, adversarial and read-only. Prior reports and merged correction plan were legitimate inputs; no current-round peer material was read. Filed verbatim by the lane owner.

**Verdict: NO-GO** — original P2-1 is closed; one new P2 public lifecycle edge remains. I pre-commit to GO on a revision resolving P2-2 as specified, absent new blocking evidence.

**Architectural-root classification:** **NEW ARCHITECTURAL ROOT: NONE.** P2-2 is a local completion of the direct Future’s method contract. No architectural root has been established for this amendment.

---

## Prior-finding closure table

| Input | Result and evidence |
| --- | --- |
| Code P2-1: undefined AcquisitionClosing report | **Closed.** Lines 79–116 declare immediate `AcquisitionEnd`, its failure reason, optional matching cleanup receipt and status. Lines 221–239 specify explicit cancel, report Drop, pending cleanup and admission refusal. |
| Operator objection to Result/handle.wait/Acquired/select ceremony | **Addressed.** Lines 69–107 define one custom Future with `Output = Result<PreparedCredential, AuthError>`. Internal immutable selection remains guarded, without extra public stages. |
| Whole-boundary preservation | **Preserved on inspected paths.** Source-before-lookup, verified-request binding, source/auth observation separation, shared limits, joined disposal and existing authentication/physical ownership remain explicit. The new post-completion edge is P2-2 below. |

## Changed-range analysis

Read revised section 3 cold before the requirements/plan and followed the complete changed range.

The revised surface directly expresses `context.acquire(...).await?`. Constructor refusal becomes a ready failed Future with no record/effect and no observation; registered work has an observation before effects and progresses independently of polling. Successful polling performs guarded internal selection and transfers PreparedCredential once. Cancellation returns an immediate defined report.

The examples now cover first use, source identity before lookup, actual-peer authentication, explicit cancellation, errors, Drop and operation/provider close. Original source control and VerifiedRequest binding are also stated more explicitly.

The change leaves one owned-Future case undefined: calling the consuming cancel method after manually observing successful `Poll::Ready`. The former separate wait consumed the acquiring handle; implementing Future on that handle removes this type-level restriction.

## 0. Evidence base

Read the canonical round-1 prompt, complete revised amendment (lines 1–396), full delta from the prior design, both prior reports and merged `CredentialAcquisition-RemPlan-1.md`.

Rechecked declared signatures and examples at amendment lines 69–239 against terminal/publication and retention rules at lines 241–282, provider ownership at lines 284–326, explicit-file trace at lines 328–361 and supersession at lines 363–396. The accepted authorities have no additional delta from the preceding review. Portable production source/tests/package/README also have no delta between the two member references.

Both pinned commits resolved exactly and were verified as commit objects at start and end. A concurrent uncommitted change to member `CredentialExplicitKey-Plan.md` appeared at the end; those bytes were not read and are excluded from the immutable object. Authorized prompts were also untracked.

No builds, tests, writes, network operations or additional agents were used. This is design review, not implementation or native-provider acceptance.

## Problem, requirement provenance, and scope

The operator authorized repairing the acquisition boundary before the first real file provider and expressly preferred a simpler directly awaited public surface. The revised Future removes public steps that were not needed to preserve the internal acquisition/selection/publication distinction.

The underlying necessity remains supported by the controlled-stall source probe and existing core reserve/work/join/publication route inspected in the original review. Acquisition cannot be delayed until actual-peer authentication without violating explicit-key prelookup timing. No larger supervisor, dependency upgrade, Connection source change or application migration is needed.

This correction stays within authorized design scope. P2-2 requires a local method-state rule and design trace, not another public staging chain or broader implementation.

## 1. Findings

### [P2-2] cancel’s contract excludes a completed Future that still exists after polling

**Locations:** Amendment lines 75–86, 109–116, 142–150 and 234–239.

**Invariant:** Successful publication is an immutable one-time transfer. Every callable public lifecycle method must define whether it remains valid after that transfer and must not imply that already-published material was never published.

`Future::poll` borrows the acquisition; `Poll::Ready(Ok(prepared))` does not consume the Future value. With the ordinary Unpin implementation allowed by this draft, a manual polling caller can safely retain that completed value and then call its declared `cancel(self)` method:

```rust,ignore
let mut acquiring = context.acquire(selection, purpose, control);
let observation = acquiring.observation();
// A manual executor polls Pin::new(&mut acquiring) until:
let prepared = /* Poll::Ready(Ok(prepared)) */;
let end = acquiring.cancel(); // method remains available on the retained value
```

Successful Ready has already fixed `material_published = true` and transferred the material pin. Yet lines 109–116 unconditionally define cancel as sealing a failure and assert that no material has been published from the owning Future. `AcquisitionEnd.reason` can express only an AuthError. There is no completed-success/WrongState rule, explicit precondition or specified panic behavior for this method.

The standard Future rule about polling again after completion does not specify the behavior of this separate public cancel method. The normal `.await` example hides the distinction because it consumes and drops the Future automatically.

**Impact:** Implementers must invent the post-transfer cancellation policy. Applying the documented cancellation transition could conflict with the immutable successful observation or wrongly revoke/describe a PreparedCredential already owned by the caller.

**Required correction:** Specify completed-Future behavior for cancel and Drop. A bounded option is an explicitly defined effect-free WrongState report after successful transfer, leaving source success, the transferred PreparedCredential and its pins unchanged; a documented invalid-state precondition/panic policy is another design choice. Qualify the assertion that cancellation implies no publication to the pre-transfer states where it is true. Preserve the first error after completed failure as well.

No return to public wait/Acquired/select stages is required.

**Smallest closure check:** Manually poll a movable acquisition to Ready(Ok(prepared)), retain the completed Future, invoke cancel according to the chosen rule, and prove that the success observation and transferred material remain unchanged. Repeat completed Err and completed-Future Drop. At this gate, specify the trace and expected results; the implementation gate should compile and exercise them.

**Classification:** Local public lifecycle specification defect. **NEW ARCHITECTURAL ROOT: NO.**

## 2. Invariant analysis

The original P2-1 counterexample now has a complete answer:

1. Construct acquisition; registered work supplies Some(observation).
2. Stall its source worker.
3. Call cancel and receive immediate AcquisitionEnd with Cancelled and matching Pending cleanup.
4. Drop the report; independent retention still owns the native handle and reservation.
5. Release/join the worker; the retained observation confirms matching cleanup, without late PreparedCredential publication.

Admission refusal is likewise specified: await returns Err with no record/effect, observation is None, and explicit cancel reports the original error plus Confirmed/no receipt. Successful transfer yields PreparedCredential directly; source identity exists before lookup, and actual-peer authentication remains a separate stage.

Other attacks held:

- Unpolled and Pending Drop synchronously seal unpublished acquisition; private ready results remain unpublished until guarded Ready transfer.
- Cancellation/deadline and provider/operation revocation are checked before transfer; prepared authority retains original request controls afterward.
- VerifiedRequest material remains bound to the actual supplied request/peer/generation; BeforeLookup is not permission for early helper/native contact.
- Source identity remains equality only, with exact bytes and lineage; every explicit-key lookup acquires anew and retains independent trust/authorization requirements.
- Source/auth records share the total allowance and retention service; owner replacement cannot bypass stalled native work.
- Native result readiness is separate from exit/join and disposal; output/material/observation pins retain their matching charges.
- No network port, wire format, secret getter, general signer or new cleanup framework is introduced.

## 3. Risks and next action

Implementation must still qualify the scoped file worker, native regular-file handling, panic/exit/join ordering and last-pin accounting on supported platforms. Those remain explicit later gates.

**Next action:** specify the completed-Future cancel/Drop behavior and its manual-poll trace, then return the revised immutable design for focused closure. Production implementation remains paused until both axes report GO at the same tuple.
