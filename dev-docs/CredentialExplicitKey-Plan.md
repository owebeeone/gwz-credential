# First explicit-key provider and native qualification

Date: 2026-10-19. Status: first real source implemented; native qualification and implementation review pending.

Operator authorization: create public `owebeeone/gwz-credential`, qualify the
standalone library on Mac, Windows and Linux, and implement the first actual
endpoint-local explicit SSH file-key source against the accepted portable ports.
The operator explicitly selected E: ReFS for Windows runtime checkouts, overriding
older evidence replay examples using D:. No crates.io publication, tags or release.

Editable components: gwz-credential source, focused product tests, CI and member
documents; GWZ-managed remote/member metadata; scoped raw qualification evidence
in private gwz-core-evidence. Core, transport, Git, Connection, CLI, Python, SSPI
and their platform workers are reference-only. No dependency upgrades, new
supervisor/carrier/general signer or application integration are authorized.

Baseline: gwz-credential main b3809e2264c696099ca02903fcd5f1f900578853, clean.
Accepted portable production source/tests d6fc64d6a38457018377020765229405920a4833.
Controlling API/boundary/Connection documents are pinned at root
0de87d56764792ea70cbe75e5ea0148d3ad7b43c. Dependency: zeroize =1.9.0;
Rust 1.95.0. Root builder and other members are concurrent unrelated work.

## Bounded sequence

1. Check source admission feasibility before implementation. Preserve native
   paths, one bounded zeroized snapshot, exact-byte identity, explicit source
   refusal/no fallback, and native key-pin disposal. Source lookup is distinct
   from native SSH authentication. Existing core algorithms are reference only.
2. Audit all intended public history/tree for secrets and raw private evidence;
   create the exact public repository, attach origin through supported GWZ
   mechanics and synchronize metadata; push only this member.
3. Run independent Mac, Windows (gianni@dabeest, E:/gwz-tests/unique directory)
   and Linux (host identity from existing records) standalone gates. Raw logs
   belong in the private evidence campaign. No broad GWZ build.
4. At the resulting settled boundary run the review-loop skill with two
   peer-blind Code/State reviewers, Surface folded into Code, generated prompts,
   immutable tuple and verbatim reports. Maximum two remediation rounds.
5. Record exact scope/results and stop. Connection composition is a separate
   handoff. Source-provider/fake tests do not prove actual native SSH auth,
   endpoint placement, persistent reuse or installed CLI/Python behavior.

If source admission cannot fit existing ports, record a small concrete
counterexample and explicit contract question. Do not silently weaken ownership,
retention or prompt-return constraints or start a redesign.

Initial ceilings: 1,000 production lines, 600 test/CI lines, 12 changed product
files, zero wire/protocol changes and zero new production dependencies. No
production files have been changed. Development-document caps: 75 active files
per repository and 2,500 lines per document; archive intermediates unchanged at
stable checkpoints.

## Operator scope amendment and current checkpoint

The operator subsequently directed "So fix the acquisition boundary" and
identified underspecified requirements. The complete affected Credential
lifecycle/Connection seam is now re-derived in root
`dev-docs/GwzCredentialAcquisitionAmendment.md` at c9640db2. Exact acquisition
clauses in the three root contracts are editable; all application/Connection/
core/CLI/Python/SSPI sources and existing supervisors remain reference-only.
Do not preserve a wrong signature merely to minimize the patch. Capture, source
acquisition, immutable selection, peer auth and reusable scope are distinct;
explicit-key exact bytes must exist before EVERY pool lookup. Helper/native
sources preserve valid verified-origin/challenge timing. Production implementation
is paused pending joint Code/State GO on the WHOLE affected boundary.

Public hosting and native portable qualification are complete separately; see
CredentialPlatform-Results.md. This does not fulfill the first-real-provider
implementation deliverable, which resumes only after the amended boundary passes.

## Read-only SDAX fit note

Operator reference: https://github.com/owebeeone/sdax-rs. Inspected local
sdax-rs ccf06e76a90e22a454471a71f0cf6f5cb878baac after reading its workspace
and member AGENTS. No SDAX edits, builds, spike, dependency change or adoption.
Evidence: docs/Running.md; crates/sdax-tokio/src/running.rs (Future/Drop/output);
crates/sdax-tokio/src/driver.rs (deadline/blocking dispatch); crate manifests;
crates/sdax/src/plan.rs and report.rs. This is a source-based fit assessment,
not executed provider-composition or performance evidence.

SDAX's typed resource/step/service lifecycle, reverse release, custom Running
Future with cancel/handle/snapshot, tracked drainer, bounded blocking execution
and simulation/testkit are relevant substrate candidates. They do not themselves
supply Credential's policy/publication/secret-ownership contract.

