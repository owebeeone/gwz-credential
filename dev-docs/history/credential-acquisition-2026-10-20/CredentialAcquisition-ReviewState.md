# Credential acquisition boundary amendment — STATE-AXIS REVIEW

**Review object:** Whole affected Credential lifecycle and Connection seam, controlled by DRAFT `dev-docs/GwzCredentialAcquisitionAmendment.md` at root `c9640db2881d6682cc06de58b4fa2464d0a4b706`.

**Baseline:** Root `c9640db2881d6682cc06de58b4fa2464d0a4b706`; portable reference member `f6df6be69d54927b3eed6140aa59f92bb2ab13a4`; pre-amendment authority `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`. Sources read through pinned `git show` objects. Concurrent root advancement excluded.

**Date:** 2026-10-19

**Axis:** State — lifecycle legality, races, cancellation/deadlines, retained ownership, bounded admission and fail-closed recovery. Independent, adversarial and read-only. Nothing here relies on the parallel Code review. Filed verbatim by the lane owner.

**Verdict: GO** — no P0–P3 finding established. This accepts the proposed design boundary only; it is not implementation, native-provider or Connection-integration acceptance.

**NEW ARCHITECTURAL ROOT: NONE.**

---

## 0. Evidence base

Read:

- The canonical State prompt and review-loop skill.
- The complete amendment, lines 1–287.
- The affected Credential public ownership, key-use, terminal, retention and consumer clauses, particularly §§4–5 and 7.2–7.4.
- The complete Connection–Credential boundary contract and Connection §§7–7.3, including explicit-key prelookup identity and retained-scope rules.
- The comparison against pre-amendment root authorities: the three accepted documents acquire draft pointers; the amendment explicitly defines their prospective supersession.
- Portable `tests/source_admission.rs:1–84`, `src/provider.rs:15–67` and `src/api.rs:119–166` at the reference member.
- Supporting core source at `461ddf4bc6892373a00b600da9778efb08ffc5d6`: `ssh_key_snapshot.rs:1–215`, `agent_job.rs:52–159,251–357`, and `regular_file.rs:1–172`.
- Applicable pinned workspace/process instructions.

At start and end, both exact review commits resolved and were verified as commit objects. Root live HEAD was not used as authority. No current-round peer report was read.

No tests, builds, writes, network operations, mutations or additional agents were performed. The source-admission probe was inspected, not independently executed. Its deliberately nonconforming provider demonstrates the synchronous seam’s limitation; it does not refute the existing kernel’s conforming-provider contract.

## Problem, requirement provenance, and scope

The operator explicitly authorized fixing the underspecified acquisition boundary before implementing the first real explicit-file provider. Existing requirements already demand fresh exact-byte identity before every explicit-key pool lookup, prompt public calls, inherited absolute controls and retained cleanup after caller loss.

The current synchronous selection path cannot express an independently retained blocked source read before a physical peer exists. The inspected barrier probe makes that limitation concrete. Moving the read into authentication would occur after lookup and require peer facts too early; constructor preloading would bypass per-request controls. The amendment’s distinct acquisition stage addresses this problem directly.

The proposal keeps selection pure, preserves actual-peer authentication, uses the existing retention service and narrows the file execution route to an owned native thread/session. It does not require importing core, creating a general supervisor, upgrading dependencies or changing Connection’s host ports.

Root acquisition documentation and the eventual bounded Credential implementation are authorized components. Core, Connection implementation, transports, Git, SSPI, CLI/Python and existing supervisors remain reference-only. Deferred production integration, other source implementations and real SSH qualification are legitimate later gates. No unsupported prerequisite or unauthorized expansion was found.

## 2. Invariant analysis

