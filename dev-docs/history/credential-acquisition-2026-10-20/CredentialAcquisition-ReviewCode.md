# Credential acquisition boundary amendment — CODE-AXIS REVIEW

**Review object:** Whole affected Credential lifecycle and Connection seam, controlled by `dev-docs/GwzCredentialAcquisitionAmendment.md` at root `c9640db2881d6682cc06de58b4fa2464d0a4b706`. DRAFT design gate; production implementation is paused.

**Baseline:** Root `c9640db2881d6682cc06de58b4fa2464d0a4b706`; portable reference `gwz-credential` `f6df6be69d54927b3eed6140aa59f92bb2ab13a4`; pre-amendment authority `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`. Supporting core source: `461ddf4bc6892373a00b600da9778efb08ffc5d6`. Inspection used pinned Git objects; concurrent live-root advancement is excluded.

**Date:** 2026-10-19

**Axis:** Code, with Surface folded in: architecture, public lifecycle, compatibility and provider/Connection handoffs. Independent, adversarial and read-only. No current-round peer report was read. Filed verbatim by the lane owner.

**Verdict: NO-GO** — one P2 Surface finding blocks the otherwise coherent acquisition design. I pre-commit to GO on a revision resolving P2-1 as specified, absent new blocking evidence.

**Architectural-root classification:** **NEW ARCHITECTURAL ROOT: NONE.** The finding is a local omission in the new closing/report contract, not a failure of the selected acquisition architecture.

---

## 0. Evidence base

Read the portable README/public lifecycle first, then its acquisition-relevant public API/provider interfaces. Read the complete amendment, the affected accepted Credential and boundary clauses, and Connection’s identity, reuse, timing and retention obligations. Compared the three authority documents against `0de87d56`: their only changes are DRAFT pointers, so earlier acceptance is not silently rewritten.

Inspected:

- Amendment `GwzCredentialAcquisitionAmendment.md:1–287`.
- Portable `README.md:1–103`, `src/api.rs` admission/selection/begin surface, `src/provider.rs`, and `tests/source_admission.rs:1–84`.
- Member admission-question and explicit-key task documents.
- Core `ssh_key_snapshot.rs:1–226`, `agent_job.rs:52–158,251–357`, and `regular_file.rs:1–172` at the recorded reference revision.
- Applicable workspace/process instructions and review-loop skill.

Both specified immutable root/member commits existed and resolved exactly at start and end. No tracked member modification was reported. Authorized review artifacts were the only untracked member files observed; their peer-report contents were not read.

No builds, tests, writes, network operations or additional agents were used. The controlled-stall probe was inspected, not rerun. Native/source implementation acceptance is not claimed.

## Problem, requirement provenance, and scope

The operator requested the first real explicit-file source, independent platform qualification and public hosting, then expressly directed repair of the underspecified acquisition boundary. Preserving the old synchronous selection signature is therefore not a requirement.

The concrete limitation is established narrowly: the current inline provider selection can be held behind a source barrier without a retained source-work record, controls or observation. The probe deliberately violates the current prompt-return provider obligation; it does not prove that conforming portable providers are defective.

Existing core code supplies a useful feasibility witness: reserve before source work, retain the native job handle, publish admitted exact bytes only after joined completion and a live-request check, and refuse special-file sources. Moving acquisition to peer-authentication begin would occur too late for explicit-key pool lookup; uncharged constructor preloading would lose per-request admission/control. A distinct acquisition stage is consequently justified.

The proposed narrow worker projects existing ownership mechanics into a session held by the same injected retainer. It does not require importing core or creating a general supervisor. The current design-only gate and future Credential-member source work fit the stated authorization; Connection/core/transport/Git/SSPI/CLI/Python implementation remains reference-only. No dependency upgrade or broader migration is demonstrated as necessary.

## 1. Findings

### [P2-1] AcquisitionClosing has no declared observation operation or result contract

**Locations:** Amendment `:69–89`, particularly `cancel(self) -> AcquisitionClosing` at line 80; lifecycle table `:146–156` and close coverage `:166–173`.

**Violated invariant:** Every public owned lifecycle stage must have a complete, usable close/report pair. Acquisition cancellation must report its terminal source decision separately from pending native disposal, without requiring callers or implementers to invent a public operation.

The draft declares that `CredentialAcquiring::wait` is an async function returning `Result<AcquiredCredential, AuthError>`. It does **not** declare that `CredentialAcquiring` itself implements Future. That distinction is clear and implementable.

