# Credential portable implementation, round 2 — CODE-AXIS REVIEW

**Review object:** Portable `gwz-credential` implementation at `d6fc64d6a38457018377020765229405920a4833`, controlled by `dev-docs/Implementation.md`.

**Baseline:** Previous member `edf0d776df11917d47567194e1061d0e9c50c4ab`; immutable root authority `0de87d56`. Sources inspected through pinned `git show` and the complete implementation/test delta. Live root, builder advancement and adjacent implementations are excluded.

**Date:** 2026-10-19

**Axis:** Code, including Surface. Independent, adversarial and read-only. Both prior-round reports were legitimate inputs; no current-round peer material was read. Filed verbatim by the lane owner.

**Verdict: GO** — P2-5 is closed, all earlier closures are preserved, and no new P0–P3 finding was established.

**Architectural-root classification:** **NEW ARCHITECTURAL ROOT: NONE.** The previously identified observation-boundary root remains closed. Cumulative implementation-review root count remains **one**.

---

## Prior-finding closure table

| Finding | Closure |
| --- | --- |
| Code P2-1 / State P2-1: missing retained terminal observation | **Closed and preserved.** The pre-effect retainer observation, qualified terminal facts, independent revocation and matching cleanup receipt are unchanged. |
| Code P2-2: cancellation before pending scope transfer | **Closed and preserved.** Pending transfer still checks cancellation/deadline; successful transfer retains its specified lifetime. |
| Code P2-3 / State P2-4: uncharged selection allocation | **Closed and preserved.** Kernel target/path capacities and selection storage remain charged separately from provider allocation. |
| Code P2-4: native path representation | **Closed and preserved.** Native `PathBuf` selection and enclosed Unix non-UTF-8/newline regressions are unchanged. |
| State P2-2: key error/panic permits re-entry | **Closed and preserved.** Sticky terminalization and retained key-disposal ownership are unchanged. |
| State P2-3: shutdown replaces the original cause | **Closed and preserved.** Fixed terminal provenance and separate provider revocation are unchanged. |
| Code P2-5: observation omits its fixed peer allocation | **Closed.** `src/api.rs:200–215` now charges both fixed payloads, peer-buffer capacities and explicit two-Arc overhead. The new public-port regression checks surviving storage, budget refusal, shared accounting and last-receipt release. |

## Changed-range analysis

The sole production change adds `size_of::<PeerContext>()` and a documented 128-byte allowance for both Arc headers/alignment/bookkeeping to the observation reservation, using checked addition.

That complete charge is established before registration and provider effects, stored in `Observed`, and retained until the final observation or cleanup receipt disappears. Conversation retirement still releases only its separate conversation charge.

The added test support exposes the fake retainer’s charged bytes. The regression retains completed observations after conversation retirement, checks a storage lower bound and owner/shared-charge agreement, reaches `InputBudget` refusal, then preserves one cleanup-receipt clone through final release.

There is no public API, lifecycle, provider, dependency or native implementation change.

## 0. Evidence base

Read:

- Generated `CredentialPortable-PromptCode-2.md`, merged `CredentialPortable-RemPlan-2.md`, both round-1 reports and updated implementation evidence.
- Complete source/test/documentation delta from `edf0d776df11917d47567194e1061d0e9c50c4ab`.
- Current registration and failure accounting in `src/api.rs:182–298`.
- Observation ownership/destruction in `src/observation.rs:1–72`.
- Conversation retirement in `src/driver.rs:141–183`.
- The complete added retained-observation regression and fake-retainer accessor.

The earlier complete source, Surface, dependency and contract review remains applicable to unchanged paths.

At start and end, HEAD matched `d6fc64d6a38457018377020765229405920a4833`; tracked source was clean. Only the two authorized round-2 review prompts were untracked.

No builds, tests, writes, network operations, mutations or additional agents were run. Passing standalone test/fmt/clippy/doc gates are owner-reported evidence.

## Problem, requirement provenance, and scope

The operator authorized an isolated portable Credential implementation using the accepted API and boundary. This correction implements their existing finite-retention requirement.

The remedy is confined to this member’s accounting and focused fake regression. It requires no dependency upgrade, new infrastructure, design amendment or adjacent implementation. Native providers, production composition, placement and application migration remain separately authorized work.

## 1. Findings

No open findings.

## 2. Invariant analysis

The original P2-5 trace no longer reproduces:

1. Each registration reserves peer-buffer capacities, both fixed observation/peer payloads and allocation overhead.
2. Confirmed cleanup retires conversation capacity without subtracting the observation charge.
3. Retained observations accumulate charged storage and cause bounded admission refusal.
4. Owner and shared-retainer charges agree.
5. A surviving cleanup-receipt clone keeps the reservation alive.
6. Final receipt destruction releases the observation charge once.

The regression tests surviving storage rather than merely checking that some reservation remains, so it addresses the original omission.

The narrow delta preserves prior proofs: terminal observation remains available without caller polling; terminal causes remain fixed; provider revocation remains separately observable; pending scope transfer remains fenced; key failures remain terminal; native paths remain exact; and output/cleanup ownership remains separate from peer acceptance.

## 3. Risks and next action

Trusted providers and the supplied retainer must still satisfy their allocation, progress and disposal contracts. Portable fakes do not qualify native authentication, production scheduling, physical Connection disposal or endpoint placement.

**Next action:** accept the portable implementation checkpoint once both independent axes report GO at this same SHA. This verdict authorizes no native, integration or release claim.
