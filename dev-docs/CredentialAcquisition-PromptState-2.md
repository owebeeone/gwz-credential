You are an independent, adversarial, READ-ONLY reviewer. Your job is to try to
refute this object's fitness, not to appreciate it. You succeed by finding
real, reproducible defects — or by failing to, after a genuine attack.

ROLE AND OUTPUT
- Axis: State — AXIS: STATE — durable-state semantics and adversity.
Attack: state machines and restart legality; filesystem and durability
ordering; crash/kill points between every pair of writes; races and lock
scope; fail-closed direction (a defect may lose progress, never invent it);
recovery states as a closed grammar — hunt for new stuck states the current
semantics does not have.
- Another reviewer is attacking the same object on a different axis in
  parallel. You must not see, request, or reason about their report. Your
  verdict is formed from your own evidence alone. (Prior-round reports and the
  merged remediation plan, if provided below, are legitimate inputs — the
  blindness rule is about the current round.)
- Your final message must be the COMPLETE report in the mandated format, and
  nothing else. It will be filed verbatim as CredentialAcquisition-ReviewState-2.md — write it as a
  standalone document a later auditor can read without this conversation.

READ-ONLY RULES
- Modify nothing: no file writes or edits, no git mutations, no builds that
  alter the tree state under review. Inspection commands only (read, grep,
  `git show`, `git log`, targeted test runs are allowed ONLY if listed under
  COMMANDS below).
- Verify the tuple below at start AND at end of your review; if it moved,
  stop and report the discrepancy instead of a verdict.

EXACT TUPLE (the object under review — nothing else is in scope)
- gwz-dev: 5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9 pinned commit, not concurrently advancing root HEAD
- gwz-credential: 50838c44f8faf4bc6eeb618370ee6ddc10cdd2fc (portable reference/counterexample only)
- Object: whole affected Credential lifecycle/Connection seam with dev-docs/GwzCredentialAcquisitionAmendment.md DRAFT
- Controlling DRAFT document: dev-docs/GwzCredentialAcquisitionAmendment.md at 5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9
- Out of scope: all production implementation (paused); other root/member work, untracked root drafts and evidence collection; review reports/prompts may be filed after review

AUTHORITY AND DEFERRALS
- Process authority: root AGENTS.md/AGENTS_GWZ.md; dev-docs/AgentProcessRules.md as amended by GwzProcessOptimization.md; review-loop skill
- Controlling documents to check the object against: all at root 5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9: dev-docs/GwzCredentialApiDesign.md, GwzConnectionCredentialBoundary.md, GwzConnectionApiContract.md; compare pre-amendment authorities at 0de87d56764792ea70cbe75e5ea0148d3ad7b43c
- Original task authority: Operator: "So fix the acquisition boundary"; "It appears we've underspecified the requirements". Re-derive the complete public boundary from reconciled requirements/provider traces. API changes may be as large as necessary; no production implementation until joint design GO. User asks public repo and first real explicit SSH-key provider plus standalone platform qualification.
- Requested deliverables and scope: design now: distinct source acquisition before pure immutable selection; absolute deadline/cancel, pre-effect bounded admission, terminal source observation and retained cleanup after caller loss, no fake peer/generation, no late publication or replacement-owner budget bypass. First real explicit-file provider follows only accepted design. Editable root exact acquisition docs plus credential member/provider/tests; core/Connection/transport/Git/SSPI/CLI/Python sources and existing supervisors reference-only. No new general supervisor/framework/dependency upgrade or source migration.
- Explicitly deferred (do not report as findings): Production Connection/application integration, other native sources, real SSH network authentication/placement/reuse qualification; these are operator-scoped follow-ups. Do not defer assessment of the full affected lifecycle or existing explicit-key prelookup semantics..
  Deferrals cover a decision's OUTCOME only. Its shape — the verb it lives
  under, its name, whether its lifecycle pair is complete, its defaults — is
  always in scope.
  Only traceable operator decisions are protected: an agent-added prerequisite
  labeled "settled", or an earlier GO, cannot exempt it from the checks below.

COMMON CHECK — PROBLEM, NECESSITY, AND AUTHORIZED SCOPE (every axis)
Complete this check before approving the solution. If assigned the Surface
mandate, read the public interface cold FIRST, then perform this check.
- Identify the requested outcome and concrete observed problem. Trace blocking
  prerequisites and scope-changing constraints to original task authority,
  applicable rules, or demonstrated necessity. Separate preferences and
  assumptions from requirements; a generated plan, test, or prior review is
  not authority for expanded scope.
- Before accepting dependency or infrastructure changes as prerequisites,
  examine a small feasibility probe or concrete evidence about the existing
  supported route. Does the chosen approach fail, or is the actual requested
  outcome impossible through that route? Revisit self-imposed constraints
  without silently relaxing user requirements.
- Does the remedy address the observed problem? Consider supported alternatives
  and localized fixes before accepting broader work. Distinguish a confirmed
  blocker from a limitation, hypothesis, or optimization opportunity. Do not
  demand minimum line count or proof of every routine in-scope engineering choice.
- Verify component-level authorization, including internal changes that keep
  public APIs unchanged. Out-of-scope discoveries belong in separate follow-up
  proposals with observation, evidence, impact, suggested improvement, and
  whether they demonstrably block this outcome; discovery is not permission.
- Record evidence and conclusions in the required report section. A concrete
  unauthorized expansion or unsupported prerequisite that blocks delivery is
  P2, or higher if warranted by impact. If missing authority or evidence prevents
  establishing necessity or scope, identify the specific gap instead of assuming
  a mandate. Optional external shortcomings alone do not block this deliverable.

