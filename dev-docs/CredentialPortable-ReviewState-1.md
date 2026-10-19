# Credential portable implementation round 1 — STATE-AXIS REVIEW

**Review object:** Entire portable `gwz-credential` implementation at `edf0d776df11917d47567194e1061d0e9c50c4ab`, controlled by `dev-docs/Implementation.md`. Portable implementation acceptance checkpoint; no native or integration qualification claimed.

**Baseline:** Previous member `3c342c4a57330f30dc1aeb2bc696251b66162c77`; current member `edf0d776df11917d47567194e1061d0e9c50c4ab`. Immutable root authority `0de87d56764792ea70cbe75e5ea0148d3ad7b43c`. Live root/builder advancement and adjacent implementations excluded.

**Date:** 2026-10-19

**Axis:** State — lifecycle legality, cancellation/deadlines, lock ordering, retained ownership, accounting, publication, scope reuse, lost futures and provider faults. Independent, adversarial and read-only. Nothing here relies on the current parallel review. Filed verbatim by the lane owner.

**Verdict: GO** — all four original State P2 findings are closed. No new P0–P3 finding was established.

**Architectural-root classification:** The previously identified missing observation boundary is closed. **NEW ARCHITECTURAL ROOT: NONE.** Cumulative implementation-review root count remains **one**.

---

## Prior-finding closure table

| Original State finding | Result and closure evidence |
| --- | --- |
| P2-1: retained terminal outcomes inaccessible | **Closed.** `RetainedWork::observation` exposes the charged receipt before provider effects (`src/provider.rs:15–23`, `src/api.rs:200–292`). `AuthObservation` supplies identity, exact peer/generation, immutable terminal facts and matching cleanup without release or transfer authority (`src/observation.rs:33–66`). Public-port regressions cover failed/lost opening, all ordinary owners disappearing, lost finish/cancel futures and generation isolation (`tests/observation.rs:7–109`). |
| P2-2: explicit-key error/panic leaves session live | **Closed.** Key access and authentication share the caught transition; errors terminalize before returning (`src/output.rs:235–255`). Re-entry encounters the sticky reason before native authentication. The key-use pin and disposal probe remain retained. Both error and panic regressions assert one native invocation, Retire and pending capacity until probe confirmation (`tests/key_use.rs:135–194`). |
| P2-3: shutdown replaces an existing terminal error | **Closed.** `Child::error` returns the recorded reason first; terminal publication writes its slot once (`src/state.rs:186–201,233–251`). Later provider revocation is separate. The original fault remains visible after operation/provider drop and subsequent response/finish attempts (`tests/lifecycle.rs:130–155`). |
| P2-4: selection metadata bypasses byte limits | **Closed.** Kernel target/path capacities and structural storage are added separately to provider allocation before retention (`src/values.rs:109–118`, `src/api.rs:125–159`). Inflated equal values are refused without accounting drift; zero-charge providers still consume bounded metadata allowance (`tests/selection.rs:7–112`). |

The merged plan’s additional pending-scope and native-path corrections were independently checked. Cancellation or deadline expiry before transfer now yields Accepted plus Cancelled/Timeout, Retire and no scope; transfer first preserves the specified lifetime across original-operation closure (`tests/observation.rs:110–175`). Native `PathBuf` selection preserves non-UTF-8 and newline names through policy comparison and provider selection (`tests/selection.rs:115–150`).

## Changed-range analysis

Reviewed the complete implementation delta from the previous member, then followed the affected public surface through admission, retention, terminalization, scope transfer, key calls and destruction.

The new observation allocation shares peer data with the child, carries its own reservation, and remains charged through its last receipt clone. Conversation retirement releases the conversation charge separately; observation destruction releases its charge and operation reference (`src/observation.rs:18–31`, `src/driver.rs:145–177`). There is no new terminal-history registry or cleanup owner.

Pending scope transfer now participates in cancellation/deadline checks in both retained progress and final transfer (`src/state.rs:196–231`, `src/future.rs:148–166`). Abandoning the finishing future settles the untransferred connection scope as Retire while preserving recorded peer acceptance.

The patch retains the established lock order and keeps provider/native callbacks outside authority and payload locks. New observation reads introduce no inverse lock acquisition. Documentation accurately distinguishes immutable terminal facts, later revocation and outstanding observation charges.

