# Credential portable implementation round 2 — STATE-AXIS REVIEW

**Review object:** Portable `gwz-credential` implementation at `d6fc64d6a38457018377020765229405920a4833`, controlled by `dev-docs/Implementation.md`. Final focused portable implementation closure; no native or integration qualification claimed.

**Baseline:** Previous member `edf0d776df11917d47567194e1061d0e9c50c4ab`. Immutable root authority `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`. Live root/builder advancement and adjacent implementations excluded.

**Date:** 2026-10-19

**Axis:** State — lifecycle, cancellation/deadlines, retained ownership, accounting, publication, scope reuse and provider faults. Independent, adversarial and read-only. No current-round peer material was read. Filed verbatim by the lane owner.

**Verdict: GO** — the surviving observation-accounting omission is closed; previous State closures are preserved. No new P0–P3 finding was established.

**Architectural classification:** **NEW ARCHITECTURAL ROOT: NONE.** Cumulative root count remains **one**, the already closed missing observation boundary.

---

## Prior-finding closure table

| Prior finding | Current result |
| --- | --- |
| State P2-1 / Code P2-1: missing retained terminal observation | **Closed, preserved.** Pre-effect observation, immutable terminal facts, generation association and independent cleanup receipt are unchanged. |
| State P2-2: key error/panic permits re-entry | **Closed, preserved.** Sticky terminalization, native-call pin and disposal-probe retention are unchanged. |
| State P2-3: shutdown replaces the terminal cause | **Closed, preserved.** Recorded reason retains precedence; later provider revocation remains separate. |
| State P2-4 / Code P2-3: uncharged selection metadata | **Closed, preserved.** Kernel capacities and structural selection charge remain separate from provider allocation. |
| Code P2-2: cancellation/deadline before scope transfer | **Closed, preserved.** Pending transfer remains fenced; Accepted remains independent of cancellation. |
| Code P2-4: native explicit-key paths | **Closed, preserved.** `PathBuf` representation, exact native comparison and enclosed platform regressions are unchanged. |
| Code P2-5: observation omits fixed peer allocation | **Closed.** `src/api.rs:200–215` now charges both fixed payloads, peer buffers and explicit two-Arc overhead. `tests/observation.rs:177–236` exercises surviving charges, budget refusal and final receipt release. |

These are source-traced conclusions. Test execution is owner-reported.

## Changed-range analysis

The sole production-source change is the observation-charge calculation in `src/api.rs:200–211`. It now adds `size_of::<Observed>()`, `size_of::<PeerContext>()` and a documented 128-byte allowance for both Arc headers and allocation bookkeeping, using checked additions.

That complete amount follows the existing reservation and lifetime:

1. Shared retention reserves it before effects (`src/api.rs:212–215`).
2. Operation/owner admission includes it with the separate conversation charge (`src/api.rs:223–227`).
3. The observation stores the charge and reservation (`src/api.rs:237–245`).
4. Conversation disposal subtracts only `record_bytes` (`src/driver.rs:158–170`).
5. The surviving observation charge is released by `Observed::drop` after its last observation or cleanup-receipt clone disappears (`src/observation.rs:18–31,33–66`).

Public APIs, terminal arbitration, scope transfer, locking, provider calls and dependencies have no delta. The added test-helper accessor reads shared retained bytes; it adds no production authority or lifecycle.

## 0. Evidence base

Read the complete canonical State prompt, merged `CredentialPortable-RemPlan-2.md`, both legitimate round-1 reports, the complete source/test/documentation delta, and the affected admission, observation-destruction and conversation-retirement paths.

Inspected the full new regression at `tests/observation.rs:177–236` and its retention-accounting support at `tests/support/mod.rs:23–100`. Rechecked the accepted finite-retention requirement at immutable root Credential design §5.2. The earlier complete source/test/package review remains applicable to unchanged material.

At start and end:

- Member HEAD matched `d6fc64d6a38457018377020765229405920a4833`.
- Tracked source was clean.
- Only the two authorized round-2 prompts were untracked.
- Immutable root authority resolved to `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`.

No builds, tests, writes, network operations, mutations or additional agents were used. Passing standalone test, formatter, clippy and documentation gates are owner-reported evidence.

## Problem, requirement provenance, and scope

The operator authorized isolated portable Credential lifecycle/provider implementation against the accepted contracts. Those contracts require retained context and observations to remain charged throughout their actual ownership lifetime.

The observed omission was local: a completed observation continued owning a separately allocated fixed peer payload after its conversation charge retired. Adding that payload to the surviving reservation directly addresses the defect without changing lifecycle or architecture.

The correction remains within the authorized member. No dependency, infrastructure, native-provider or integration prerequisite was introduced. Core, Connection, transport, Git and platform implementations remain reference-only.

## 2. Invariant analysis

The original accounting counterexample now fails: repeatedly complete conversations while retaining observations. Conversation counts return to zero, but each surviving receipt retains the complete observation reservation. Admission eventually refuses another conversation against the finite operation allowance.

The regression checks an independently expressed minimum surviving allocation, owner/shared charge agreement, refusal, and retention through a cleanup-receipt clone after ordinary observations disappear. Dropping that final receipt returns usage to the operation baseline; dropping the operation returns both owner and shared usage to zero.

Failure rollback also preserves balance: a rejected registration subtracts the combined charge, while its untransferred reservations drop (`src/api.rs:287–293`). No additional release path was added.

The source delta cannot bypass the prior cancellation, first-error, key-pin or publication guards: it changes only the amount admitted before the existing registration transition. The previously reviewed lock order and separate native-cleanup, conversation and observation lifetimes are preserved.

## 3. Risks and next action

Portable fakes do not qualify real providers, production retention scheduling, physical Connection disposal or endpoint placement. Those remain separate gates.

**Next action:** accept the portable implementation checkpoint at `d6fc64d6a38457018377020765229405920a4833`. No further State-axis remediation is required.
