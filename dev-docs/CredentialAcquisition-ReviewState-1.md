# Credential acquisition boundary amendment revision 1 — STATE-AXIS REVIEW

**Review object:** Whole affected Credential lifecycle and Connection seam, controlled by DRAFT `dev-docs/GwzCredentialAcquisitionAmendment.md` at root `dc7d6bbdd1611838badda78b9f891a74cc4d2106`.

**Baseline:** Previous root `c9640db2881d6682cc06de58b4fa2464d0a4b706`; current root `dc7d6bbdd1611838badda78b9f891a74cc4d2106`; portable reference member `15b68ced3f123692662a634aba8c3b0450eaa129`. Pre-amendment authority remains `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`. Inspected pinned Git objects; concurrent root advancement excluded.

**Date:** 2026-10-19

**Axis:** State — lifecycle legality, races, cancellation/deadlines, retained ownership, bounded admission and fail-closed recovery. Independent, adversarial and read-only. No current-round peer report was read. Filed verbatim by the lane owner.

**Verdict: NO-GO** — one new local P2 contract omission remains in completed-Future cancellation. The prior closing/report omission is corrected. I pre-commit to GO on a revision resolving State P2-1 below, absent new blocking evidence.

**NEW ARCHITECTURAL ROOT: NONE.** This is a missing completed-state rule in the chosen public surface; it does not require another lifecycle stage or architecture change.

---

## Prior-finding closure table

| Prior input | Result |
| --- | --- |
| Original Code P2-1: undefined AcquisitionClosing/report | **Closed on the declared pre-transfer paths.** The undefined type is removed. `cancel(self) -> AcquisitionEnd`, its fields and `cleanup_status()` are declared, with immediate Pending/Confirmed behavior and report Drop semantics (amendment lines 79–116,221–239). |
| Operator request to remove caller wait/Acquired/select chores | **Addressed.** One directly awaited Future yields PreparedCredential; internal acquisition, immutable selection and publication remain distinct (lines 69–107,142–153). |
| Original State lifecycle/retention conclusions | **Preserved for admission, private readiness, transfer, caller loss, source cleanup, ownership and capacity.** The new public Future adds an externally retained completed state that needs the rule in State P2-1. |

## Changed-range analysis

Read the complete amendment delta and revision 1. The new constructor always returns CredentialAcquiring. Admission failure is a ready error with no registration or observation; admitted work has a source observation and progresses independently of polling.

Internal pure selection now atomically transfers PreparedCredential through `Poll::Ready(Ok(...))`. Source controls persist into that prepared authority; VerifiedRequest binding is expressly retained. Explicit cancel returns an immediate report, and periodic retainer progress observes external flags/deadlines without worker wakeups.

The three accepted root API/boundary documents have no further delta. Member production source, tests and dependency files are unchanged. The worker ownership and separate source/auth observation architecture therefore remain the earlier reviewed design, rather than an implemented correction.

## 0. Evidence base

Read the generated revision-1 State prompt, both legitimate prior reports, merged `CredentialAcquisition-RemPlan-1.md`, complete amendment lines 1–396 and the complete root-document delta. Re-traced the affected public signatures, first-use/error/cancel examples, publication guard, retained-job ownership and capacity clauses.

The earlier supporting inspection remains applicable: accepted Credential §§4–5 and consumer traces; complete boundary contract; Connection identity/reuse/timing clauses; portable source-admission counterexample; core source ownership at `461ddf4bc6892373a00b600da9778efb08ffc5d6`. No production source delta was found in the current member comparison.

Both specified immutable root/member commits resolved exactly and remained commit objects at start and end. Root live HEAD was not used as authority.

No builds, tests, writes, network operations, mutations or additional agents were performed. No implementation or native acceptance is claimed.

## Problem, requirement provenance, and scope

The operator authorized correcting the acquisition boundary and then explicitly requested fewer caller chores. Preserving the earlier Result/wait/Acquired/select spelling is therefore not a requirement.

The directly awaited acquisition still addresses the demonstrated synchronous-source limitation: source work needs bounded retained ownership before explicit-key pool lookup, while authentication requires actual verified peer facts later. Internal acquisition/selection phases remain necessary; separate public wrappers are not.

The proposed correction stays within the authorized Credential/documentation boundary and requires no dependency, supervisor, Connection port or application migration. Production remains paused. State P2-1 can be resolved by a small completed-state contract rule; it does not justify restoring the removed stages.

## 1. Findings

