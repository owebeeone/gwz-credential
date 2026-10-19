# Portable Credential remediation — round 1

Object: 3c342c4a57330f30dc1aeb2bc696251b66162c77. Both axes NO-GO.
One architectural root, independently found by both: the implementation omitted
bridge-visible retained terminal observation. The accepted contracts already
require it; no design amendment or adjacent implementation is authorized.

| Finding | Disposition and regression |
| --- | --- |
| Code P2-1 / State P2-1 | Add charged, generation-qualified cloneable observation before effects and on RetainedWork. Lost/failed opening and finish/cancel Drop remain observable; receipt has no release authority. Test through public fake bridge only, including pending cleanup and generation isolation. |
| Code P2-2 | Recheck cancellation/deadline before pending scope transfer. Preserve Accepted separately; cancellation wins -> no scope/Retire; transfer wins -> ordinary operation close does not revoke transferred authority. |
| Code P2-3 / State P2-4 | Charge kernel metadata/string/path capacities plus fixed storage independently of declared provider allocations. Test inflated equal values and zero-charge provider sustainability/refusal. |
| Code P2-4 | Native PathBuf source, bounded native allocation, no lossy conversion/control-character ban. Enclosed Unix module tests non-UTF-8 and newline names without filesystem I/O. |
| State P2-2 | Key-port Err/panic seals child and prevents re-entry while disposal probe/pin remains retained. Test both faults, sticky observation and pending capacity. |
| State P2-3 | First nonempty terminal reason remains fixed through later operation/provider shutdown. Separate provider revocation from recorded cause. Test competing causes and preserved peer fact. |

One member-only patch; rerun standalone test/fmt/clippy/doc gates, then same two
reviewers re-trace original counterexamples at settled SHA. Root builder may
advance independently; immutable authority remains 0de87d56. Maximum two
remediation rounds; no implementation expansion into providers or integration.
