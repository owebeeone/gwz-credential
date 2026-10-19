# Acquisition design — first Surface correction

Date: 2026-10-19. Original reviewed tuple: root c9640db2/member f6df6be6.
Code NO-GO on P2-1 (undefined AcquisitionClosing report); State GO.
Reviewer classification: zero new architectural roots. Production remains paused.

| Input | Disposition and closure |
| --- | --- |
| Code P2-1 | Remove undefined AcquisitionClosing. CredentialAcquiring is explicitly Future<Output=Result<PreparedCredential,AuthError>>; cancel consumes it and immediately yields defined AcquisitionEnd with original error and optional matching cleanup receipt/status. Complete examples cover admission refusal, pending cancel/report, transfer and last-pin Drop. Code must trace original stalled-source sequence on the new public surface. |
| Operator API usability objection | Collapse outer Result/handle.wait/Acquired/context.select into ONE directly awaited acquisition returning PreparedCredential. Immediate admission failure is a ready failed Future with no record/effect. Pure immutable selection remains internal and guarded. No requirement to preserve old spelling. |
| Whole affected boundary recheck | Both axes recheck all old ownership/timing/accounting/security arguments at one new exact tuple; old State GO is not new-shape acceptance. Explicit-key prelookup, verified-request binding, replacement-owner limits and independent retained cleanup remain unchanged. |

This is one merged correction, not a staged production patch. No production API
or provider source has been implemented. Reviewer closure is required before
implementation. Same two reviewers are retained as explicitly directed by the
operator/root lane owner; Surface remains folded into Code. This is correction
round1 of maximum2; any new architectural root is classified by reviewers.
