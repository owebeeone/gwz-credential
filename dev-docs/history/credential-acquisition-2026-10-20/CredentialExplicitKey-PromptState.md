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
  nothing else. It will be filed verbatim as CredentialExplicitKey-ReviewState.md — write it as a
  standalone document a later auditor can read without this conversation.

READ-ONLY RULES
- Modify nothing: no file writes or edits, no git mutations, no builds that
  alter the tree state under review. Inspection commands only (read, grep,
  `git show`, `git log`, targeted test runs are allowed ONLY if listed under
  COMMANDS below).
- Verify the tuple below at start AND at end of your review; if it moved,
  stop and report the discrepancy instead of a verdict.

EXACT TUPLE (the object under review — nothing else is in scope)
- gwz-dev: a80515f364194b04e38c4a6d59597cee0d8347cc pinned root authority; concurrent root HEAD excluded
- gwz-credential: e4a080b3d99a4ab98478d0b06c8d0d104e4180a0 (production/test candidate3e995edd53d315f639cbfe167fd4562778ee42b9 plus qualification docs)
- private evidence: cd4acf0af1aa662356680bcb8a649c5b5e518e23
- Object: controlled acquisition API and first endpoint-local explicit SSH file-key source; separate implementation gate after joint design GO/GO
- Controlling DRAFT document: gwz-credential/dev-docs/Implementation.md and CredentialExplicitKey-Plan.md at e4a080b3d99a4ab98478d0b06c8d0d104e4180a0
- Out of scope: all unrelated root/member/source work and root untracked drafts. Prompts/reports filed after review are not production deltas.

AUTHORITY AND DEFERRALS
- Process authority: root AGENTS.md/AGENTS_GWZ.md, EVIDENCE.md, dev-docs/AgentProcessRules.md amended by GwzProcessOptimization.md; review-loop skill; private evidence AGENTS only if reading logs
- Controlling documents to check the object against: root a80515f364194b04e38c4a6d59597cee0d8347cc: dev-docs/GwzCredentialAcquisitionAmendment.md (accepted design at5dd59da1), GwzCredentialApiDesign.md, GwzConnectionCredentialBoundary.md, GwzConnectionApiContract.md. Member CredentialAcquisition-Verdict.md and final ReviewCode-2/ReviewState-2 are design-only authority/evidence, not this implementation acceptance.
- Original task authority: Operator requested separate agent, PUBLIC owebeeone/gwz-credential hosting, independent native Mac/Windows/Linux qualification and first real explicit SSH key source. Subsequently: "So fix the acquisition boundary"; reconcile underspecified requirements and redesign whole affected lifecycle; simplify public API to directly awaited acquisition; same two independent reviewers. Root delegated current bounded implementation after design GO/GO.
- Requested deliverables and scope: Editable member library/provider/tests/CI/docs; exact acquisition amendments in root contracts already accepted; GWZ-managed own metadata and scoped private evidence. Core/Connection/transport/Git/SSPI/CLI/Python sources/existing supervisors are reference-only. No dependency upgrades, general supervisor/runtime/carrier/signer or broader migration. Existing core algorithms referenced narrowly at461ddf4bc6892373a00b600da9778efb08ffc5d6. base64=0.22.1 added as exact existing core pin for bounded container decoding, zeroize1.9.0 unchanged; assess necessity/scope. SDAX read-only fit note does NOT authorize adoption.
- Explicitly deferred (do not report as findings): Other source providers (agent/helper/native), HTTP offer parsing, real native SSH adapter/cryptographic authentication, Connection/application integration and endpoint placement/reuse qualification. Passing source/fake/kernel tests must not be represented as those gates..
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
- Review whole affected Credential lifecycle against accepted joint boundary, not only new files. Trace original caller/policy capture, source-specific timing, exact bytes before EVERY explicit-key lookup, pure internal selection, actual-peer authentication and scope reuse separately. No fake peer/generation for source jobs.
- Attack pre-effect shared admission/reservations, provider/source callback lock boundaries, absolute deadline/cancellation, private readiness vs guarded publication, completed Future cancel/Drop/re-poll and unique Prepared authority. Check original controls survive acquisition/VerifiedRequest and cannot be extended at begin.
- Attack retained file worker: native JoinHandle+private result owned by registered source session; fixed charged stack/buffer; stored result != exit; actual join before publication/disposal; blocked/panic/error cleanup and lost caller/owner; source and auth share64-record allowance; replacement owners cannot bypass retained native work. Independent retainer periodic progress remains required.
- Attack snapshot/identity/observation last-pin accounting and zeroization-before-release; source cleanup versus surviving material pins; same exact native key snapshot over NeedNetwork and physical disposal. Opaque identity equality is not checkout authority.
- Attack native filenames/captured relative base/no ambient recapture or fallback, regular-file/race admission, encrypted/multiple/malformed/oversize/CNG/device refusals and bounded container decode. Source framing tests do NOT prove cryptographic validity. Mac APFS refusal/no lossy fallback and Linux actual nonUTF8 tests differ truthfully.
- Compare tests/provider conformance traces with design permanent cases; generic kernel pass cannot stand for real provider composability. Public CI/product tests cannot depend on private evidence or adjacent core. Assess requirement provenance, scope, files/budgets and no unsupported infrastructure prerequisite.

COMMANDS
Working directory /Volumes/projects/limbo/gwz-dev. Review pinned Git objects at start/end (git cat-file/git rev-parse). Read member through git -C gwz-credential show e4a080b3d99a4ab98478d0b06c8d0d104e4180a0:PATH; root through git show a80515f364194b04e38c4a6d59597cee0d8347cc:PATH. Diff member648aae24e6ba00e27b65a56314739a67ee1ccc33..e4a080b3d99a4ab98478d0b06c8d0d104e4180a0. Production/tests unchanged after3e995edd; source currently clean. Optional focused tests with cargo +1.95.0 --offline --locked and external CARGO_TARGET_DIR; do not edit, mutate Git or run broad GWZ. Qualification: fmt/test/clippy(-Dwarnings)/doc passed pinned1.95 on native MacARM64, WindowsMSVC E:Healthy/ReFS, Linuxweftpi aarch64. Exact source3e995edd, commands/results/private raw logs at evidence cd4acf0af1aa662356680bcb8a649c5b5e518e23:campaigns/credential-qualification/runs/2026-10-19-explicit-source; public summary CredentialPlatform-Results.md. These gates are native SOURCE/library tests, not native SSH/Connection acceptance. New implementation object, initial round0 (max2 correction rounds); same two reserved reviewers, no third reviewer/helper. Previous design reports may be read; no current implementation peer report may be seen. Classify NEW ARCHITECTURAL ROOT explicitly. Return COMPLETE report only, no report-file writes. Assess code/test evidence independently and disclose any unexecuted claim.

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
