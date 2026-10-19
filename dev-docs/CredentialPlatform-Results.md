# Standalone native platform results

Date: 2026-10-19. Source/test/CI checkpoint:
`f6df6be69d54927b3eed6140aa59f92bb2ab13a4`.
Production source equals the accepted portable kernel d6fc64d6. This checkpoint
adds public CI and the controlled-stall source-admission counterexample; no real
source provider is implemented or qualified by these results.

All platforms passed the same pinned gates: cargo +1.95.0 fmt --check;
cargo +1.95.0 test --offline --locked; cargo +1.95.0 clippy --offline --locked
--all-targets -- -D warnings; cargo +1.95.0 doc --offline --locked --no-deps.
The ordinary runner exit status is the gate; no count/inventory pins.

| Native host | Target/filesystem | Result |
| --- | --- | --- |
| Mac | aarch64-apple-darwin, external build target | All four gates PASS |
| dabeest Win11 | x86_64-pc-windows-msvc, isolated E: ReFS checkout | All four gates PASS |
| weftpi Debian13 | aarch64-unknown-linux-gnu, isolated /mnt/work checkout | All four gates PASS |

[Public CI39248862281](https://github.com/owebeeone/gwz-credential/actions/runs/39248862281)
also passed on hosted Ubuntu, macOS and Windows. Public build/test/CI needs only
this repository and its locked dependency; no private evidence or adjacent GWZ
member is needed.

Raw commands, toolchains, logs, initial failed discovery/unpinned attempts and
hashes are in the private
[credential qualification run](https://github.com/owebeeone/gwz-core-evidence/tree/main/campaigns/credential-qualification/runs/2026-10-19-portable-hosting)
(access required). Windows E: ReFS was explicitly selected by the operator;
Get-Volume confirmed it. The Windows shared booking was released by its owner.
No host-wide security, service, trust or agent configuration changed.

The first Mac invocation used the root toolchain through --manifest-path; tests
passed but newer clippy rejected the existing manual no-op waker. That invocation
was not a1.95.0 qualification gate. All Mac gates were then run with explicit
+1.95.0 and passed without source changes. Windows/Linux used the pin explicitly.

Public hosting is complete at
[owebeeone/gwz-credential](https://github.com/owebeeone/gwz-credential).
Accepted history was audited for private-key/token patterns and raw/build paths
before the initial public push. No package publication, release or tag occurred.

These are portable-library/source-counterexample results, distinct from native
file acquisition, SSH authentication, endpoint placement, persistent connection
reuse and CLI/Python qualification. The operator subsequently authorized an
acquisition-boundary redesign; its joint DRAFT is root
`dev-docs/GwzCredentialAcquisitionAmendment.md` at c9640db2. Production changes
remain paused until that revised joint boundary passes review.

## First real explicit-file source candidate

Source3e995edd53d315f639cbfe167fd4562778ee42b9 passes the same four gates
on all three native platforms, explicitly pinned +1.95.0: fmt --check; test
--offline --locked; clippy --offline --locked --all-targets -- -D warnings;
doc --offline --locked --no-deps. This includes the migrated portable suite
and new acquisition/file/native-worker regressions. It is implementation
qualification evidence, pending separate dual settled source review.

| Platform | Isolated source/target location | Result |
| --- | --- | --- |
| Mac aarch64-apple-darwin | member source; external build-targets/credential-source-20261019-3e995edd | PASS all gates |
| Windows x86_64-pc-windows-msvc | E:/gwz-tests/credential-source-20261019-3e995edd/src; E:Healthy/ReFS | PASS all gates |
| Linux weftpi aarch64 | /mnt/work/git/credential-source-20261019-3e995edd/src | PASS all gates |

Both remote checkouts came from the sealed committed source bundle and printed
the exact HEAD before execution. Windows shared booking was atomically acquired
and ownership-checked before release. No broad GWZ build or host security/agent
change. Linux executed actual nonUTF8/newline paths and FIFO replacement; APFS
refused nonUTF8 creation, and its explicit test branch verifies native refusal
without lossy-name fallback. Native container/device policy tables run on every
platform; the Windows native CNG refusal is also exercised by source tests.

Private raw evidence (access required): gwz-core-evidence campaign
credential-qualification/runs/2026-10-19-explicit-source, with exact commands,
exit statuses, toolchains, hashes and failed development-attempt limitations.
This does not prove real SSH cryptography/authentication, production scheduler
composition, endpoint placement, Connection reuse or CLI/Python integration.

Correction1 source29b8838b9fb3bf60cda58ab259e7d85039a2da88 passes all four
ordinary gates on the same three native platform/toolchain classes. It closes
the initial review's provider cutoff and attribution regressions without worker/
parser/dependency changes. Isolated runtime suffix:credential-source-20261020-29b8838b.
Private raw commands/results and red-first regressions are in credential-qualification/
runs/2026-10-20-explicit-source-correction1. Windows booking released after exact
owner check. The separate originating dual review closure is still pending.