The new `AcquisitionClosing` is different: its sole public appearance is as the return type of `cancel`. The prose says cancellation reports immediately with Pending where needed and says dropping AcquisitionClosing abandons observation. It never states whether AcquisitionClosing is awaitable, has a wait/report method, or is an already materialized report, and never defines its output/report fields or relationship to AcquisitionObservation.

**Concrete first-use failure:**

```rust,ignore
let acquiring = context.acquire(selection, purpose, control)?;
let observation = acquiring.observation();
let closing = acquiring.cancel();
// No declared operation obtains the cancellation report from `closing`.
```

Neither `closing.await` nor `closing.wait().await` follows from a declared contract. Polling the separately captured observation may show terminal/cleanup facts, but does not define what the returned closing stage does. The implementation would have to choose an unreviewed public shape. The older authentication design explicitly states what AuthClosing yields; this amendment supplies no equivalent rule for source acquisition.

**Impact:** The design is incomplete at the explicit cancellation/report boundary required by the first real source. Consumers cannot write the complete cancel-and-observe flow from the public contract.

**Required correction:** Choose and specify the closing model. For example, declare AcquisitionClosing as an owned awaitable yielding a defined source-end report containing the terminal source facts and matching cleanup receipt, with bounded reporting and Drop behavior; alternatively define it as an immediate report with explicit observation access. Preserve the independent observation receipt and retained cleanup semantics. No spelling is mandatory.

Add a short public first-use example showing immediate admission refusal, `wait().await`, pure selection/identity-before-lookup, and explicit cancellation/report consumption. State explicitly which returned objects are futures and what each yields.

**Smallest closure check:** Trace the example using only declared public signatures. With a barrier-stalled source, cancel, consume the specified closing/report operation, observe Cancelled plus Pending, drop the report, and later observe matching cleanup confirmation through the retained observation. Also show the admission-error and successful-transfer branches. This is a design trace now; compilation and the fake regression belong to the authorized implementation gate.

**Classification:** Local specification/Surface defect. **NEW ARCHITECTURAL ROOT: NO.**

## 2. Invariant analysis

The following attacks did not establish another defect:

- **The two Result layers have distinct meanings.** `acquire` validates and reserves synchronously, so outer Err means no registered acquisition or source effect. `wait().await` reports registered source/control failure or atomically transfers material. That separation is meaningful; it is not inherently ceremony. A different API could represent those distinctions, but preserving either spelling is not a requirement.
- **Capture and paths:** source I/O is excluded from capture; the executing endpoint supplies bounded native base and caller authority. Worker cwd/caller recapture and local fallback are prohibited.
- **Prelookup identity:** explicit files use BeforeLookup without invented peer/generation. Every prospective lookup requires fresh acquisition; SourceIdentity compares exact admitted bytes and required lineage, while caller/policy, possession and fresh trust remain separate prerequisites.
- **Reuse versus authentication:** equality alone cannot authorize checkout. For a new connection, begin still requires actual verified peer/generation and LocalSshKeyPort borrows the same snapshot. NeedNetwork and uncertain physical disposal retain the existing key pin.
- **Other source timing:** VerifiedRequest preserves actual origin/peer/challenge prerequisites. BeforeLookup is expressly not blanket permission to contact helpers/native sources early.
- **Lost success and late work:** ready-but-untransferred material remains record-owned; transfer races close/deadline. Dropped acquisition/wait seals it, and late native completion cannot republish material.
- **Retention and capacity:** source/auth observations are distinct WorkObservation variants over one retainer. The 64-record allowance is shared, stalled work remains charged across owner replacement, and material/observation pins retain their own charges.
- **Native worker ownership:** registered session ownership, one bounded private result slot, retained JoinHandle, nonblocking completion inspection and actual join before disposal provide a concrete scoped route. A stored result or dropped handle is not disposal evidence.
- **Authority compatibility:** clause-specific supersession preserves the accepted authentication, physical-resource and provider policies. No fifth HostPort, Connection dependency, wire carrier or production integration is introduced.

The public first-use path is otherwise expressible as capture → acquire → `wait().await` → pure select → source identity before lookup → actual-peer authentication where required. P2-1 identifies the missing explicit cancellation/report branch.

## 3. Risks and next action

The design still requires implementation evidence for native exit versus stored-result ordering, panic/error cleanup, native filenames and regular-file admission on each supported platform. The draft explicitly retains those gates; their unexecuted outcomes are not findings against this design.

**Next action:** complete AcquisitionClosing’s public contract and add the compact first-use/cancel trace, then return the revised pinned design for focused closure. Production implementation remains paused until both axes report GO at the same tuple.
