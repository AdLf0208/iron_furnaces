# Iron Furnaces → Horizon

Iron Furnaces **4.3.2** (`ironfurnaces-neoforge-1.21.1-4.3.2.jar`, Apache-2.0)
ported to Horizon as **one native SDK package** (`.hmod`, mod API `0.2`), version **0.2.0**.
Pins: SDK nightly `0b14f8acfd97fa2c750e42795249dcc981178ca4`, horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, game 26.3.

The eight fuel furnaces — copper, iron, gold, diamond, emerald, obsidian,
crystal, netherite — smelt with fuel at their upstream speed, use the vanilla
furnace menu (input / fuel / output), show the lit state, and upgrade in place
with the tier-upgrade items. `notes.md` has the behaviour table, the hooks, the
deviations and the verification (`## Migration to the SDK format` is the 0.2.0
recipe and its traps); `FINDINGS.md` has the upstream analysis;
`ports/_shared/carry-work/ironfurnaces/SOURCE.md` has the jar provenance.

## Layout

- `horizon-mod.toml` — the only manifest and identity: `id`, `version = "0.2.0"`,
  `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native]` pins.
- `Cargo.toml` — one crate; its only dependency is `horizon-mod-api` from the fetched
  `.horizon/sdk/` (never packed).
- `src/lib.rs` — the one `#[mod_api::horizon_mod] impl Hooks` and the `Furnace` Part.
- `src/support.rs` — the game glue: the block entity type, the three-slot container, the tick,
  the upgrade click and `/ironfurnaces info`.
- `src/runtime.rs` — the pure smelting/fuel/upgrade rules, byte-identical to the 0.1.0 file.
- `assets/`, `data/` — the carried content (the jar's assets, the 26.3 data, the block/item
  declarations, the `c:` tags the recipes need).
- `tools/e2e.sh` — the fail-capable in-game scenario (The End arena), for a launcher-built game.
- `ironfurnaces.hmod` — the delivered package.

The retired `.hnmod` layout (`mixins/`, `content/`, `compatibility.json`, `tools/horizon.py`,
`tools/package.py`, the audit crate, the old `.hnmod`) is kept as evidence under
`../_shared/evidence/ironfurnaces/retired-hnmod-0.1.0/`; it is not part of this package.

## Commands

Every step runs through the campaign slot registry, from the campaign root:

```sh
ports/tools/build-slot.sh run --job ironfurnaces-sdk --port ports/ironfurnaces --step test -- \
    bash -c 'cd ports/ironfurnaces && CARGO_INCREMENTAL=0 cargo test --offline'
ports/tools/build-slot.sh run --job ironfurnaces-sdk --port ports/ironfurnaces --step build -- \
    bash -c 'cd ports/ironfurnaces && rm -f Cargo.lock && cargo horizon-mod build'
cargo horizon-mod check ironfurnaces.hmod        # from the port
cargo horizon-mod fits ironfurnaces.hmod --build nightly
```

Use `BUILD_LOCAL=1`: the port is not synced to the remote build box.

The harness readout is `/ironfurnaces info <x> <y> <z>` (send it with
`ctl chat "/ironfurnaces info …"`).

## Licence

Apache-2.0. The port is **local-only** until the maintainer approves publishing
(ARCHITECTURE.md §8).
