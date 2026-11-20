# Validation

The package is a native SDK `.hmod` (`api = "0.2"`), built against the campaign pin
**Horizon `0b14f8acfd97fa2c750e42795249dcc981178ca4`** + mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, for game **26.3**. Every step below ran through
`ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`; no bare `cargo` build, no `--target-dir`.

Package **0.2.2**: `ironfurnaces.hmod`, **281,821 B**, sha256
`71590264a3b1857466ff1b032856e1990fc2427ab564667e247438662b1250d7`, **487 entries**, no packed
`Cargo.lock`. `src/runtime.rs` is byte-identical to the retired `.hnmod` port's file
(`2efd8c89bc21a5a4c6400d47724741c4968101a9acf01c4d009660841736bb29`), inside the archive too.

| Check | Result | Log |
|---|---|---|
| `cargo test --offline` (red, before the fix) | **24 passed, 2 failed** — the two new tests | `evidence/ironfurnaces/sdk-test-red-0.2.2.log` |
| `cargo test --offline` | **28 passed**, 0 failed | `sdk-test.log` |
| `rustfmt --check src/lib.rs src/support.rs` + `cargo fmt --check` | exit 0, empty diff | `sdk-fmt.log` |
| `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** | `sdk-clippy.log` |
| `rm -f Cargo.lock && cargo horizon-mod build` (twice) | 281,821 B both times, `cmp` identical | `sdk-build.log`, `sdk-build-second.log` |
| `cargo horizon-mod check ironfurnaces.hmod` | `0.2.2 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)`, `EXIT:0` | `sdk-check.log`, `sdk-check-json.log` |
| `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  made for`, `EXIT:0` | `sdk-fits.log` |
| `cargo horizon-mod fits …` (newest nightly, a different build) | `0a875da  fits`, `EXIT:0` | `sdk-fits-other.log` |
| in-game leg (`launcher-game.sh build` + `tools/e2e.sh`) | **PASS** + `EXIT:0` | `evidence/ironfurnaces-e2e/e2e-launcher.log` |
| the same leg on the pre-fix transfer | **FAIL** + `EXIT:1` (two upgrades lost all three slots) | `e2e-launcher-prefix-red.log` |
| the same leg on the unmodified Nightly (`GAME=$CONTROL_GAME`) | `EXIT:1` (the furnace ids are unknown) | `e2e-launcher-control.log` |
| `SELFTEST=1` (netherite inversion) / `SELFTEST=2` (two-upgrade inversion) | `EXIT:1` each | `e2e-launcher-selftest.log`, `e2e-launcher-selftest-upgrades.log` |
| `check-e2e-logs.py ironfurnaces-e2e --strict` | `logs=5 ok=5`, exit 0 | — |
| `verify-sdk-package.py ironfurnaces` | **1/1** | — |
| `check-skills.py --notes ports/ironfurnaces --strict` | PASS | — |

The game is built by `horizon-launcher native-build` (launcher 0.2.78) from the Nightly at the
pin; the scenario runs on a scratch world in The End through `horizon-ctl` (`--offscreen`, scratch
`HORIZON_HOME`/`XDG_*`), and the control leg is the same Nightly without the package. No Mojang
files are stored in the tree; the prepared 26.3 resources are read from
`/home/adlf/Desktop/horizon/.local/horizon-client/26.3`.

`check` and `fits` prove the package's shape and its API surface, not its behaviour: the
behaviour claims come from the in-game leg (see `notes.md` under *Review fixes (2026-11-21)* for
the transcripts and the coverage table).
