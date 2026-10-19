# Acquisition design — completed Future closure

Date: 2026-10-19. Reviewed revision1: root dc7d6bbd/member15b68ced.
Both axes returned NO-GO on the SAME completed-Future cancellation omission:
Code P2-2 and State P2-1. Original Code P2-1 remains closed. Both reviewers
classify this as a local public-state omission, not a new architectural root.
This is the strongest blind-convergence signal in this design loop.

| Finding | One merged correction / closure trace |
| --- | --- |
| Code P2-2, State P2-1 | AcquisitionEnd truthfully represents fixed source outcome: Option<AuthError>, material_published and matching optional cleanup receipt. Before transfer cancel still records failure. After Ready(Ok(prepared)), cancel is observation only: success/publication remain, prepared remains unique/live, no native/material pin release. Completed Err preserves the original error; completed Drop has no new transition. Explicit Unpin permits the borrowed-await witness; observation remains stable and re-poll returns only WrongState without changing original terminal facts or transferring twice. |

Revised design includes manual/borrowed Ready-success→cancel example and exact
success/error cancel/Drop/re-poll closure traces. Same two reviewers verify their
original counterexamples at one settled tuple and reassess changed-range/scope.
No extra stages, result wrappers, supervisor, runtime, dependency or source work.
Production remains paused until both axes return GO. Correction round2 of2;
no architectural root has been established. Reviewers classify any new root.
