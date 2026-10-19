# CredentialExplicitKey — CODE-AXIS REVIEW

**Review object:** Controlled acquisition API and endpoint-local explicit SSH file-key source at `gwz-credential e4a080b3d99a4ab98478d0b06c8d0d104e4180a0`; implementation acceptance, initial round 0. Production/test candidate: `3e995edd53d315f639cbfe167fd4562778ee42b9`.
**Baseline:** Member diff `648aae24e6ba00e27b65a56314739a67ee1ccc33..e4a080b3d99a4ab98478d0b06c8d0d104e4180a0`; immutable root authority `a80515f364194b04e38c4a6d59597cee0d8347cc`; private evidence `cd4acf0af1aa662356680bcb8a649c5b5e518e23`. Sources and authorities read through pinned `git show` objects. Concurrent root HEAD is excluded.
**Date:** 2026-10-20
**Axis:** Code, with Surface folded in. Independent, adversarial, read-only. No current implementation peer report was read or used. Filed verbatim by the lane owner.

**Verdict: NO-GO** — one P2 blocks; one P3 does not block. I pre-commit to GO on a revision that resolves P2-1 as specified, preserves the other reviewed invariants, and supplies the focused regression evidence.

**NEW ARCHITECTURAL ROOT classification:** None. Both findings are local implementation omissions against the accepted design; neither requires a new ownership boundary, lifecycle or provider abstraction. This is implementation round 0, separate from the earlier design correction rounds.

---

## 0. Evidence base

- Read the public README and acquisition/provider API before the implementation plan. Traced build → captured operation → directly awaited acquisition → PreparedCredential/source identity → actual-peer begin/respond/finish, including cancel, errors and Drop.
- Reviewed the complete member delta: 32 files, 3,093 insertions and 241 deletions. Inspected `src/acquisition.rs:1–348`, `acquisition_driver.rs:1–166`, `acquisition_observation.rs:1–99`, `acquisition_state.rs:1–73`, `explicit_key.rs:1–130`, `key_worker.rs:1–455`, `key_file.rs:1–315`, `key_container.rs:1–305`, and the affected API, driver, Future, state, values, exports and provider-port changes.
- Read new acquisition/file-source/admission tests, affected selection tests, fake-provider support and the migrated lifecycle, bounds, key-use, observation, publication and schedule tests. Inspected the existing retained-authentication call paths affected by acquisition controls and shared accounting.
- Compared `Implementation.md`, `CredentialExplicitKey-Plan.md`, `CredentialPlatform-Results.md`, Cargo manifests/lockfile and the accepted root Acquisition amendment, Credential API, Connection API and Connection–Credential boundary. Consulted repository/process/evidence rules and the review-loop skill. Existing core algorithms were reference-only at `461ddf4bc6892373a00b600da9778efb08ffc5d6`.
- Read the pinned private qualification run README at `campaigns/credential-qualification/runs/2026-10-19-explicit-source/README.md`. It and the public results report PASS for fmt/test/clippy/doc on pinned Rust 1.95.0, native Mac ARM64, Windows MSVC E: Healthy/ReFS and Linux Pi aarch64. I did not rerun those gates or independently audit every raw result file. These are source/library gates, not native SSH or Connection acceptance.
- `git diff --check` passed. The production/tests/Cargo diff from `3e995edd` to `e4a080b3` is empty. No builds, tests, network operations, writes or Git mutations were performed by this reviewer.
- Verified all three pinned objects at start and end. Member HEAD remained `e4a080b3d99a4ab98478d0b06c8d0d104e4180a0`; tracked state remained clean. Only the two declared generated implementation prompts were untracked. No tuple discrepancy was found.

## Problem, requirement provenance, and scope

The operator requested a public standalone Credential library, a real explicit SSH file-key source, independent native platform qualification, and correction of the acquisition boundary before implementation. The concrete deficiency was a source/selection seam that could not honestly represent controlled asynchronous acquisition and retained native cleanup. The operator subsequently authorized redesign of the affected lifecycle and a directly awaited acquisition Future. The accepted amendment supplies the resulting behavioral requirements; its earlier design GO does not establish this implementation's correctness.

The editable boundary comprises this member's library, provider, standalone tests, CI and documentation, plus separately authorized managed metadata and private qualification evidence. Core, Connection, transport, Git, SSPI, CLI, Python and existing supervisors remain reference-only. The implementation stays within that boundary. Real SSH cryptography/authentication, other providers, application integration and placement/reuse qualification remain separate gates, accurately disclosed in README and implementation status.

