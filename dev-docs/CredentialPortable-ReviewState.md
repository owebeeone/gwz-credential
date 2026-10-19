# Credential portable implementation — STATE-AXIS REVIEW

**Review object:** Entire initial portable `gwz-credential` member at `3c342c4a57330f30dc1aeb2bc696251b66162c77`, controlled by `dev-docs/Implementation.md`. Implementation checkpoint; not accepted or integrated.

**Baseline:** Member source read with pinned `git show`. Immutable root authority: `0de87d56`, including the accepted Credential API, Connection API and boundary contract. Live root/builder advancement and all other member implementations were excluded.

**Date:** 2026-10-19  
**Axis:** State — lifecycle legality, cancellation, retained ownership, publication, accounting, scope reuse and provider faults. Independent, adversarial, read-only. Nothing here relies on the parallel reviewer’s report. Filed verbatim by the lane owner.

**Verdict: NO-GO** — four P2 findings block; zero P0 or P1 findings. I pre-commit to GO on a revision resolving P2-1 through P2-4 as specified.

---

## 0. Evidence base

Read all eight `src/*.rs` files and all six integration-test files plus `tests/support/mod.rs`, using the pinned member commit. Also inspected:

- `Cargo.toml`, `Cargo.lock`, toolchain configuration, README and implementation checkpoint.
- Root instructions at immutable authority `0de87d56`.
- Accepted Credential lifecycle, accounting and key-use clauses, particularly §§4–5.
- Accepted boundary terminal/publication rules and Connection retention requirements.

The member HEAD matched at start and end. Tracked source was clean; only the two authorized untracked review prompts were present.

No builds, tests, writes, network operations, mutations or agents were run. The reported passing test/fmt/clippy commands are implementer evidence, not independently executed results.

## Problem, requirement provenance, and scope

The operator authorized isolated implementation of the reviewed portable Credential API and boundary. The requested outcome is independently testable lifecycle/provider machinery, not native providers or production integration.

The implementation stays within the intended dependency boundary: its sole dependency is pinned `zeroize = 1.9.0`; provider work, clocks and retention are injected. No dependency upgrade, new supervisor, network implementation or adjacent source change is needed to resolve these findings.

The accepted design already requires a bridge-visible terminal observation, sticky terminal arbitration, uncertain-key-work retirement and finite retained allocation. The findings concern implementation of those requirements. They do not reopen the accepted designs or demand deferred providers, parsing, cryptography, placement or application migration.

## 1. Findings

### [P2-1] Retained terminal outcomes are inaccessible after failed or lost public observation

**Location:** `src/provider.rs:15–33`; `src/future.rs:12–63,132–193`; `src/values.rs:30`; `src/driver.rs:146–181`.

**Invariant:** A generation-qualified terminal decision must remain observable to the bridge independently of caller polling and final native cleanup.

`RetainedWork` exposes only `advance`. `CleanupReceipt` exposes cleanup status only. An opening failure returns the bare `AuthError` alias, while AuthEnd is available only by polling AuthFinishing.

Consequently:

1. A provider returns a session whose `poll_start` fails.
2. Its `poll_close` remains Pending.
3. AuthOpening returns `Err(AuthError)`, removes its child reference and provides no terminal/cleanup receipt.
4. The retainer continues owning the child, but its public interface cannot inspect the child’s peer/generation, terminal reason or physical disposition.

Likewise, dropping AuthFinishing abandons the only public route to AuthEnd. The internal state survives temporarily, but no cloneable terminal observation reaches the future bridge. When disposal completes, that state can disappear entirely.

A generic retainer cannot reconstruct the missing association from a byte/record charge or aggregate Snapshot. Implementing a production bridge later cannot recover private information absent from the portable interface.

**Impact:** The delivered API cannot implement the accepted immediate matching-generation terminal projection after these paths. Pending cleanup and physical retirement cannot be composed through the promised observation mechanism.

**Correction:** Expose the accepted owned terminal-observation capability before work can fail or public observation can disappear. Preserve bounded terminal facts, peer/generation association, physical disposition and cleanup status independently of the caller future. Keep capability transfer such as ScopeLease separate from cloneable observation.

**Regression:** Obtain the observation before polling an opening; fail startup with cleanup pending, discard the opening, and verify the correct terminal/disposition and pending cleanup remain observable. Repeat after dropping finish/cancel observation and after all ordinary handles disappear. Verify generation isolation and eventual exactly-once settlement.

**NEW ARCHITECTURAL ROOT: YES.** The implementation omits the required retained terminal-observation channel from its public provider/lifecycle boundary. The accepted design supplies the remedy; no broader subsystem is needed.

### [P2-2] Explicit-key errors and panics leave authentication live

**Location:** `src/output.rs:235–255`.

**Invariant:** A failed or uncertain native key transition must seal further authentication and require retirement, while retaining its key/disposal ownership.

`use_ssh_key` catches a port panic and converts it to `Err(ProviderFault)`, but only `Ok(SshProgress::Rejected)` calls `child.cancel`. Other errors return without terminalizing the child.

