# Credential portable implementation, round 1 — CODE-AXIS REVIEW

**Review object:** Entire portable `gwz-credential` implementation at `edf0d776df11917d47567194e1061d0e9c50c4ab`, including the revised observation surface and affected lifecycle/accounting paths.

**Baseline:** Previous member `3c342c4a57330f30dc1aeb2bc696251b66162c77`; current member `edf0d776df11917d47567194e1061d0e9c50c4ab`. Immutable root authority remains `0de87d56`. Sources were inspected using pinned `git show` and the complete relevant delta. Concurrent root/builder advancement is excluded.

**Date:** 2026-10-19

**Axis:** Code, including Surface. Independent, adversarial and read-only. Prior reports and merged remediation are legitimate inputs; no current-round peer material was read. Filed verbatim by the lane owner.

**Verdict: NO-GO** — all four original Code findings are closed, but one P2 accounting defect in the new observation storage blocks acceptance. **No NEW ARCHITECTURAL root.** Cumulative architectural-root count remains **one**. I pre-commit to GO on a revision resolving P2-5 as specified, absent new blocking evidence.

---

## Prior-finding closure table

| Prior finding | Result | Closure evidence |
| --- | --- | --- |
| Code P2-1 / State P2-1: missing retained terminal observation | **Closed** | `RetainedWork::observation()` supplies the receipt before effects. `AuthObservation` exposes qualified identity, peer/generation, immutable terminal facts, revocation and matching cleanup without release authority. `tests/observation.rs:7–109` covers failed/lost openings, dropped terminal futures, accepted facts, generation separation and cleanup after ordinary handles disappear. |
| Code P2-2: cancellation before initial scope transfer | **Closed** | Driver and finishing projection call `refresh_pending`; cancellation/deadline fence pending transfer while preserving Accepted. `tests/observation.rs:111–175` covers cancellation/timeout first and successful transfer first. |
| Code P2-3 / State P2-4: uncharged selection allocation | **Closed for the original counterexamples** | `ExchangeSelection::charge()` accounts for target/path capacities and fixed selection storage independently of provider allocation. `tests/selection.rs:7–113` covers inflated equal values and a zero-charge provider. A separate omission exists in the newly introduced observation allocation: P2-5. |
| Code P2-4: native key-path representation | **Closed** | `Source::ExplicitKey(PathBuf)` preserves native names; validation no longer imposes UTF-8 or rejects newline names. `tests/selection.rs:115–150` passes exact native values through the provider seam inside an enclosing Unix module. |
| State P2-2: key error/panic leaves authentication live | **Closed on inspected paths** | `src/output.rs:235–248` catches key access/native failure and terminalizes errors before return. `tests/key_use.rs:153–193` checks no re-entry, sticky retirement and retained disposal ownership. |
| State P2-3: shutdown replaces the recorded cause | **Closed on inspected paths** | Stored reason precedes later provider closure in `Child::error`; immutable terminal facts and separate revocation preserve provenance. `tests/lifecycle.rs:131–155` and key-failure tests exercise this ordering. |

These are source-traced closures. Regression execution is owner-reported, not independently rerun.

## Changed-range analysis

The patch restores the accepted observation boundary without adding another supervisor or library dependency. The retainer can obtain a cloneable receipt before provider effects, and the receipt cannot cancel, transfer scope or release capacity.

Observation lifetime is separated from conversation lifetime. Terminal facts remain fixed; later provider revocation is separately observable. Pending scope transfer is checked against cancellation/deadline, while transferred scope retains its specified operation-independent lifetime. Key failures now seal re-entry, selection metadata receives its own charge, and explicit paths use native storage.

The remaining defect is in that lifetime split: the observation reservation includes dynamically allocated peer contents but omits the fixed `PeerContext` allocation that survives conversation retirement.

## 0. Evidence base

Read the regenerated canonical prompt, both original reports, merged `CredentialPortable-RemPlan-1.md`, revised README/public exports, new `observation.rs`, and the affected API, state, driver, future, output and value call graphs. Inspected the changed tests completely, including observation, selection, key-failure, terminal-provenance and sustained-retirement regressions.

The unchanged initial source/test/package review remains applicable. Package metadata, lockfile, toolchain, license and dependency surface have no delta. Controlling requirements remain the accepted Credential API, Connection API and boundary contract at root `0de87d56`.

At start and end:

- Member HEAD matched `edf0d776df11917d47567194e1061d0e9c50c4ab`.
- No tracked modification was reported.
- Only the authorized round-1 Code and State prompts were untracked.

No builds, tests, writes, network operations, mutations or additional agents were run. Standalone test/fmt/clippy/doc success is owner-reported evidence.

## Problem, requirement provenance, and scope