| Concrete attack | Result and controlling text |
| --- | --- |
| Saturate admission, then attempt another source open | Snapshot/work maxima, observation and retained record are reserved before source effects. Refusal is synchronous and effect-free; the retainer alone starts work (amendment lines 101–106,221–225). |
| Cancel or close while an OS read is stalled; discard every public future | Terminal failure becomes observable immediately; the same registered owner retains work, slot and budget until actual completion/join. A stalled process-lifetime read can cause bounded refusal, not replacement capacity (lines 148–173,195–215). |
| Worker stores success but has not exited | A stored result is expressly insufficient disposal evidence. Join/probe precedes cleanup confirmation, and the explicit-file trace requires joined admission before publication (lines 162–164,195–211,229–231). |
| Result becomes ready, then cancellation/deadline/owner close wins before transfer | Ready is private and terminal success has not yet been published. Transfer shares provider/operation revocation arbitration; late material is discarded/zeroized and the first reason remains fixed (lines 150–164). |
| Successful result remains unconsumed and acquiring/wait is dropped | Drop synchronously seals that acquisition; independent cleanup continues. No authentication or late selection capability can emerge (lines 152–164). |
| Transfer wins, then owner closes while identity/material pins remain | The immutable success fact is preserved, revocation is separate, new selection requires the exact live owning provider/operation, and storage remains charged through its final legitimate pin (lines 115–144,151–156). |
| Replace the owner while old source jobs remain pending | Source and authentication records share the existing total allowance; the default is 64 combined records. Pending work remains charged against shared retention across replacement owners (lines 166–173). |
| Replace key path contents between successive lookups | Every lookup acquires anew. Equality compares admitted bytes and required lineage, not path or a digest hint; a subsequent path change cannot mutate the earlier snapshot (lines 124–133,221–238). |
| Reuse a matching identity as permission to publish or checkout | SourceIdentity is equality only. Caller/policy, possession, fresh host trust and the existing live reuse grant remain independent requirements; transferred BoundScope is not fresh source acquisition (lines 61–67,124–133 and retained boundary §6). |
| Acquire helper/native material before its verified challenge | BeforeLookup is explicitly restricted by source policy; VerifiedRequest carries actual verified facts and the required bounded challenge. The amendment does not authorize early helper/native contact (lines 91–99). |
| Resolve a relative key path on the worker’s ambient cwd/caller | Explicit captured endpoint base and original caller govern resolution. Worker recapture, ambient cwd lookup, endpoint fallback and lossy path conversion are prohibited (lines 108–113). |
| Re-enter explicit-key native authentication after readiness or cancellation | The existing stable snapshot borrow, actual peer/generation, admitted key pin and connection-before-pin disposal obligations remain unchanged (lines 232–238; Credential §4.1; boundary §5). |

The immediate `Result` has a concrete role: `Err` means no registered acquisition/effect; `Ok` transfers the unique registered acquisition handle with observation and cancellation rights. Work progresses independently of `wait`. Calling `async wait(self)` moves that ownership into its returned future, including before its first poll; dropping that future must perform the acquiring-stage Drop transition.

Private provider readiness is distinct from `Poll::Ready` returning `AcquiredCredential`: the former remains unpublished in the retained record, while the latter is the guarded transfer and fixes `material_published=true`. Dropping a returned `AcquiredCredential` releases its pin rather than rewriting success. `AcquisitionClosing` abandons observation on Drop and does not become the cleanup owner. These ownership distinctions justify a registered handle and an asynchronous transfer; whether the handle also implements `Future` is not required for correctness.

The file-worker projection has a concrete existing ownership route: reserve first, retain a private result and JoinHandle, distinguish result readiness from joined completion, and release on the matching proof. The amendment requires nonblocking completion inspection and no additional reaper registry. Existing source is feasibility evidence; its naming alone does not qualify a new implementation.

No persistent mutation or restart protocol is introduced. Source work reads into process-local owned material; a process crash cannot create an accepted snapshot or reusable authentication receipt. Restart requires fresh acquisition rather than recovery of an unproved success. No filesystem journal or broader durable framework is necessary for this design.

## 3. Risks and next action

A blocked filesystem operation may remain uninterruptible indefinitely. The contract represents that state truthfully as terminal failure plus retained Pending cleanup and occupied capacity. Implementation must prove the specified worker, join, publication and last-pin accounting behavior through the permanent traces in amendment lines 240–252; this review does not claim those tests exist or pass.

**Next action:** merge the two design verdicts at the exact tuple. If both are GO, accept this acquisition amendment and proceed only with the authorized Credential API/explicit-file implementation and its separate qualification gates.
