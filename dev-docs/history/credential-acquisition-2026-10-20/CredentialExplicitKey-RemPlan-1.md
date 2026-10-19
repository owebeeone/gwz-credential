# Explicit-file source implementation correction 1

Date:2026-10-20. Initial object root a80515f3/member e4a080b3. Both independent
axes returned NO-GO and independently found the same P2 and P3. Reports are
filed verbatim. Zero architectural roots; this is correction1 of maximum2 for
this implementation gate, separate from the earlier completed design gate.

| Finding | Root and bounded correction | Closure evidence |
| --- | --- | --- |
| Code/State P2-1 | Source controls were checked by the kernel but the later begin deadline was forwarded unchanged. Normalize Child.control.deadline_ms to min(source deadline, begin deadline) before admission/provider work. Preserve both cancellation checks and retained cleanup. | Fake selected material records actual start deadline. Source50/begin1000 and source1000/begin50 both forward50; time50 prevents output and retains Pending cleanup until provider disposal. Observe failing assertion before production correction. |
| Code/State P3-1 | Source observation has acquisition ID but no public operation attribution. Expose existing namespace-qualified operation ID through a read-only accessor; no registry/owner/authority addition. | Retainer-only observations group A1/A2 versus B1 with otherwise identical facts, preserve distinct acquisition IDs and remain stable after caller loss/cleanup. Observe missing accessor compile failure first. |

Editable only member API/observation/tests/support and checkpoint/evidence docs.
No contract change, new abstraction, dependency or native worker change. Rerun
focused affected tests plus ordinary full native library gates on all3 platforms
at the committed correction, using new isolated dirs and the same pin/booking
rules. Return canonical generated prompts and exact tuple to the same two peers;
prior reports/this merged plan are legitimate inputs, current round remains blind.