The operator authorized an isolated portable implementation using the reviewed APIs and boundary. The required outcome remains independently testable credential lifecycle/provider machinery, without core, Git, Connection implementation, native workers or production composition.

The remediation stays within this member. No new dependency, runtime, supervisor, protocol or adjacent source change appears. Deferred native providers, parsing, cryptography, placement and production integration are not prerequisites for resolving P2-5.

The accepted finite-retention requirement already covers the new observation allocation. Its correction is local accounting work, not a design amendment or scope expansion.

## 1. Findings

### [P2-5] Long-lived observation charge omits the fixed peer allocation

**Locations:** `src/api.rs:200–215,232–240`; `src/observation.rs:18–30`; `src/driver.rs:158–170`.

**Violated invariant:** Credential design §5.2 requires retained context/observation storage to remain within the operation, owner and shared-retention byte allowances throughout its actual lifetime.

`observation_bytes` adds:

- Capacities of the peer’s endpoint, target, verified-peer and channel-binding buffers.
- `size_of::<Observed>() + 64`.

Registration then creates two separately allocated Arc payloads:

```rust
let peer = Arc::new(peer);
let receipt = CleanupReceipt(Arc::new(Observed {
    peer: peer.clone(),
    // ...
}));
```

The charge includes the `Observed` payload but never `size_of::<PeerContext>()`. The additional 64 bytes cannot cover that missing payload plus Arc overhead on the supported 64-bit layout: `PeerContext` alone contains a String, a target capable of containing two Strings, two Vecs and a generation.

The conversation’s separate 2 KiB structural allowance does not close the omission. The driver subtracts the entire conversation charge and releases its registration when cleanup and pins settle, while retained observations continue owning `Arc<PeerContext>`.

**Counterexample:**

1. Keep one operation alive.
2. Repeatedly complete conversations, retaining their `AuthObservation` or cleanup receipts.
3. Drive each conversation to disposal, then drop its session/end so the conversation charge is released.
4. Each retained receipt still owns both fixed allocation payloads and the peer buffers.
5. Accounting retains only `observation_bytes`; every receipt’s fixed peer payload remains undercharged.
6. Repetition accumulates the discrepancy before the configured byte limit refuses another admission.

The new tests prove that observations retain *some* reservation and release it eventually. They do not verify that the reservation covers all storage surviving conversation retirement.

**Impact:** The stated retained-byte allowances understate the actual library-owned observation storage. The omission accumulates across independently retained terminal receipts.

**Required correction:** Include the fixed `PeerContext` payload in the observation charge, with an explicit allowance for both Arc allocations. Keep that complete charge until the last observation/cleanup-receipt owner disappears. Use checked arithmetic.

**Smallest closure test:** Retain several completed observations after their conversation registrations retire. Verify that the remaining charge covers the peer buffers, both fixed payloads and their declared allocation overhead; then verify budget refusal and balanced release when the last receipt drops.

**Classification:** Local accounting defect introduced by the observation-storage implementation. **NEW ARCHITECTURAL ROOT: NO.**

## 2. Invariant analysis

| Area attacked | Result |
| --- | --- |
| Retained observation | The public retainer seam now exposes terminal and matching cleanup facts independently of caller polling. |
| Generation separation | Observation identity includes supplied namespace and local serial; peer facts preserve the physical generation. Separate conversations have separate observations and receipts. |
| Terminal provenance | Fixed reasons and peer facts survive later shutdown. Provider revocation is a separate monotonic fact. |
| Scope transfer | Cancellation/deadline before pending transfer prevents scope publication; ordinary operation closure after successful transfer does not revoke it. |
| Key failure | Error/panic seals authentication while its disposal probe and charged pin remain retained. |
| Selection bounds | Inflated target/path capacities and intrinsic selection storage are charged independently of provider declarations. |
| Native paths | Exact native path values reach selection/provider comparison without lossy conversion; explicit-path diagnostics are redacted. |
| Cleanup ownership | Confirmed disposal retires conversation capacity once; retained observation storage remains separately owned. Its charge needs P2-5’s correction. |
| Surface and construction | Required ports are explicit; ownership and close/cancel pairs remain intact. Observation has no release or scope-transfer authority. |
| Dependencies and scope | Only pinned zeroize remains; no native implementation, production bridge or adjacent component was added. |

The original publication, sibling-isolation, absolute-deadline, replacement-owner capacity and native-pin invariants remain supported by the inspected call graph and existing regressions.

## 3. Risks and next action

Native/provider qualification, production scheduling, actual composition and endpoint placement remain separate gates. Their absence does not block this portable checkpoint.

**Next action:** correct P2-5 within this member, add the focused retained-observation accounting regression, rerun the standalone gates and return the settled revision for closure. The original architectural observation root is resolved; cumulative architectural-root count remains **one**.
