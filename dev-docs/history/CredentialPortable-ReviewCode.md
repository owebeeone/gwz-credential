# Credential portable implementation — CODE-AXIS REVIEW

**Review object:** Entire initial `gwz-credential` portable implementation at `3c342c4a57330f30dc1aeb2bc696251b66162c77`, including tracked source, tests, package metadata and documentation. Implementation checkpoint; not accepted or integrated.

**Baseline:** `gwz-credential` HEAD `3c342c4a57330f30dc1aeb2bc696251b66162c77`; immutable root authority `0de87d56`. Sources read with `git show HEAD:` and authority documents with `git show 0de87d56:`. Concurrent live-root and builder changes are excluded.

**Date:** 2026-10-19

**Axis:** Code, including Surface: architecture, interfaces, compatibility, lifecycle composition, allocation and trust boundaries. Independent, adversarial and read-only; nothing here relies on the parallel reviewer. Filed verbatim by the lane owner.

**Verdict: NO-GO** — four P2 findings block acceptance. One is a **NEW ARCHITECTURAL implementation root**; three are local implementation defects. The accepted design itself does not require amendment. I pre-commit to GO on a revision resolving P2-1 through P2-4 as specified, absent new blocking evidence.

---

## 0. Evidence base

Read:

- `README.md:1–83`, public exports and compile-fail examples in `src/lib.rs:1–28`, then the complete small source files:
  - `api.rs:1–335`
  - `driver.rs:1–183`
  - `future.rs:1–243`
  - `output.rs:1–257`
  - `provider.rs:1–99`
  - `state.rs:1–223`
  - `values.rs:1–292`
