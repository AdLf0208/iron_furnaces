# Iron Furnaces — a Horizon port

**Iron Furnaces** is a Minecraft mod. A Mod for Minecraft which adds new furnaces that have different speeds. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [Qelifern/IronFurnaces](https://github.com/Qelifern/IronFurnaces) — licence **Apache-2.0**.

| | |
|---|---|
| Upstream mod | Iron Furnaces ([source](https://github.com/Qelifern/IronFurnaces)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `ironfurnaces.hmod` (install it with the Horizon launcher) |
| Licence | **Apache-2.0** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Reviewer:** licence evidence (entry says Apache-2.0, upstream branch says GPL-3.0)
- **Reviewer:** attribution
- **Reviewer:** relative link
- **Reviewer:** registry artifact delivery
- **`mark_modified` is gone.** 0.1.0 inserted the chunk into `game.world.modified` by hand because a
- **The creative check is gone.** 0.1.0 read `game.native_acting_player().game_mode == 1` before
- **One block entity type for twelve blocks.** Observably identical (same ids, same states, same
- **The readout is byte-identical** (`ironfurnaces <tier> <x> <y> <z> burn=…/… cook=…/… input=…

## Silver and the modded tiers

Four furnaces beyond the jar's vanilla-only eight, each with the upstream speed and upgrades:

| Furnace | Ticks per smelt | Crafted from | Upgrades |
| --- | --- | --- | --- |
| Silver | 140 | `c:ingots/silver` on an iron or copper furnace | copper → silver, iron → silver, silver → gold |
| Allthemodium | 5 | allthemodium or stellarite ingots and blocks on a netherite furnace | netherite → allthemodium |
| Vibranium | 3 | vibranium or infernium on an allthemodium furnace | allthemodium → vibranium |
| Unobtainium | 1 | unobtainium or voidium on a vibranium furnace | vibranium → unobtainium |

They need another mod's metals. Silver comes from any mod that fills
`c:ingots/silver`, such as [Vast Ores](https://github.com/voltzz900/vast-ores).
The modded tiers take Allthemodium's metals through `c:` tags, or
[Vast Depths](https://github.com/voltzz900/vast-ores)' stellarite, infernium and
voidium, which fill the same three steps; the `ironfurnaces:<tier>_ingots` and
`ironfurnaces:<tier>_corners` item tags list both. With neither mod, the recipes
are skipped and the furnaces are only reachable by command.

The jar gates these recipes with `neoforge:tag_empty`; Horizon leaves a file
with an unknown condition out, so the port drops the gate and relies on a
recipe with an empty tag being skipped when it loads.

## Not yet ported

- **No rainbow furnace.** It needs every other tier and is left out; its assets stay inert.
- **Augments, factory, generator, wireless heater, settings GUI, auto IO,
- **Light 13 when lit** (the `minecraft:furnace` template) vs upstream 14.
- **Vanilla-furnace → copper/iron upgrades** are implemented (the source is
- `jovial`/`type` states are declared and default to 0; the spooky/xmas
- The comparator uses the container path (furnace slot fullness), not
- No custom particles, sounds or animation; the furnace template's are used.
- The `c:furnaces` tags list the out-of-scope items; the loader drops the

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `ironfurnaces.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Iron Furnaces was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **Apache-2.0**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/Qelifern/IronFurnaces

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run ironfurnaces` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/ironfurnaces/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