| Required Credential property | Read-only fit / outstanding proof |
| --- | --- |
| Blocked native read remains charged/owned through actual exit | Running.md says shutdown budget can abandon a blocking node while its native thread continues. A run report therefore cannot free Credential shared capacity, source bytes or native key pins. Any adapter needs actual native-exit/join evidence retained beyond that report; no drop-in equivalence established. |
| Progress after all caller futures disappear | Running Drop uses one tracked drainer, but current_thread has no progress between block_on calls. A permanently driven runtime/retainer is necessary; merely adopting Running is insufficient. Existing Credential independent bounded periodic progress remains mandatory. |
| Original absolute deadline | SDAX driver propagates engine deadlines; the bridge would have to map the original clock/deadline without starting fresh durations at run/node/retry boundaries. No such mapping has been implemented/tested. |
| Unique prepared result and checked publication | Running yields Report<Out> whose output is Arc<Out>; Credential prepared authority is unique, guarded and non-Clone. An internal once-transfer cell could adapt this without copying secret bytes, but must prove only one live prepared capability and no post-cancel transfer. Arc output is not automatically that proof. |
| Shared limits across owner replacement | Plan/run/blocking bounds are relevant, but Credential needs one shared retention reservation surviving abandoned native work and replacement owners. Rebuilding a Runtime/Plan/provider must not reset that ledger; no bridge proof exists. |
| Cheap isolated tests | Pure sdax is std-only; executing sdax-tokio adds pinned Tokio1.53.1 and Tokio-util0.7.19. Testkit/simulation could support deterministic faults, but adoption/build-cost effects and provider/kernel/native test separation have not been measured here. |

Conclusion: potentially useful internal lifecycle substrate; no demonstrated
requirement to adopt it and no claim that its shutdown report means native
disposal. Its directly awaited custom Future is also a useful API precedent.
A dependency/adoption spike would require separate authorization. Current
acquisition Surface review may finish without adding SDAX or a runtime.

## Accepted redesign and implementation resumption

Revision2 of root GwzCredentialAcquisitionAmendment.md is accepted GO/GO at
5dd59da1 after two bounded corrections, with zero architectural roots and no
open P0–P3. See CredentialAcquisition-Verdict.md and the final verbatim reports.
The original provider stop is superseded by this operator-authorized design.

Resume this member's acquisition API and first explicit-file provider only.
The implementation budget is revised for the operator's complete boundary
redesign: 2,400 new/modified production lines, 1,800 test/CI lines, 22 product
files, zero wire changes. The earlier 1,000-line ceiling described the initially
assumed provider-only path; it is not an operator requirement to preserve the
underspecified API. Existing lifecycle mechanisms may be reused where correct;
no new general supervisor/runtime is introduced. The first parser will reuse
core's bounded container-validation algorithm and exact already-used
base64=0.22.1 rather than invent another decoder; this is one focused production
dependency addition with no version upgrade. Zeroize remains pinned=1.9.0.

Work test-first against the declared directly awaited Future, source-specific
terminal observation, native worker join and exact snapshot identity. Keep native
file acquisition tests distinct from fake kernel tests and actual SSH auth.
Settle/review the implemented boundary and rerun focused native gates before
claiming provider completion. Application integration and SDAX adoption remain
outside scope. Archive closed intermediate design reviews/prompts/plans unchanged
at the provider checkpoint; final design provenance stays active.

## Implementation candidate

The direct acquisition Future and first explicit-file source are implemented
against accepted root5dd59da1. The original failing acquisition and file-provider
compile probes were observed before their production interfaces existed. Existing
portable tests were migrated without deleting the10k-operation/500-schedule
coverage. Source failure reports contain finite sanitized variants, never paths
or key bytes. One fixed2MiB native stack is explicitly charged in each source
work reservation; no owner replacement resets the shared ledger.

Pinned Rust1.95.0 Mac test/fmt/clippy/doc and the disabled-branch cfg boundary
guard pass locally. Actual blocked native work, worker panic/join, stored-result
versus exit and clock callbacks outside kernel locks have focused regressions.
Native new-source Windows/Linux and settled dual implementation review remain
pending; the earlier platform results qualify only the older portable commit.

The Unix filename fixture initially assumed APFS could create arbitrary non-UTF8
names. APFS returned EILSEQ. The corrected Mac branch verifies native refusal
and no lossy-name fallback; Linux must exercise actual non-UTF8 creation/read.
This is a fixture assumption correction, not a key-path conversion in production.