## 0. Evidence base

Read or re-read:

- All portable source files, including the new `src/observation.rs`, and the affected public/provider call graph.
- The complete implementation/test/documentation delta between the two member SHAs.
- All regression tests and existing lifecycle, ownership, publication, key-use, bounds and seeded-schedule tests, including `tests/support/mod.rs`.
- `README.md`, `dev-docs/Implementation.md`, package/toolchain metadata and the merged `CredentialPortable-RemPlan-1.md`.
- Both legitimate prior-round reports. No current-round peer report was read.
- Accepted Credential lifecycle/accounting/key-use requirements, particularly §§4–5; boundary terminal/revocation rules; relevant Connection retention requirements, using immutable root authority.

At both start and end:

- Member HEAD matched `edf0d776df11917d47567194e1061d0e9c50c4ab`.
- Tracked source was clean.
- Only the two authorized untracked round-1 review prompts were present.

No builds, tests, writes, network operations, mutations or additional agents were used. Passing standalone test, fmt, clippy and documentation gates are owner-reported evidence, not independently executed results.

## Problem, requirement provenance, and scope

The operator authorized an isolated portable Credential implementation using the accepted API and boundary contract. Its purpose is independently testable lifecycle/provider machinery before native providers or production integration.

The remediation implements existing requirements: retained terminal observation, sticky failure arbitration, cancellation-safe scope transfer, finite allocation and native-path preservation. It does not amend the accepted design or require broader infrastructure.

Changes remain within the portable member. The sole dependency remains pinned `zeroize = 1.9.0`; clocks, progress, providers and retention remain injected. Core, Connection, transports, Git, native providers and application integrations remain reference-only. No unsupported prerequisite or unauthorized implementation expansion was found.

Localized remedies resolved the original defects. Native qualification and actual composition remain separate gates; their absence does not block this checkpoint.

## 2. Invariant analysis

| Attack | Result |
| --- | --- |
| Fail startup while cleanup stalls, then lose the opening and owners | The pre-effect public observation retains the matching terminal reason, Retire and Pending receipt. Independent progress later confirms cleanup; the observation stays charged until dropped. |
| Drop accepted finishing and cancellation futures | Terminal facts remain observable without caller polling. Distinct conversation IDs and physical generations remain associated with their own receipts. |
| Record provider failure, then close operation/provider | The first cause remains fixed. Later revocation remains separately observable and prevents fresh use. |
| Return an error or panic from explicit-key authentication, then retry | The child is sealed; the second attempt does not re-enter authentication. Native disposal ownership survives the error. NeedNetwork remains the nonterminal continuation path. |
| Close the provider during an admitted key call | The in-flight pin prevents premature retirement; late success cannot escape revocation. Probe confirmation remains necessary. |
| Cancel or expire after Accepted but before scope transfer | Transfer rechecks control state under authority. Accepted remains a peer fact; no scope is published and disposition is Retire. |
| Transfer scope, then cancel/close its original operation | The transferred scope retains its specified lifetime. A new use still requires matching caller, policy, endpoint and complete peer/generation; provider closure fences commit. |
| Inflate content-equal selection capacity or declare zero provider allocation | Kernel metadata is charged independently. Over-budget admission fails; dropping selections balances accounting. |
| Commit output before/after cancellation | Cancellation first prevents publication. Commit first preserves the offered fact and queued output pin without claiming peer acceptance. |
| Stall cleanup, replace an owner, or repeatedly retire records | Shared retention admission still bounds pending work. Matching cleanup releases records once; no completed-session history accumulates. Caller-held observations remain explicitly charged. |
| Concurrent progress and foreign callbacks | The driving guard prevents concurrent stepping of one child. Foreign calls run outside library state locks; no new lock-order cycle was found. |

The regressions directly cover the original counterexamples. Inspection also covered their error, drop and ownership paths rather than relying solely on passing test assertions.

## 3. Risks and next action

Trusted provider ports and the supplied retention service must honor their nonblocking progress, allocation and disposal contracts. Portable fakes do not qualify real native providers, production scheduling, physical Connection disposal or installed endpoint behavior.

**Next action:** accept this portable implementation checkpoint at the reviewed member SHA. No additional State-axis remediation is required; native and composition work remains subject to its separately authorized gates.
