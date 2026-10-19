# First explicit-key provider and native qualification

Date: 2026-10-19. Status: feasibility investigation; no provider acceptance.

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