A concrete trace:

1. A valid explicit-key session invokes a port that returns an error or panics after entering native authentication.
2. `use_ssh_key` returns ProviderFault or the port’s error.
3. The same session invokes `use_ssh_key` again.
4. The second call passes the state checks and re-enters the native port.

The implementation has forgotten the uncertain failure. It can also subsequently accept `finish(Accepted)` without retaining that fault as the retirement reason.

**Impact:** Uncertain authentication can be retried on the same native state, and its terminal disposition can omit the failure.

**Correction:** Make terminal key-port errors sticky under the child authority before returning them. Preserve the in-flight/key-cleanup pin; do not equate error return with disposal. Keep NeedNetwork as the explicit nonterminal outcome.

**Regression:** A fake key port fails or panics on its first call. A second attempt must not invoke it. Terminal observation must retain the fault and Retire disposition, and capacity must remain held until the key-disposal probe confirms.

**NEW ARCHITECTURAL ROOT: NO.** Local terminal-transition omission.

### [P2-3] Provider shutdown replaces an already fixed terminal error

**Location:** `src/state.rs:180–185`; `src/future.rs:150–156`.

**Invariant:** The first recorded terminal reason wins; later shutdown may revoke scope or change physical eligibility without rewriting that reason.

Both `Child::error` and AuthFinishing prefer `authority.closed` over the child’s existing terminal reason.

Reproduction:

1. `poll_respond` fails with ProviderRejected or ProviderFault.
2. The driver records that terminal reason, and the response future reports it.
3. CredentialOwner closes.
4. The session’s terminal future now reports ProviderClosed instead.

The same conversation therefore exposes inconsistent terminal causes. The internal `terminate` first-wins rule does not protect the public projection.

**Impact:** Shutdown masks the actual authentication/provider failure, weakening diagnostics and terminal provenance.

**Correction:** Preserve the fixed terminal reason when projecting errors and AuthEnd. Represent subsequent provider revocation separately where needed for scope invalidation and Retire disposition.

**Regression:** Fix each terminal error, then close/drop the operation and provider before consuming terminal observation. The original cause and peer fact must remain stable; required revocation and retirement must still occur.

**NEW ARCHITECTURAL ROOT: NO.** Local terminal-projection defect.

### [P2-4] Stored selection metadata bypasses retained-byte limits

**Location:** `src/api.rs:120–155`; `src/values.rs:99–113`; `src/provider.rs:41–49`.

**Invariant:** All library-retained selection allocation must count against operation, owner and shared-retention limits.

Selection admission charges only `provider.selection_charge`. It then stores the caller-supplied `ExchangeSelection` in `Selected.choice`. Validation checks string lengths, while equality with the allowed selection ignores allocation capacity.

Using the existing fake provider:

1. Configure the operation/owner byte limits to 64 KiB.
2. Admit an ordinary explicit-key policy.
3. Supply an equal selection whose key-path String contains the same short path but has 128 KiB capacity.
4. Validation and policy equality pass.
5. The provider declares 64 bytes, and the library retains the oversized String while charging only those 64 bytes.

The provider’s own allocation can remain entirely within its declaration; the uncharged object is stored by the library.

**Impact:** Actual retained memory can exceed the configured bounds without InputBudget refusal or corresponding shared-retention charge.

**Correction:** Include kernel-owned selection metadata capacities and structural storage in the charge, separately from the provider’s declared additional allocation. Alternatively, move into bounded normalized storage before retaining it.

**Regression:** Use equal selections with inflated capacities for target and explicit-key strings. Admission must refuse or normalize them within the configured budget, with balanced accounting after every failure/drop path.

**NEW ARCHITECTURAL ROOT: NO.** Local charge omission.

## 2. Invariant analysis

Several attacks held:

- Owner and operation close synchronously set revocation under the authority mutex; a prepared selection cannot register effectful work afterward.
- Provider callbacks run outside authority/payload locks. The driving guard prevents concurrent stepping of one child.
- Pending provider cleanup and key-disposal probes retain registrations. Recreating an owner cannot evade the supplied shared retention budget.
- Output commitment checks peer context and owner/session state. Already committed output remains separately charged until sent or discarded.
- Key snapshots remain selected across NeedNetwork re-entry, and owner closure during an admitted key call prevents late success publication while retaining the pin.
- Scope reuse checks both callers’ full identity comparison, allowed policy and complete peer/generation equality; it does not rely solely on the identity hint.
- Absolute deadlines are retained across rounds, and the retention-port contract requires periodic progress when providers do not wake.
- Completed record retirement avoids a host-wide tombstone history.

The tests cover these paths usefully. They do not cover the missing terminal observation, key-error re-entry, competing terminal causes or retained selection capacity described above.

## 3. Risks and next action

Native providers, real bridge behavior and platform qualification remain correctly excluded. Passing portable tests cannot establish those outcomes.

**Next action:** correct the four portable findings in one bounded patch, add the specified regressions and rerun the focused member gates. Return the settled member revision for independent closure; no adjacent implementation or design amendment is required.