- All six test files and `tests/support/mod.rs:1–308`.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.gitignore`, `LICENSE`, and `dev-docs/Implementation.md:1–48`.
- Pinned Credential design, especially §§4–5, finite defaults and consumer mappings; boundary contract §§2–7; relevant Connection construction, authentication, cleanup and error clauses.
- Pinned workspace/process instructions, including native filesystem-name preservation under `AgentProcessRules.md` L2-08.

At both start and end:

- Member HEAD matched `3c342c4a57330f30dc1aeb2bc696251b66162c77`.
- No tracked modification was reported.
- The only untracked files were the authorized Code and State review prompts. The State prompt/report was not read.

No build, test, network operation, write or mutation was performed. Passing test/fmt/clippy results in `Implementation.md:27–34` are owner-reported evidence, not independently executed evidence.

## Problem, requirement provenance, and scope

The operator authorized an isolated portable implementation using the reviewed Credential API and boundary contract. The concrete goal is independent lifecycle/provider testing without core, Git, Connection, native workers or platform builds.

The delivered package respects that dependency boundary: its only dependency is pinned `zeroize = 1.9.0`; it introduces no runtime, network implementation, supervisor or application migration. Provider parsing, native qualification, production composition and placement remain legitimately deferred.

The findings concern capabilities and invariants already required by the accepted contracts. None requires implementing a deferred provider or changing another repository. In particular, deferring the production bridge does not defer the portable interface through which that bridge must observe terminal outcomes.

Localized remedies exist within this member. No dependency upgrade, new shared crate or broader infrastructure prerequisite is demonstrated or necessary.

## 1. Findings

### [P2-1] Retained conversations provide no terminal observation interface to the bridge

**Locations:** `src/provider.rs:15–33`; `src/values.rs:30`; `src/future.rs:12–63,132–193`; `src/state.rs:61–94,212–223`.

**Invariant:** Credential design §5.1, lines 287–309, requires an immutable terminal slot visible to the bridge and a cloneable observation receipt. Boundary §6, lines 147–158, requires translation and generation retirement even when the waiting caller disappears.

The retainer receives only `Arc<dyn RetainedWork>`, whose sole method returns `Poll<()>`. It cannot observe the conversation’s peer/generation, terminal reason, offered fact, disposition or cleanup receipt. `CleanupReceipt` observes cleanup alone and is obtained through `AuthEnd`. An opening failure returns only the `AuthError` alias; a dropped opening or finishing future exposes no retained terminal observation.

**Counterexample:**

1. Begin authentication for physical generation G and let native work start.
2. Drop the opening while cleanup remains pending.
3. The retained driver cancels the child and continues cleanup correctly.
4. The bridge’s public retained-work interface supplies only Pending/Ready. It cannot retrieve G’s terminal record or its matching cleanup receipt.
5. Alternatively, record peer acceptance and drop `AuthFinishing`: the peer fact remains private in `ChildState`, inaccessible through the retainer.

An adapter could reconstruct a second outcome ledger around every public stage, but that duplicates authority and still cannot obtain the hidden cleanup receipt on failed opening. This is the missing portable half of the accepted observation boundary, not a demand for production integration.

**Impact:** The future bridge cannot implement the required per-generation terminal/cleanup mapping using the delivered surface.

**Required correction:** Expose the accepted bounded terminal observation receipt to the bridge/retainer before effects. It must survive public-stage loss, identify its conversation/generation, preserve independent peer/publication/disposition facts, and expose matching cleanup progress without release authority.

**Smallest closure test:** A fake bridge retains only that public observation and retained-work interfaces. Drop an opening, then separately drop an accepted finishing future while cleanup stalls. Prove immediate matching-generation Retire/terminal observation, preservation of Accepted where recorded, subsequent cleanup confirmation, and exactly-once retirement without caller polling.

**Classification:** **NEW ARCHITECTURAL ROOT A1 — implementation observation boundary is absent.** This restores an already accepted design obligation; it does not justify redesigning the accepted API contract.

### [P2-2] Cancellation after accepted finish can still publish a new scope lease

**Locations:** `src/api.rs:300–318`; `src/driver.rs:32–41`; `src/future.rs:147–179`.

**Invariant:** Credential design lines 266–275 distinguishes peer acceptance from cancellation: acceptance remains a fact, while cancellation changes publication and physical disposition. A scope survives ordinary operation closure only after explicit transfer.

`finish(Accepted)` sets `terminal = true` and `scope_pending = true`. After that:

- The retained driver checks cancellation/deadline only when `!state.terminal`.
- `AuthFinishing::poll` checks provider closure and stored reason, but never the conversation cancellation flag.
- Consequently, a still-untransferred scope can be published after cancellation.

**Counterexample:**

1. Open a permitted Windows-logon connection-scoped conversation with a separately retained clone of `AuthControl.cancellation`.
2. Construct `session.finish(Accepted)` but do not poll it.
3. Cancel the conversation using that clone.
4. Drive retained work and poll the finishing future.
5. The result remains `reason = None`, `BoundScope`, with a usable `ScopeLease`.
6. A fresh matching operation can obtain and commit a grant from that lease.

The existing scope test covers provider closure after obtaining a grant; it does not cover cancellation before the initial scope transfer.

**Impact:** Cancelled authentication can publish fresh reusable authority.

**Required correction:** Fence pending terminal publication/scope transfer against conversation cancellation. Preserve the recorded Accepted fact, but return Retire and no scope when cancellation wins before transfer. Preserve the separately specified lifetime of a scope already transferred successfully.

**Smallest closure test:** Exercise both orders around initial scope transfer. Cancellation first must yield Accepted + Cancelled + Retire with no scope; successful transfer first must retain the contract’s ordinary operation-lifetime behavior.

**Classification:** Local implementation defect; **not a new architectural root**.

### [P2-3] Selection admission omits allocations retained by the kernel itself

**Locations:** `src/api.rs:120–156`; `src/state.rs:47–58`; `src/values.rs:64–81,98–111`; `src/provider.rs:41–53`.

**Invariant:** Credential design §5.2 bounds retained secret/context bytes, including selections. Known library-owned allocation must be charged or refused before retention.

Selection charges only `provider.selection_charge(&selection)`. The subsequent check measures `provider.retained_bytes()`. However, `Selected` also takes ownership of the supplied `ExchangeSelection`, including its independently allocated target and explicit-path strings. Their capacities are neither added nor checked against the admitted charge.

**Counterexample using the delivered fake provider:**

1. Prepare an operation with its normal small allowed HTTPS selection.
2. Construct an equal selection whose origin string has, for example, 32 MiB capacity but the same short contents.
3. `validate()` checks length; `allowed.contains()` compares contents. Both pass.
4. The fake provider declares 64 bytes and retains its small key snapshot.
5. `select()` succeeds, retaining the 32 MiB string while adding only 64 bytes to the operation/shared charge.

This exceeds the default 16 MiB operation allowance before a conversation even begins. More generally, intrinsic `Selected` storage also has no independently enforced charge floor.

**Impact:** The advertised retained-byte limits do not bound memory actually retained by prepared selections.

**Required correction:** Account for kernel-owned selection storage and string capacities separately from provider-declared additional allocation, or retain a bounded charged reference to the admitted policy selection. Include fixed record overhead consistently and use checked arithmetic.

**Smallest closure test:** Supply content-equal selections with oversized capacities and a provider declaring only its own small allocation. Admission must refuse over-budget storage or normalize it into a demonstrably bounded charged representation. Also test a zero-allocation provider against repeated prepared selections.

**Classification:** Local accounting defect; **not a new architectural root**.

### [P2-4] Explicit-key paths cannot preserve native filesystem names

**Locations:** `src/values.rs:86–111,152–155`.

**Invariant:** `AgentProcessRules.md` L2-08 requires preserving native path bytes/units. The accepted Credential design §3 keeps paths endpoint-local and §4.1 retains the existing explicit-file route; implementation did not authorize narrowing path representation.

`Source::ExplicitKey(String)` makes UTF-8 a prerequisite for expressing a selected key path. Validation additionally rejects every control character, including valid Unix filename characters.

**Counterexample:** An endpoint-local Unix key path containing a non-UTF-8 byte cannot be represented by this API. Converting it lossily selects different filesystem bytes. A valid path containing a newline is representable as a native path but is rejected before the provider can apply its source policy.

Native provider execution is deferred; the path-bearing interface shape is not.

**Impact:** A future conforming provider cannot preserve native explicit-key selection through the portable API without changing that API or introducing an undocumented encoding convention.

**Required correction:** Use a native path representation such as `PathBuf`, with explicit bounded native-byte/unit accounting and appropriate source validation. Keep diagnostics redacted and endpoint-local resolution unchanged.

**Smallest closure test:** Portable selection/capture tests preserve an endpoint-local native path exactly, including a Unix non-UTF-8 case through an explicitly enclosed platform test module. No filesystem access or native authentication is required.

**Classification:** Local compatibility/interface defect; **not a new architectural root**.

## 2. Invariant analysis

| Area attacked | Result |
| --- | --- |
| Owner/handle separation | Unique provider and operation owners; cloneable submission/selection views cannot close another owner. |
| First use and synchronous close | Build supplies all ports. Close construction seals authority before polling; selected-before-close begin is checked before provider start. |
| Independent conversations | Child cancellation is separate from sibling operation/conversation authority; tests exercise this distinction. |
| Publication | Output has no general token getter, exact peer comparison precedes queue insertion, and provider/operation closure shares the publication lock. Committed output remains charged through queue ownership. |
| Native key pinning | Key calls occur outside state locks; in-flight pins and disposal probes prevent premature record release. Barrier tests cover provider closure during a call. |
| Cleanup truthfulness | Failed or pending disposal retains capacity. Matching confirmed cleanup updates receipts; disposed records are removed once. |
| Replacement-owner capacity | Shared injected retention admission prevents replacement owners bypassing pending conversation capacity. |
| Defaults and dependency isolation | Documented defaults are present; configuration validates finite nonzero bounds. Only pinned zeroize is imported; fake tests have no forbidden library/native dependency. |
| Surface/lifecycle completeness | Public close/cancel pairs are present, but retained terminal observation is incomplete (P2-1), pending scope publication misses cancellation (P2-2), selection accounting is incomplete (P2-3), and path representation narrows compatibility (P2-4). |
| Secret/trust boundary | Response/key access is confined to declared privileged local ports. This does not protect against malicious in-process adapters, and the documentation correctly avoids claiming otherwise. |
| Scope authorization | No inspected source expands into native providers, production bridge, Connection implementation or application migration. |

The sustained and seeded tests provide useful portable coverage. Their present oracles inspect caller-visible outcomes and aggregate retention; they do not establish the missing bridge observation boundary or the counterexamples above.

## 3. Risks and next action

Real provider correctness, native containment, production scheduling, placement and actual Connection reuse remain separate authorized gates. Their absence does not block this portable checkpoint.

The next action is a bounded member-only remediation of P2-1 through P2-4, followed by review at a new settled member SHA. Architectural-root count for this implementation review: **one**. No design amendment or implementation outside `gwz-credential` is required by this report.