### [P2-1] Cancellation has no defined result after the Future has transferred success

**Locations:** Amendment lines 75–88,109–116,234–239; immutable source terminal contract at lines 243–250.

**Violated invariant:** The public lifecycle needs a closed terminal grammar. A successful guarded transfer must remain success, and a later operation on the still-existing completed Future must have a defined validity/result rule.

`Future::poll` borrows `Pin<&mut Self>`; returning PreparedCredential does not consume the Future object. The draft permits an ordinary Unpin implementation and does not restrict `cancel(self)` to the pre-completion state. A safe caller can therefore await a mutable borrow and retain the object:

```rust,ignore
let mut acquiring = context.acquire(selection, purpose, control);
let observation = acquiring.observation();
let prepared = (&mut acquiring).await?; // for an Unpin implementation
let end = acquiring.cancel();
```

At this point the contract has already fixed source success and `material_published=true`. However, cancel is specified to return an AuthError that is the first terminal failure, normally Cancelled, and states that no material has been published from that owning future. Neither statement describes this reachable completed state. No earlier terminal failure exists, and rewriting success would violate the immutable terminal slot. Returning WrongState or rejecting the method after completion could be reasonable, but neither behavior/precondition is declared.

This does not require every implementation to be Unpin: the issue is that the reviewed contract permits that implementation and leaves the resulting public state undefined. The standard prohibition on polling a completed Future again does not itself specify this separate cancel method.

**Impact:** Implementers must invent post-transfer cancellation behavior, potentially reporting a false cancellation or changing the lifetime of an already transferred prepared credential. The same public lifecycle is therefore not fully determined by the draft.

**Required correction:** Define completed-Future cancellation explicitly. A documented pre-completion-only operation with a defined invalid-use response, or a truthful completed-state report, is sufficient. Preserve the fixed source success and transferred material ownership. No extra public stage is necessary.

**Smallest closure check:** Trace manual/borrowed polling through `Poll::Ready(Ok(prepared))`, retain the completed Future, then attempt its declared cancel operation. Verify the specified response, unchanged successful source observation, no second transfer and no accidental pin/accounting release. Also state the completed-error behavior. Implementation tests follow at the separate implementation gate.

**Classification:** Local public-state specification defect. **NEW ARCHITECTURAL ROOT: NO.**

## 2. Invariant analysis

The following attacks still hold under the revised surface:

- **Admission failure:** one awaited Err, observation None, no source work/record. Explicit cancel on that ready failed admission returns the original error and Confirmed without inventing a receipt.
- **Before first poll:** registration and its observation already exist. Drop/cancel seals the acquisition; retainer ownership does not depend on initial caller polling.
- **Private success readiness:** worker readiness is not publication. Drop or close before guarded transfer discards the result; a stored result is not native disposal.
- **Pending explicit cancel:** immediate AcquisitionEnd carries the original failure and matching Pending receipt. Report Drop leaves the retained session responsible for join and cleanup.
- **External cancellation/deadline:** bounded periodic progress observes them independently of worker wakeups; every transfer checks them again under the publication guard.
- **Successful transfer:** PreparedCredential owns the same charged material and source authority. Later prepared/identity Drop releases only that pin; owner revocation prevents fresh use. State P2-1 concerns the residual completed Future object, not this intended transfer.
- **Source/native ownership:** the registered session retains its native handle and private result until actual joined disposal. Lost callers and replacement owners cannot release or bypass occupied shared capacity.
- **Sustained bounds:** acquisitions and authentication share the default 64-record allowance; successful material and observations retain separate charges through their last legitimate owners.
- **Identity and timing:** each explicit-key lookup requires fresh admitted exact bytes. VerifiedRequest results retain their actual request binding. Source equality remains separate from caller/policy, possession, trust and checkout authorization.
- **Placement and physical ownership:** native base/caller capture remains endpoint-local; no fallback or key export is added. Authentication still requires actual peer/generation and preserves key pins through native readiness and physical disposal.

No persistent mutation or restart grammar is introduced. Process loss cannot publish an in-memory unproved source success; a new process must acquire again.

## 3. Risks and next action

Real source qualification must still prove the specified thread/result/join ordering, regular-file handling, native paths, delayed exit, panic/error cleanup and last-pin accounting. Portable/design evidence does not establish those outcomes.

**Next action:** define the completed-Future cancel state in this document, then return the pinned correction for focused closure. Keep production paused until both axes report GO at the same tuple.
