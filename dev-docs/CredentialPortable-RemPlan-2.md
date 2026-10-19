# Portable Credential remediation — round 2

Previous member: edf0d776df11917d47567194e1061d0e9c50c4ab.
Code: original findings closed, P2-5 remains. State: GO, all original findings
closed. Architectural roots remain one; no new architecture/design issue.

Code P2-5: add the separately allocated fixed PeerContext payload and an explicit
128-byte allowance for both Arc headers/alignment/bookkeeping to the observation
reservation, using checked arithmetic. Keep that whole charge through the last
observation or cleanup receipt. No lifecycle, provider or public API changes.

Closure: retain completed observations after conversation retirement; assert
remaining owner/shared charges cover surviving payload/metadata, accumulate to
budget refusal, keep a cleanup-receipt clone, then verify balanced final release.
Run focused member test/fmt/clippy/doc gates and return the settled revision to
the same two reviewers. This is the second and final planned remediation round;
no additional implementation programme is authorized.