The exact `base64=0.22.1` addition is justified by the existing core decoding route and the inspected bounded container decoder; no dependency upgrade or general runtime/supervisor was introduced. The decoder uses bounded stack scratch instead of allocating a decoded key copy. A custom replacement decoder is not a necessary prerequisite. The revised lifecycle work and native worker are authorized by the operator's whole-boundary correction, rather than merely by the generated plan. No component-level unauthorized expansion or unsupported infrastructure prerequisite was established. The localized remedies below fit the existing design and scope.

## 1. Findings

### [P2-1] Begin forwards an extended deadline to the authentication provider

**Location:** `src/api.rs:132–157,211–216`; `src/driver.rs:67–73`. The incomplete regression is `tests/acquisition.rs:100–118`.

**Violated invariant:** The acquisition's original absolute controls must survive transfer and cannot be extended at begin. The accepted amendment at root `a80515f3`, §3, lines 164–172 requires preservation of those facts; README lines 33–36 explicitly promises that a later control cannot extend the acquisition deadline. Connection contract §10, lines 710–713 requires native cutoffs to derive from the remaining budget without an unlabelled extension.

**Trace:**

1. Acquire with absolute deadline 50 and successfully transfer PreparedCredential while the injected clock is 0.
2. Call `prepared.begin(peer, AuthControl::new(1000))` at time 0. The original deadline check passes, but registration stores the new control unchanged in Child.
3. The retainer calls `SelectedCredential::start(peer, 1000, ...)`, although only 50 units of the original request budget remain.
4. A conforming provider starting a retained native operation from that supplied cutoff therefore receives permission to run until 1000. Subsequent kernel polling still notices the original deadline and can report Timeout at 50; that does not correct the cutoff already given to native work.

This is directly visible in the public provider call arguments; it does not depend on implementing the deferred native SSH adapter. The existing regression reproduces the differing deadlines but checks only the kernel's later Timeout. Its fake provider ignores the deadline argument, so it misses this boundary failure.

**Impact:** Kernel publication remains fenced by the original deadline, but native provider work is initialized with an extended budget and may retain resources or continue effects beyond the original cutoff. This breaks the advertised provider contract and future provider composability.

**Required correction:** Establish an effective absolute deadline no later than either the preserved acquisition deadline or the begin deadline before storing/forwarding conversation control. Rejecting an extension is also valid if consistently documented; silently forwarding the later deadline is not. Preserve the existing checks of both cancellation authorities.

**Smallest closure test:** Make the fake SelectedCredential record the deadline passed to `start`. Acquire at deadline 50, begin at 1000, and assert that provider start receives 50 or begin rejects the extension before effects. Also acquire at 1000 and begin at 50 to verify that the stricter begin deadline wins. Retain the existing time-50 output/Timeout and cleanup assertions.

**Root classification:** Local deadline propagation defect. **NEW ARCHITECTURAL ROOT: NO.**

### [P3-1] Source observations omit the promised operation identity

**Location:** `src/acquisition_observation.rs:32–39,49–72`.

**Violated invariant:** Accepted Acquisition amendment §4, lines 287–294 requires AcquisitionObservation to identify provider namespace, operation and acquisition serial, as well as endpoint/target/source kind.

**Trace:** Create two operations with identical captured endpoint/target facts. Issue two acquisitions from operation A and one from B. A retainer can obtain all three public WorkObservation values before effects and retain them after public operation contexts disappear. `AcquisitionObservation::id()` exposes only `(namespace, acquisition_serial)`; its remaining accessors expose no operation identifier. The private `Arc<Operation>` does not make that attribution available through the public port. Unique acquisition serials distinguish records but cannot establish which two belong to the same operation.

**Impact:** The independent retention service cannot perform the promised operation-level attribution using the observation surface alone. Terminal outcome, revocation and cleanup remain observable, so this is a bounded Surface/diagnostic omission rather than a demonstrated cleanup or authorization failure.

**Required correction:** Expose a stable opaque operation identity, either separately or as part of the source observation identifier. Keep it free of caller secrets, paths and lifecycle authority.

**Smallest closure test:** Obtain observations for A1, A2 and B1 with identical endpoint/target/source inputs. Assert equal operation attribution for A1/A2, distinct attribution for B1, distinct acquisition identities, and stable values after caller/operation loss and cleanup.