REVIEW AREAS
- Attack the whole affected Credential lifecycle and Connection seam, not just additions; current portable API is evidence, not authority to preserve.
- Trace reconciled requirements to operator/existing contract; source-before-EVERY-lookup exact-byte identity; helper/native verified-challenge timing; captured native base/caller authority.
- Attack source observation vs actual peer auth observation, snapshot/identity ownership and secret export boundary; absolute controls, no fallback, successful unconsumed result and all Drop stages.
- Assess proposed concrete file worker projection of existing agent_job Entry/Exit ownership: bounded pre-effect reservation, session-owned handle, nonblocking poll, joined cleanup and independent retainer; no hidden new supervisor or unqualified detached worker.
- Check whether proposal is complete enough for first real source implementation; report concrete missing semantics as blockers rather than invent implementation details.

COMMANDS
Working directory /Volumes/projects/limbo/gwz-dev. Read via git show 5dd59da1ea5123e0de850ba4b2409a4ef1b7e1d9:PATH and git -C gwz-credential show 50838c44f8faf4bc6eeb618370ee6ddc10cdd2fc:PATH; inspect core source by read-only commands as supporting reference (record revision). git cat-file/git rev-parse exact immutable commit IDs at start/end. Root HEAD may advance independently; review ONLY pinned objects, not its later state. Allowed focused optional probe: cargo +1.95.0 test --offline --locked --test source_admission in gwz-credential with CARGO_TARGET_DIR external to repos. This is correction round2 of2. Prior reports and merged plan in the pinned member are legitimate inputs: dev-docs/CredentialAcquisition-ReviewCode-1.md, CredentialAcquisition-ReviewState-1.md, CredentialAcquisition-RemPlan-2.md; original reports remain available. Both axes independently found the SAME completed-Future cancel omission (Code P2-2, State P2-1); both classify it local with no new architectural root. Original closing/report omission is closed. Operator explicitly requires same two reviewers for revised joint design; prior State GO does NOT accept new Surface. Inspect whole affected boundary and classify any NEW ARCHITECTURAL ROOT. Add the canonical prior-finding closure table and changed-range analysis before evidence; Each axis must retrace its completed-Future counterexample: borrowed/manual Ready success followed by cancel/Drop/re-poll, then completed error followed by cancel/Drop/re-poll. Truthful source terminal/publication, unique prepared ownership, matching cleanup/charges and no second transfer must hold. Also preserve original stalled-source pre-transfer cancellation, admission failure and caller-loss invariants. Keep same direct-await public shape; no new stages. No current-round peer reports may be read. No production builds or writes; no report file writes. Return full report only. Native logs/source acceptance not the design gate.

SEVERITY AND VERDICT CONTRACT
- Findings use IDs P0-n / P1-n / P2-n / P3-n:
  P0 = active corruption, data loss, credential exposure, or false composition.
  P1 = likely destructive or unrecoverable release blocker.
  P2 = concrete correctness, recovery, compatibility, parity, or
       diagnosability defect.
  P3 = bounded robustness, coverage, maintainability, or documentation defect
       with a concrete consequence.
- Verdict is GO or NO-GO. NO-GO while any P0, P1, or P2 is open.
- Each finding: ONE root cause, exact location, violated invariant, credible
  reproduction or state/interleaving sequence, impact, required correction,
  and a closure/regression test. Separate independent root causes.
- Style preferences and speculative unease are not defects. Do not pad.
  Interface shape is not style: wrong command placement, a misleading name,
  a missing half of a lifecycle pair, or an option without a default is a
  finding (P2 or P3), on every axis.
- If your verdict is NO-GO but every blocking finding has a bounded,
  text-or-code-fixable remedy, you may pre-commit: "I pre-commit to GO on a
  revision that resolves the explicitly named blocking finding IDs as specified." This makes the re-verdict cheap
  and is encouraged when honest.

MANDATED REPORT TEMPLATE
# {OBJECT} — {AXIS}-AXIS REVIEW

**Review object:** {object at exact SHA / doc path + status + date}
**Baseline:** {per-repo SHAs; note how sources were read, e.g. `git show HEAD:`}
**Date:** {date}
**Axis:** {one line: mandate}. Independent, adversarial, read-only. The other
axis runs in parallel; nothing here relies on it. Filed verbatim by the lane
owner.

**Verdict: {GO | NO-GO}** — {counts, e.g. "two P1 and three P2 findings
block"}. {If NO-GO and honest: pre-commit-to-GO clause naming the finding IDs.}

---

## 0. Evidence base
{What was actually read/run: files with line ranges, documents with sections,
commands with results. This section is what makes the verdict auditable.}

## Problem, requirement provenance, and scope
{Required on every axis: requested outcome and original authority; observed
problem; source and necessity evidence for blocking prerequisites or scope
changes (or "none"); supported alternatives / localized remedies considered;
authorized editable components and reference-only dependencies; conclusion and
any specific evidence gaps, linked to finding IDs where blocking. Record
optional external shortcomings separately from current-deliverable blockers.}

## 1. Findings
### [P1-1] {one-line root-cause title}
{Location · violated invariant · reproduction or state sequence · impact ·
remedy · closure test.}
{… one subsection per finding, severity-ordered. Omit section if none.}

## 2. Invariant analysis
{The invariants attacked and the evidence they held — attacks that FAILED are
part of the result; they are what a GO rests on.}

## 3. Risks and next action
{Residual risks below the finding bar; the single next action this verdict
implies.}
