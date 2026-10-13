# Local Photoshop analysis environment

Verified on 2026-10-13 on the development Mac:

| Component | Observation |
| --- | --- |
| Reference application | Adobe Photoshop 2026, 27.8.0 |
| Application artifact | `/Applications/Adobe Photoshop 2026/Adobe Photoshop 2026.app` |
| Node | 24.19.0 |
| pnpm | 11.21.0 |
| Java | Amazon Corretto 21, ARM64 |
| REA | 6.4.0, acquired with pnpm |
| Analysis engine | Existing Hopper 5.19.5-demo, in an application directory named v4 |
| Codex registration | Setup applied; scoped setup readiness reports healthy |
| Hopper readiness | Scoped provider doctor reports healthy |
| Rust | cargo 1.99.0; baseline workspace test suite passes |

Setup preserved unrelated Codex configuration and created
`~/.codex/config.toml.rea.backup`. The MCP entry launches the locally acquired REA
script directly, with the detected `HOPPER_LAUNCHER_PATH`. The matching
`reverse-engineer-anything` skill is installed under `~/.agents/skills/`.
No additional model credential, database, or web development server is required.

Codex must restart/reconnect to expose newly registered MCP tools. File readiness
does not prove a live connection or successful native analysis of Photoshop.
Use the CLI for bounded operations while the current chat lacks those tools.
Hopper's first-run/demo/license dialog must be completed if it appears. Its demo
limits remain relevant; deep analysis of the full Photoshop binary is not yet
verified. Ghidra and IDA have not been configured as redundant providers.

Commands (run from the repository):

```sh
python3 scripts/run-bounded.py --timeout 90 -- pnpm dlx rea-agents@6.4.0 doctor --client codex --json
python3 scripts/run-bounded.py --timeout 90 -- pnpm dlx rea-agents@6.4.0 doctor --provider hopper --json
python3 scripts/run-bounded.py --timeout 600 -- cargo test --workspace
```

Photoshop UI observations use the native Codex computer-use interface. Reference
experiments use disposable documents and must preserve the user's open document.
OpenPhoto is a Rust native application; its UI verification uses egui_kittest and
native app interaction. The browser plugin is relevant only if a web surface is
introduced. Development app processes are stopped after verification, with their
PIDs recorded. No server or listening port is needed for the native application.

The current native UI interface can read Photoshop's accessibility tree, capture
its window and operate AX controls/keyboard shortcuts. Coordinate clicks fail
with `-10005 noWindowsAvailable`; raising the reference window did not fix that
failure. A working coordinate-input route is required before certifying live
Photoshop drag parity. REA's native UI scenario command requires an exact PID
and window ID and exposes AX actions, so it has not been treated as an equivalent
unverified substitute for pixel-based drag input.