**Root classification:** Local public observation accessor omission. **NEW ARCHITECTURAL ROOT: NO.**

## 2. Invariant analysis

### Surface and lifecycle

The acquisition Surface implements the accepted direct-Future shape: one `acquire(...).await` yields PreparedCredential; there is no extra public wait/Acquired/select ladder. Optional observation and immediate consuming cancel form a complete lifecycle. Failed admission is a ready error with no source observation/effect. Pending Drop seals unpublished work. Successful borrowed/manual polling transfers once; subsequent cancel reports fixed success without revoking the prepared value, completed Drop releases only its view, and re-poll returns WrongState without rewriting terminal facts. Completed errors retain their original reason. These traces are represented in `tests/acquisition.rs` and agree with `acquisition.rs:231–322`.

The README describes the construction chain, required absolute controls, retainer obligations, default budgets and deferred outcomes. Existing authentication opening/wait behavior remains distinguishable from acquisition. The operation-attribution omission is P3-1; provider deadline propagation is P2-1.

### Admission, publication and retained ownership

Acquisition reserves declared snapshot/work maxima and observation storage before registration/effects. Source/authentication records share the same configured allowance. The retention service receives the registered source owner before provider work starts; source work is progressed independently of caller Future polling. Provider/clock metadata calls inspected here occur outside authority/payload locks. Unknown start effects and failed cleanup are retained rather than declared disposed.

The source record distinguishes private material readiness, native cleanup and public transfer. Transfer checks owner/operation/control under the authority guard and requires cleanup. The retained worker owns its JoinHandle and private result slot; a stored result is insufficient until actual thread completion and join. Cancel, owner loss, worker panic, ready-but-unconsumed Future Drop and result-before-exit tests attack those distinctions. No detached successful worker or premature cleanup confirmation was established on the inspected explicit-file path.

Snapshot storage is shared by prepared, identity, authentication and native-key pins. Its charge survives source-job disposal and is released after the final material owner; key buffers use Zeroizing ownership. Source/terminal observations retain their own charge through the last clone. The new surviving-observation and failed-unpolled-Future tests address storage that outlives work completion. Shared retention admission prevents replacing an owner to evade pending native-work limits.

### Source, identity and authentication separation

Acquisition has no fabricated physical peer/generation. VerifiedRequest preserves its supplied peer/purpose and rejects a mismatching later begin. Explicit-file acquisition is BeforeLookup, reads fresh bytes, and cannot itself authorize checkout. SourceIdentity compares exact admitted bytes with endpoint/target/source/provider lineage while retaining the same snapshot. It neither exports key bytes nor grants fresh selection after revocation.

Authentication remains bound to the actual peer. Exact key bytes survive NeedNetwork without reopening the path, and native-key cleanup proof gates release. Provider completion remains separate from remote acceptance; output/scope publication remains guarded. Kernel checks preserve the original source cancellation/deadline after transfer, with the provider cutoff exception identified in P2-1.

### Native file source, parsing and isolation

Captured native PathBuf/base inputs resolve relative paths without ambient cwd recapture. Qualified Mac/Linux nonblocking opens and post-open regular-file checks address FIFO replacement; Windows namespace/device screening and regular-file checks are explicit. Tests cover native path behavior, missing/refused sources, no fallback, exact line-ending identity, file replacement, cancellation and retained key use.

The reader bounds input to 1 MiB and the decoder bounds scratch storage. Framing/refusal tests cover empty/oversize, encrypted, malformed, multiple PEM containers and unsupported native forms. These checks do not establish cryptographic validity; the README and qualification evidence explicitly reserve that decision for the trusted native SSH port. APFS refusal without lossy fallback and Linux actual nonUTF8 creation/read are accurately distinguished.

Product tests and dependencies remain standalone; they do not require private evidence or adjacent core. The narrow core algorithm reference does not add a product dependency. Reported native source/library qualification is evidence for those gates only.

## 3. Risks and next action

The external retention service remains responsible for bounded periodic progress and preserving pending registrations during shutdown. An uninterruptible filesystem call can retain its reserved work indefinitely; the implementation reports Pending rather than inventing disposal. Native SSH cryptography/authentication and Connection/application composition still require their separately authorized qualification. None is made a prerequisite for this source/library deliverable.

Next action: correct P2-1 within the current member, add its provider-argument regression, run the focused affected gates, and return the settled correction for originating closure. P3-1 can be corrected alongside it or recorded as nonblocking follow-up. No architectural redesign or scope expansion is required.
