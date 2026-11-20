# Iron Furnaces → Horizon

Ported from **Iron Furnaces 4.3.2** (`ironfurnaces-neoforge-1.21.1-4.3.2.jar`,
SHA-1 `9a40f949ec9794cdef79bdabb61a12a918613194`, SHA-256
`97c0d7c323fda4aa852fdce69bf1250a40323bb9d34ef6c87790e07334d1c2de`,
576,866 B), **Apache-2.0** (the jar's `neoforge.mods.toml`) → **rank 0**.
Minecraft Java 1.21.1 / NeoForge 21.1.265, upstream
`Qelifern/IronFurnacesNeoForge` branch `4.3.2`, commit
`41fde020636b79154e1c454a893962c639d89e1b`. The upstream source was read for
behaviour only; this is a clean-room Rust rewrite. Local-only until publishing
is approved (ARCHITECTURE.md §8).

**Why this mod now:** the next small, self-contained P0 machine after the
dependency-blocked rows (`ae2*` addons need the AE2 native API,
`apothic_enchanting` needs Apothic Attributes, `bigreactors` needs ZeroCore,
`hostilenetworks` needs Placebo, `industrialforegoing` needs Titanium, the MI
addons need Modern Industrialization). It is a **core** mod (no required ATM10
content parent) at licence rank 0.

## Class + tier and the approach

- **Class:** `content`; **tier:** `machines` (`ports/_shared/source-map/
  atm10-sources.csv`, mirroring `ports/ledger.csv`).
- **Approach:** `machines` per `horizon/docs/modpacks.md` — a block entity with
  item slots, a menu, a server tick and the furnace's `lit` state; the content
  half is the carried assets/data and the 18 item/block declarations. This is
  not a data-only carry and not a deep/client slice.

## Scope (slice 1)

The eight furnace tiers with vanilla-only recipes in the jar: **copper, iron,
gold, diamond, emerald, obsidian, crystal, netherite**. Smelting with fuel,
the tier speed, the coal-scaled burn time, the vanilla furnace menu (input /
fuel / output + the flame and arrow data slots), the `lit` state and the
tier-upgrade items. The jar also ships silver, allthemodium, vibranium,
unobtainium and the rainbow furnace; they need other mods' materials and are
deferred (their assets stay in the package, inert).

## Behaviour (from `BlockIronFurnaceTileBase.java` / `Config.java`)

| Area | Upstream | This port |
|---|---|---|
| slots | input 0, fuel 1, output 2 (+ augments 3–5, generator 6, factory 7–18) | 0/1/2 only |
| menu | own container with augment/settings slots | vanilla `minecraft:furnace` menu (`MOD_MENUS`): 3 slots + 4 data slots (flame `data[0]/[1]`, arrow `data[2]/[3]`) |
| base cook ticks (200-tick recipe) | copper 180, iron 160, gold 120, diamond 80, emerald 40, obsidian 20, crystal 40, netherite 5 | same (`Tier::base_ticks`) |
| recipe time scaling | `regular / (200.0 / recipe.getCookingTime())`, `(int) max(1, …)` | same in `f64` then truncating cast (`cook_ticks`) |
| fuel burn | `getBurnTime(fuel, type) * getCookTime() / 200` | same int math (`fuel_burn_ticks`) |
| tick | decrement burn; if not burning + valid recipe + fuel, light and consume one fuel; if burning + valid, `cookTime++`; at total smelt and reset; else reset; a stopped fire cools progress by 2 | same (`runtime::tick`) |
| can smelt | input, recipe, non-empty result, output empty or same item+components with room to `min(maxStack, 64)` | same |
| fuel consumption | shrink one, crafting remainder (lava bucket → bucket), remainder dropped if the slot cannot hold it | same (`consume_fuel`) |
| lit | `furnaceBurnTime > 0`, light 14 | `lit` via `native_set_block_property`; **light 13** (the furnace template), recorded deviation |
| comparator | container fullness over the furnace slots (settings-gated) | the container path's comparator |
| upgrade | `ItemUpgrade(from, to).useOn`: right-click `from`, replace with `to` keeping facing, drop/hand over the contents, copy burn/cook/settings/inventory, consume one unless creative | right-click (`right_click_block`, before the furnace opens its menu) swaps the block and restores the 3 slots + burn/cook/total from the new block entity's first tick; one item consumed in survival |

## Hooks

| Hook (API) | Helper | What it does |
|---|---|---|
| `block_entity` (15) | `block_entity.rs` | The tier block gets a block entity. |
| `block_entity_ticks` (15) | `block_entity_ticks.rs` | It ticks on the server. |
| `block_entity_container` (16) | `block_entity_container.rs` | 3 slots, all faces, inputs 0/1, outputs 2, menu `minecraft:furnace`. |
| `block_entity_tick` (17) | `block_entity_tick.rs` | The smelting/fuel/progress tick, `lit`, and a queued upgrade's restore. |
| `right_click_block` (21) | `right_click_block.rs` | An upgrade item on a compatible furnace swaps the tier before the block's own use opens the menu. |
| `register_commands` (8) | `register.rs` | `/ironfurnaces info <pos>` (the harness readout). |
| `perform_command` (8) | `perform.rs` | The command body. |

## Content (carried from the jar, no invented assets)

- 8 furnace block declarations on the `minecraft:furnace` template with the
  jar's `jovial` and `type` state properties declared (`jovial` 0/1,
  `type` 0/1/2), `lit`/`facing` from the template; per-tier hardness 3.0,
  the source block's explosion resistance/sounds and tool need.
- 10 upgrade items (`upgrade_iron`, `upgrade_gold`, `upgrade_diamond`,
  `upgrade_emerald`, `upgrade_obsidian`, `upgrade_crystal`, `upgrade_netherite`,
  `upgrade_copper`, `upgrade_obsidian2`, `upgrade_iron2`) as `minecraft:stick`
  template items with the jar's own models/textures.
- 10 in-scope recipes (8 tier + `iron_furnace2` + `obsidian_furnace2`) and
  10 upgrade recipes, upgraded to 26.3 by `horizon-port-pack check`.
- 14 loot tables (the 8 in-scope plus inert out-of-scope ones), the
  `mineable/pickaxe` tag, the jar's `c:furnaces/*` item tags and a
  `itemGroup.ironfurnaces` creative tab with 18 entries.
- 9 convention tags the recipes need (`c:ingots/{copper,iron,gold}`,
  `c:gems/{diamond,emerald}`, `c:glass_blocks/colorless`, `c:obsidians`,
  `c:rods/blaze`, `c:storage_blocks/gold`) filled from vanilla items, because
  26.3 ships no `c:` tags.
- Blockstates (all 48 states per block), 93 block models, 44 item models,
  163 block textures, 29 item textures, 16 GUI textures and `en_us.json`.

## Verification

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`** (game.rs
`64813cf4db0b9c3983d42908609bb633b9fbbc705f0c890396f940a6c7cef80f`), mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6). Every build/test/e2e
step ran through `ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`; no
bare `cargo`, no `--target-dir`. Evidence:
`ports/_shared/evidence/ironfurnaces/{check,test,build,e2e,selftest,hmod-check,hmod-load}.log`.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

### Ladder

| Step | Command (via the slot runner) | Result |
|---|---|---|
| runtime TDD (red) | `cargo test --offline --locked` with `todo!()` stubs | **0 passed, 14 failed** (the feature-missing reason), recorded |
| runtime TDD (green) | `cargo test --offline --locked` | **14 passed**, 0 failed |
| fmt / clippy | `rustfmt --edition 2024`, `cargo fmt --check`, `cargo clippy --all-targets --offline --locked -- -D warnings` (the standalone crate) | exit 0, 0 warnings |
| stage + check | `python3 tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** (`check.log`) |
| embedded test | `python3 tools/horizon.py test` | `test result: ok. **14 passed**; 0 failed; 1240 filtered` (`test.log`) |
| fast build | `python3 tools/horizon.py build` | `cargo_exit: 0` (`build.log`) |
| e2e | `build-slot.sh e2e -- … -- tools/e2e.sh` | **PASS**, `EXIT:0` (`e2e.log`) |
| fail-capability | `SELFTEST=1 … tools/e2e.sh` | **EXIT:1** at the netherite assertion (`selftest.log`) |
| load-check | `tools/hmod-load.sh` | **PASS**, `EXIT:0`, with the negative control (`hmod-load.log`) |

### In-game scenario (quoted from `e2e.log`)

```
  info(4,64,-8): ironfurnaces netherite 4 64 -8 burn=29/40 cook=0/5 input=none fuel=none output=minecraft:iron_ingot:1
  info(0,64,-8): ironfurnaces iron 0 64 -8 burn=1258/1280 cook=23/160 input=minecraft:raw_iron:1 fuel=none output=none
  iron cook progress after 12 ticks: 31
  info(0,64,-8): ironfurnaces iron 0 64 -8 burn=1072/1280 cook=0/160 input=none fuel=none output=minecraft:iron_ingot:1
  info(8,64,-8): ironfurnaces iron 8 64 -8 burn=0/0 cook=0/160 input=minecraft:raw_iron:1 fuel=none output=none
  info(12,64,-8): ironfurnaces iron 12 64 -8 burn=0/0 cook=0/0 input=minecraft:diamond:1 fuel=minecraft:coal:1 output=none
  iron ingots in the inventory after the menu withdrawal: 1
  block(16,64,-8) after the upgrade: ironfurnaces:iron_furnace
  info(16,64,-8): ironfurnaces iron 16 64 -8 burn=1402/1440 cook=39/160 input=minecraft:raw_iron:2 fuel=none output=none
  upgraded furnace cook progress: 47
  upgrade items: 1 -> 0
PASS: two tiers smelted (netherite before iron), a coal-scaled fuel burn, the no-fuel and non-smeltable controls stayed idle, the vanilla furnace menu showed its three slots and withdrew the ingot, and a copper furnace upgraded in place to iron keeping its contents
```

The **controls**: the no-fuel furnace (raw iron, no fuel) stays `cook=0/160`
with no output after 40 ticks; the non-smeltable furnace (diamond + coal)
stays `cook=0/0` with no output and keeps its coal (it never lights). The
differential is the netherite tier finishing in 5 ticks while iron (160) is
still cooking at the same tick.

The **fail-capability** run (`SELFTEST=1`) inverts the netherite assertion:

```
FAIL: expected 'output=none' in 'ironfurnaces netherite 4 64 -8 burn=29/40 cook=0/5 input=none fuel=none output=minecraft:iron_ingot:1'
```

**Clippy scope:** clippy ran on the port's standalone crate (the pure
`runtime.rs`), which is the code this port owns as a Cargo library; the injected
`support.rs` and helpers are compiled by the staged `cargo check` with **0
warnings**. Clippy over the whole staged workspace is blocked by pre-existing
pin lints that are not this port's (`horizon-core` names
`clippy::chunks_exact_to_as_chunks`, unknown to the local clippy 1.97;
`horizon-world` trips `manual_range_contains`), so it cannot be used as a
per-port signal here — recorded rather than hidden.

### Artifacts

| Artifact | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/ironfurnaces.hmod` | 244,891 B | `3e1e08c55232af5347a00a01c9d61281197e047206b5b41db7927a578db1fe2b` | 477 |
| `ironfurnaces.hnmod` | 12,335 B | `03576704157e22a76823ef38ddb45f43825b27fdb1e11d5fcdc8dafdb6c8e354` | 11 |

`cargo-horizon-mod check` → `ironfurnaces 4.3.2 (mod, API 0.2, both) …, no code:
content only` (`hmod-check.log`). The load-check places and gives the ids on
the shared `1d63e0a5` game with `--dev-mod content/ironfurnaces.hmod`, and the
same ids are **unknown** without the package (`Unknown block type
'ironfurnaces:iron_furnace'`), so the check is not vacuous.

The scaffold's `tools/test_template.py` is the template's raw-hook fixture: it
needs a workspace-root `native-hooks.json` and the scaffold ships none, so it
rejects a named-hook bundle with `E_HOOK_CATALOG`; `tools/test_stage.py` points
at the non-existent `ports/horizon`. The named-hook ladder used here is
`tools/horizon.py stage → check → test → build` (all green), which is the
campaign's `subagent-brief-template.md` command set. Both stale scripts are
shared with the other ports and are not used by this one.

## Harness addition

`/ironfurnaces info <x> <y> <z>` prints the tier, position, `burn`,
`cook`, and the input/fuel/output slots, e.g.
`ironfurnaces iron 0 64 -8 burn=1258/1280 cook=23/160
input=minecraft:raw_iron:1 fuel=none output=none`. The furnace menu alone
cannot expose progress headlessly, so this is the recorded test command
(precedent `/enderstorage info`). It is registered with
`register_commands`/`perform_command` and sent with `ctl chat "/ironfurnaces
info …"`.

## Gaps / deviations

1. **Eight tiers only.** Silver, allthemodium, vibranium, unobtainium and the
   rainbow furnace need other mods' materials; their assets are inert.
2. **Augments, factory, generator, wireless heater, settings GUI, auto IO,
   redstone modes and XP** are deferred. The furnace branch only.
3. **Light 13 when lit** (the `minecraft:furnace` template) vs upstream 14.
4. **Vanilla-furnace → copper/iron upgrades** are implemented (the source is
   read through the vanilla furnace block entity and its 3 slots restored) but
   the e2e exercises only copper→iron.
5. `jovial`/`type` states are declared and default to 0; the spooky/xmas
   cosmetics and the smoking/blasting augments do not set them.
6. The comparator uses the container path (furnace slot fullness), not
   upstream's settings-gated `getDirectSignal` modes.
7. No custom particles, sounds or animation; the furnace template's are used.
8. The `c:furnaces` tags list the out-of-scope items; the loader drops the
   unknown ids per `docs/mods.md`, so the in-scope tags still resolve.
9. Out-of-scope loot tables (silver/allthemodium/…/heater) are carried but
   have no blocks; they are inert.
10. The result slot is the vanilla furnace menu's result slot; a hopper takes
    from it through `outputs` (0/1 are inputs).

## 2026-11-10 — embedded content; one `.hnmod` is the deliverable

The built content mod (`content/ironfurnaces.hmod`) now travels inside the
bundle at `mixins/ironfurnaces/data/ironfurnaces-content.hmod`, is compiled in
with `horizon_client::native_data!("ironfurnaces", "ironfurnaces-content.hmod")`,
written once to `.horizon-cache/ironfurnaces-embedded-content.hmod` and appended
to the dev-mod list by the `embedded-content` `content_root` injection (HEAD,
cancellable; the AE2 pattern, proved first on `charginggadgets`).
`tools/package.py` packs `data/` and refuses an embedded file that differs from
`content/ironfurnaces.hmod`. One installed `.hnmod` installs the furnaces and the
code; the split pair is no longer a registry blocker.

Artifact: `ironfurnaces.hnmod` 155,236 B, SHA-256
`38c1b9177883f7a23f04c06decdd16a67fed3471e289d4567dc23f5c9b4af617`, 13 entries
(including `mixins/ironfurnaces/data/ironfurnaces-content.hmod`, 244,891 B);
`cargo horizon-mod check ironfurnaces.hnmod --kind native-mod` passes.
`content/ironfurnaces.hmod` stays in the port as the input of that copy, not as
a second deliverable.

In-game proof with no `--dev-mod`:
`ports/_shared/evidence/ironfurnaces/e2e-embedded.log` first runs the control (a
plain `.horizon-cache` file: no furnace item, the block stays air, so the
content provably comes from the bundle), then the full scenario plus a
byte-compare of the written file against `content/ironfurnaces.hmod`; it ends
`PASS` + `EXIT:0`. `e2e-embedded-selftest.log` ends `FAIL` + `EXIT:1`.
`check-e2e-logs.py ironfurnaces --strict` and `upstream_ready.py ironfurnaces`
are both green. Harness fixes this leg needed: `cleanup` now prints the
`EXIT:<status>` line `check-e2e-logs.py` reads (it was missing), plus the
control and embedded-content legs. No assertion was changed.

## Reviewer repairs (2026-11-11) — PR review items and what changed


Each item below is quoted from `ports/_shared/horizon-mods/reviews/pr-*-ironfurnaces.md` (the review is the specification), with what this repair did about it.

- **Reviewer:** licence evidence (entry says Apache-2.0, upstream branch says GPL-3.0)
  **Repair:** `LICENSE-NOTICE.md` records both: the exact jar `ironfurnaces-neoforge-1.21.1-4.3.2.jar` declares `license="Apache License Version 2.0"` (the artefact this port was written against), while upstream's 1.20.1 default branch says GPL-3.0. The port keeps Apache-2.0 and the Apache text.
- **Reviewer:** attribution
  **Repair:** `authors` now includes XenoMustache (the jar names Qelifern (pizzaatime) and XenoMustache); the entry links the absolute upstream URL.
- **Reviewer:** relative link
  **Repair:** Already absolute (`https://github.com/Qelifern/IronFurnaces`).
- **Reviewer:** registry artifact delivery
  **Repair:** **Staging (orchestrator, not ours)**: the artifact is not in registry release/staging as `<sha256>.<ext>`; the orchestrator answers it with `horizon-mods#686` (no write access to the registry's releases). Every repackaged artifact below has a fresh sha256 for that re-push, and the author release must be re-uploaded from the port repository.

Skills: superpowers/using-superpowers,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`: the registry
closed every `.hnmod` submission on 2026-11-18 with one reason, and an `.hnmod` no longer loads.
This port's former submission, **PR #799** (`ironfurnaces` 0.1.0), was closed by that purge.
**0.2.0** is the same behaviour republished as one flat SDK `.hmod`; the changelog text is at the
end of this section.

Pins: SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18, `package_format` 6,
`content_api` "0.2"), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for game 26.3. The
pinned SDK is cached in-tree at `../_shared/sdk/0b14f8a/` and fetched into `.horizon/sdk/`
(gitignored) by the build. The reference recipes were `ports/imfast/notes.md`,
`ports/enderstorage/notes.md` and `ports/naturesaura/notes.md`.

### Done in this order

1. The retired paths were **moved** into `ports/_shared/evidence/ironfurnaces/retired-hnmod-0.1.0/`
   **before the first build**: `mixins/`, `compatibility.json`, `rust-toolchain.toml`, `content/`,
   `.github/`, `ironfurnaces.hnmod`, `tools/{audit,horizon.py,package.py,test_stage.py,test_template.py,stamp_license.py,hmod-load.sh,__pycache__}`;
   `Cargo.lock` was deleted (never packed). The retiree is evidence and stays.
2. `content/assets` → `assets/` and `content/data` → `data/` were copied out of that evidence copy;
   all **476** carried files diff clean against it (`sha256sum` of every file, size included:
   `CMP` empty, `sdk-content-carry` in the evidence folder).
3. `src/runtime.rs` was copied with `cp` and proved with `cmp` (sha256
   `2efd8c89bc21a5a4c6400d47724741c4968101a9acf01c4d009660841736bb29`), and `cargo test` ran on
   the new crate **before any glue existed** (14 tests).
4. The glue was written test-first (`todo!()` first: 2 glue tests failed with "not yet
   implemented", 20 passed), then implemented.

### File map (0.2.0)

| Old `.hnmod` layout | New SDK package |
|---|---|
| `horizon-mixin.toml` (`[mod]`, `[compatibility]`, 2 `[[module]]`, 7 `[[injection]]`) | `horizon-mod.toml` (id/name/version/api/side/capabilities + `[native]`) |
| `Cargo.toml` (`[lib] path = "mixins/ironfurnaces/runtime.rs"`, `[workspace] members = ["tools/audit"]`) | `Cargo.toml` (`[lib] path = "src/lib.rs"`, one dependency: `horizon-mod-api` from `.horizon/sdk/`) |
| `mixins/ironfurnaces/runtime.rs` (the module **and** the crate lib) | `src/runtime.rs` — **byte-identical**, `pub mod runtime;` at the crate root |
| `mixins/ironfurnaces/support.rs` (`ironfurnaces_state`) + 7 hook helper `.rs` files | `src/lib.rs` (the one `impl Hooks`, the `Furnace` Part) + `src/support.rs` (all the glue) |
| `mixins/ironfurnaces/data/ironfurnaces-content.hmod` + `embedded_content.rs` (AE2 pattern) | **gone**: code and content are one package, so there is nothing to embed |
| `content/` (`horizon-mod.toml`, `assets/`, `data/`, `ironfurnaces.hmod`) | package root: `assets/`, `data/` (the content half of the same `.hmod`) |
| `ironfurnaces.hnmod` (12,335 B, 13 entries, embeds the 244,891 B content) | `ironfurnaces.hmod` (276,098 B, 486 entries, native + content) |
| `compatibility.json`, `rust-toolchain.toml` | `[native]` in `horizon-mod.toml`; the pinned commit decides the toolchain |
| `tools/horizon.py`, `tools/package.py`, `tools/test_stage.py`, `tools/test_template.py`, `tools/stamp_license.py`, `tools/audit/`, `tools/hmod-load.sh`, `.github/` | **gone** (moved to evidence): staging, `.hnmod` packaging and the plugin-injection audit belong to the retired format; a native `.hmod` cannot load on an unmodded game, so the content load-check has no subject |
| `Cargo.lock` | not kept (packed → changes the artifact) |

**Kept and updated:** `tools/e2e.sh` (same scenario and assertions, now for a launcher-built game:
`GAME=`, optional `CONTROL_GAME=`, `SELFTEST=1`), `README.md`, `notes.md`, `FINDINGS.md`,
`VALIDATION.md`, `LICENSE`, `LICENSE-NOTICE.md`, `icon.png` (the published 128×128 one).

### Manifest mapping

- identity: `id = "ironfurnaces"` (also the resource namespace), `name`, `version = "0.2.0"`
  (**the port's** version, not upstream's 4.3.2), `authors = ["Qelifern (pizzaatime)",
  "XenoMustache", "AdLf0208"]` (the jar's two plus the port author).
- `api = "0.2"` (exact), `side = "both"` (single player's integrated server belongs to the client
  installation), `capabilities = ["registries"]` (the schema's one value; this package really does
  declare the eight blocks, their block entity type and the ten upgrade items).
- `[native] game = "26.3"`, `horizon = "0b14f8a…"`, `mixins = "e59da90…"` replace
  `[compatibility] horizon_revision` and the bundle's mixins revision — which **moved**
  (`5a64af93…` → `e59da909…`).
- `[mod] priority = 100` and the plan's ordering are gone; nothing here needed ordering.

### Helper → hook table

Each former `[[injection]]` helper and `[[module]]` is mapped; nothing was silently dropped.

| 0.1.0 `[[injection]]` / `[[module]]` (hook, API) | 0.2.0 | Mapping notes |
|---|---|---|
| `[[module]] runtime` (`ironfurnaces`, the Cargo lib) | `src/runtime.rs`, `pub mod runtime;` | byte-identical |
| `[[module]] state` (`ironfurnaces_state`) | `src/support.rs` | all the glue in one place |
| `embedded-content` (`content_root`, AE2 HEAD, cancellable) | **gone** | the flat `.hmod` carries `assets/`+`data/` itself |
| `block-entity` (`block_entity` 15, `block_entity.rs`) | the block entity **type** in `support::declare` (`block_entity_type("furnace").blocks(&[8 bare names]).build()`) | a declared type starts with the empty compound the old hook returned; no `block_entity` Part method is needed |
| `block-entity-ticks` (`block_entity_ticks` 15, `block_entity_ticks.rs`) | `Furnace::block_entity_ticks` → `event.ticks = true` | attached to each of the eight **blocks** (`block_part`), because this hook is keyed by block |
| `block-entity-container` (`block_entity_container` 16/17, `block_entity_container.rs`) | `Registrar::block_container(&block, support::container())` | a declaration, not a hook; per **block**, so eight calls |
| `block-entity-tick` (`block_entity_tick` 17, `block_entity_tick.rs`) | `Furnace::block_entity_tick` → `support::tick` | attached to the **type** (`block_entity_part`); one Part answers all eight tiers and reads the tier from the block |
| `right-click-block` (`right_click_block` 21, `right_click_block.rs`) | `Hooks::right_click_block` → `support::right_click` | pre-block, cancellable: the upgrade wins before the furnace opens its menu |
| `register-commands` (`register_commands` 8, `register.rs`) | `Hooks::register_commands` → `support::declare_commands` | `Command::new("ironfurnaces info <pos:block_pos>")` |
| `perform-command` (`perform_command` 8, `perform.rs`) | `Hooks::run_mod_command` → `support::run_command` | typed `CommandInvocation` (`<pos:block_pos>` → `CommandArgumentValue::BlockPos`), not a split line; the readout text is byte-identical |

`uses.json` in the package is the measured summary: **385 surface entries** — the six hooks the mod
answers (`block_entity`, `block_entity_ticks`, `block_entity_tick`, `right_click_block`,
`register_commands`, `run_mod_command`), `Hooks::register` with `BlockEntityTypeBuilder`,
`Registrar::{block_part, block_entity_part}` and `DefaultCapabilitiesRegistrar::block_container`,
the `horizon:items` + `registries` capabilities, both macros, and the handles this glue uses
(`LevelRef::{block, block_state, capability, recipe_for, set_block, spawn_item}`,
`BlockEntityRef::{data, item_slot}`, `PlayerRef::level`, `HostTransaction::transaction`,
`ItemPlace::PlayerHand`, `ItemStack::{same_item, same_components, set_count}`,
`Host::{item_burn_time, item_crafting_remainder, max_stack_size}`,
`BlockStatesServerHost::set_block_property`). The list is
`evidence/ironfurnaces/sdk-uses-summary.txt`.

### The ladder (all inside one build slot, `CARGO_INCREMENTAL=0`, `BUILD_LOCAL=1`, no `--target-dir`)

There is **no staging step and no staged tree** for an SDK package: the SDK crate is compiled in
place against `.horizon/sdk/`, and `build-slot.sh` is used as the slot guard (its step names speak
the retired tool; here they only set the thread count).

| Step | Command | Result |
|---|---|---|
| runtime only, before glue | `cargo test --offline` | **14 passed**, 0 failed (`sdk-test-runtime.log`) |
| glue red | `cargo test --offline` with `todo!()` glue | **20 passed, 2 failed** — "not yet implemented" (`sdk-test-red.log`) |
| tests | `cargo test --offline` | **22 passed**, 0 failed (14 `runtime::`, 8 `support::`) — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` (never `--all`) | exit 0, empty diff — `sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a … in ./.horizon/sdk`, `Built ./target/horizon-mod/ironfurnaces.hmod` — `sdk-build.log` |
| registry check | `cargo horizon-mod check ironfurnaces.hmod` | `ironfurnaces 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)` / `276098 bytes, SHA-256 f13ed3cd…2194, unsigned`, **EXIT 0** — `sdk-check.log` |
| machine form | `cargo horizon-mod check … --json` | `{"ok":true,…,"kind":"native-mod","sha256":"f13ed3cd…","size":276098,"version":"0.2.0"}` — `sdk-check-json.log` |
| fits the pinned build | `cargo horizon-mod fits ironfurnaces.hmod --build nightly` | `Iron Furnaces:` / `0b14f8a  made for`, **EXIT 0** — `sdk-fits.log` |
| same package twice | `cargo horizon-mod build` again | **276,098 B**, sha256 `f13ed3cd…2194` both times, `cmp` identical — `sdk-build-second.log` |
| `runtime.rs` untouched | `unzip -p ironfurnaces.hmod src/runtime.rs \| sha256sum` | `2efd8c89…bb29` = the retired file and `src/runtime.rs` — `sdk-runtime-sha256.txt` |

The archive is **486 entries**: `horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`,
`README.md`, `icon.png`, `assets/…` (398), `data/…` (78), `src/{lib,runtime,support}.rs`,
`uses.json`. **No `Cargo.lock`** (`sdk-archive.txt`).

### Traps this migration hit (the next port must know)

1. **`cargo test` before the glue, and a deliberate red run.** 14 runtime tests green with only
   `pub mod runtime;` in `lib.rs`; then the two `todo!()`-backed glue tests failed with "not yet
   implemented" (20 passed). Both logs are kept; the red one is evidence, not a mistake.
2. **`Compound::get` takes `&JavaString`, which is not exported.** `nbt.get(key)` does not compile
   (`E0308`); write `nbt.get(&key.into())` (the argument type infers `JavaString`). `insert` takes
   `key.into()`.
3. **A block's id is `level.block(host, pos).map(|key| key.id(host))`.** There is no
   `Host::block_key_id` (that name belongs to the *backend* trait); the `&dyn Host` surface offers
   `BlockKey::id(host)`.
4. **`drop_at` is not a mod's to call** — it lives on the hidden `ItemsBackendMut`. The supported
   route for a block that gives up a remainder is
   `LevelRef::spawn_item(host, x+0.5, y+1.0, z+0.5, stack, 10)` (vanilla's pickup delay for a
   block's drop), which is the same thing the 0.1.0 `game.spawn_item` did.
5. **`host.transaction(..)` needs `use mod_api::HostTransaction`** (it is a trait method; `E0599`
   otherwise).
6. **A block entity tick's `entity.id()` is the block entity *type*, not the block.** The 0.1.0
   glue read the tier straight off `entity.id`; here all eight tiers share one type
   (`ironfurnaces:furnace`), so the tier comes from the block at `event.pos`. The block is still
   in the world during its block entity's tick — only the entity is out.
7. **A vanilla furnace's slots are not readable through `BlockEntityRef::item_slot`** ("the entity
   is vanilla, not a configured mod holder" → `BlockEntityItemError::Unavailable`). The upgrade's
   vanilla source is read through the `ITEMS` capability by the vanilla `WorldlyContainer` faces:
   `Up` reaches the input, the horizontal faces the fuel, `Down` the output.
8. **The upgrade must be `right_click_block`, not `use_item_on`.** `use_item_on` fires *after* the
   block's own use — which for a furnace opens its menu — and never fires for an empty hand.
9. **`set_block_property` is a no-op for an unchanged value**, so `lit` needs no read of the current
   state (0.1.0 read the state's `lit` property first and wrote only on a change). The block entity
   stays, which is what a furnace's own `setLit` relies on.
10. **`block_container` is per block, `block_entity_part` is per type, `block_part` is per block.**
    Eight container declarations and eight `block_part` attachments (the ticks hook is keyed by
    block) against one `block_entity_part` (the tick hook is keyed by type). Getting this wrong is
    silent: the furnace simply never ticks.
11. **Format only `src/lib.rs` and `src/support.rs`** — `rustfmt --edition 2024 <files>` — because
    `cargo fmt` would also rewrite `src/runtime.rs`, which must stay byte-identical; `cmp` after.
12. **`rm -f Cargo.lock` right before each build**, twice, and `cmp` the two artifacts: a packed
    lock changes the sha256 (here `f13ed3cd…2194` both times).
13. **`cargo horizon-mod check` with no file compiles the wasm target.** `check <file.hmod>` for the
    package; `cargo check`/`cargo test` for the native build.
14. **`BUILD_LOCAL=1`.** `build-slot.sh` forwards compile steps to the second machine automatically,
    which has no copy of this port: the first run failed with
    `cd: ports/ironfurnaces: No such file or directory` on the far box.
15. **The 0.1.0 e2e cannot run the new package.** It drives a game built by `tools/horizon.py` with
    the bundle compiled in and a `--dev-mod content` folder; the SDK package is built and run by
    `horizon-launcher native-build`, which is not installed in this environment. `tools/e2e.sh` now
    takes `GAME=`/`CTL=` (and an optional `CONTROL_GAME=`), keeps `SELFTEST=1`, and no 0.2.0 in-game
    leg is claimed.

### Behaviour that changed with the format (deliberate, and why)

- **`mark_modified` is gone.** 0.1.0 inserted the chunk into `game.world.modified` by hand because a
  tick's own writes were not otherwise saved. The SDK's default `BlockEntityTickReply::Changed`
  does exactly that, so the tick writes its compound/slots and leaves the reply alone.
- **The creative check is gone.** 0.1.0 read `game.native_acting_player().game_mode == 1` before
  taking an upgrade item; `ItemPlace::PlayerHand::extract` already gives without losing anything for
  a creative player, so the port takes one item through the documented path and the API decides.
- **One block entity type for eight blocks.** Observably identical (same ids, same states, same
  slots); it is how the SDK spells "these blocks have this entity".
- **The readout is byte-identical** (`ironfurnaces <tier> <x> <y> <z> burn=…/… cook=…/… input=…
  fuel=… output=…`), so the 0.1.0 e2e assertions still match.
- **The menu data slots are still written** (`data[0..4]` = burn, burn total, cook, cook total) for
  the vanilla furnace menu's flame and arrow, exactly as 0.1.0 did.

### Gaps after the migration

Everything in the 0.1.0 feature gaps above still stands (eight tiers only; augments, factory,
generator, settings GUI, auto IO and XP deferred; lit light 13; the comparator is the container
path; no custom particles/sounds; inert out-of-scope assets and loot). Added by this format:

1. **No in-game re-run in this environment** — no `horizon-launcher` here, so no game carries this
   package. The 0.1.0 runs in `../_shared/evidence/ironfurnaces/` (`e2e.log`, `e2e-embedded.log`,
   `selftest.log`, `e2e-embedded-selftest.log`) stand as the behaviour's evidence for a
   `src/runtime.rs` that did not change; what changed is the *transport* of the same calls.
2. **A block's drop has vanilla's own motion and pickup delay.** The remainder of a fuel item that
   is not the last one in its stack is now spawned with `spawn_item` (up and a little either way,
   pickup delay 10); 0.1.0 passed a zero velocity. Not observable in carried content: every vanilla
   fuel with a crafting remainder (buckets) has a max stack of 1, so the branch needs a modded fuel.
3. **No log route.** The SDK exports no logger and a mod may not add a crate, so a refused block
   state or a failed write is silent (the 0.1.0 code had none either).

`## Upstream defects fixed`: none found in this migration. The 0.1.0 defect findings (if any) are
unchanged.

### Changelog for the resubmission (supersedes #799)

> Supersedes **#799**, closed on 2026-11-18 because Horizon retired the `.hnmod` container
> (#16941; PR #17992 cleans up the old files). 0.2.0 is the flat SDK package: `api = "0.2"`,
> `[native]` pinned to nightly `0b14f8acfd97fa2c750e42795249dcc981178ca4` and mixins
> `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, for 26.3. The seven old injections became six typed
> hooks (`block_entity`, `block_entity_ticks`, `block_entity_tick`, `right_click_block`,
> `register_commands`, `run_mod_command`) plus one `Registrar::block_container` declaration per
> tier; the embedded-content injection is gone because one package now carries the code *and* the
> furnaces, their recipes, loot, tags and lang; `src/runtime.rs` and its 14 tests are unchanged,
> byte for byte. Package: 486 entries, **276,098 B**, sha256
> `f13ed3cdd0c9edd4b28f1b4a7560fb0cde93cc9fbc636e0e1741b78f50502194`; `check` and `fits` exit 0.
> No 0.2.0 in-game run: the launcher is not installed here.

### Publication route (campaign ruling 2026-11-18)

The submission form is gone; publishing is now `horizon-rust/horizon-mods#1147` — the artifact is
attached to a release of the author's own repository under the entry's exact `file` name, a PR adds
the `[[versions]]` record and links the file, and a **maintainer** runs `scripts/stage.sh` (a red
`check` before that is the expected `fetch.sh` failure). The orchestrator's
`python3 ports/tools/publish-ready.py ironfurnaces` does the bookkeeping; this job owns no
submission. Still **local-only** until the maintainer approves publishing.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

## Review fixes (2026-11-18) — registry source review #1238 (held)

Skills: superpowers/using-superpowers,systematic-debugging,test-driven-development,verification-before-completion; BeautifulRust/rust-best-practices

File: `var/review-all/1238-ironfurnaces.md` (**held until fixed: this blocked listing**). Package
**0.2.1**, `ironfurnaces.hmod` 276,980 B, sha256
`0b2a1f2b582c401b92930ecd864d74a057f4f3f47be402c93d2e186f4d7187f1`.

1. **`src/support.rs:404`/`:274` an upgrade snapshotted the furnace's inventory and called
   `set_block` with normal removal side effects, so the old container spilled its contents while
   the snapshot was restored into the replacement — every upgrade duplicated its inventory.**
   The replacement now uses `upgrade_flags()`: neighbours and clients, plus
   `SKIP_BLOCK_ENTITY_SIDE_EFFECTS`, which suppresses the removed block entity's spill; the
   snapshot is restored once, by the new block entity's first tick. Pinned by
   `an_upgrade_replaces_the_block_without_spilling_the_old_container` (watched red with the old
   flags) and in game by the upgrade phase asserting zero item entities beside the upgraded
   furnace and the contents present exactly once (`input=minecraft:raw_iron:2`, `fuel=…coal:1`).
2. **`src/support.rs:513` the public coordinate readout exposed another furnace's input, fuel and
   output anywhere in the level.** `info` is declared `Command::op(…, 2)` and its body re-checks
   `may_read(permission)`; a non-furnace still answers the generic `No Iron Furnace there.`
   Pinned by `the_readout_needs_an_operator` and the in-game de-op phase. `cargo test`: **24
   passed**.

**Found by the in-game leg (fixed here, not in the reviewer's list).**

- The Part was registered twice (`block_part` per tier *and* `block_entity_part`, which already
  covers the type's blocks), so the game refused it — the 0.2.0 package could not load. It is
  attached once, through the type.
- Three carried recipes (`furnaces/copper_furnace`, `upgrades/upgrade_copper`,
  `upgrades/upgrade_iron2`) still carried NeoForge load conditions (`neoforge:tag_empty`), which
  the target version refuses as a hard data error. They are rewritten without the conditions: a
  recipe whose tag is empty simply never matches, which is what the condition expressed.

In-game: `PASS` + `E2E EXIT:0` (`ports/_shared/evidence/ironfurnaces-e2e/e2e-launcher.log`), control
`EXIT:1`. The upgrade phase now also asserts **zero item entities** beside the upgraded furnace
(the spill the reviewer found) and the de-op phase is the readout gate.

## Review fixes (2026-11-21) — #1238 second review: 0.2.2

Skills: superpowers/using-superpowers,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

Brief: `var/round-1121/brief-ironfurnaces-0.2.2.md`; review at `1433d37e` (2026-11-20 22:36, `#1238`).
Package **0.2.2**, `ironfurnaces.hmod` **281,821 B**, sha256
`71590264a3b1857466ff1b032856e1990fc2427ab564667e247438662b1250d7`, **487 entries**, no packed
`Cargo.lock`. Pin unchanged (`0b14f8a` + `e59da90`, for 26.3); `src/runtime.rs` byte-identical
(`2efd8c89bc21a5a4c6400d47724741c4968101a9acf01c4d009660841736bb29`, also inside the archive).

### 1. The upgrade's transfer is atomic (reviewer finding 1)

**The bug.** The click snapshotted the old inventory, called `set_block`, and restored the snapshot
from the replacement's first tick. A second upgrade before that tick snapshotted the already-empty
replacement; on tick the first snapshot was taken, its tier no longer matched, and it was
discarded — then the empty one was restored. All three slots lost.

**Mechanism chosen — the contents move with the block, in the handler.** `apply_upgrade` writes
the captured contents into the replacement's block entity **in the same call**, in one
`host.transaction`: `BlockEntityRef::set_item_slot` for the three slots and `merge_data` for
`BurnTime`/`BurnTimeTotal`/`CookTime`/`CookTimeTotal`. The transaction is all-or-nothing, so a
replacement that cannot take them is left untouched. Only when that write fails (no block entity
yet, or a stale handle) are the contents queued for the replacement's first tick — and that
fallback keeps **one entry per position** (`Queued::replace`), carries the *earlier* snapshot
forward (`Queued::peek`), and the tick takes only the entry queued for the block standing there
(`Queued::take_for`), so a tier mismatch can never discard contents. Both halves are
regression-tested; the in-game assertion reads the replacement's slots **inside the same server
call**, which is what proves the atomic path (not the fallback) ran.

**Red, unit (`sdk-test-red-0.2.2.log`), before the fix:**

```
test support::tests::a_second_upgrade_before_the_tick_carries_the_first_snapshot ... FAILED
...
thread 'support::tests::a_second_upgrade_before_the_tick_carries_the_first_snapshot' panicked at src/support.rs:883:9:
assertion `left == right` failed: the second upgrade carried the empty replacement instead of the first snapshot
  left: []
 right: ["minecraft:raw_iron:2", "minecraft:coal:1", "minecraft:iron_ingot:1"]
...
test result: FAILED. 24 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Red, in game, against the pre-fix transfer** (`../_shared/evidence/ironfurnaces-e2e/e2e-launcher-prefix-red.log`;
a scratch copy of this port with today's transfer, the same harness command, two upgrades in one
server call, the same e2e script):

```
  info(20,64,-8) before the two upgrades: ironfurnaces copper 20 64 -8 burn=1250/1440 cook=11/180 input=minecraft:raw_iron:2 fuel=minecraft:coal:1 output=minecraft:iron_ingot:1
  ironfurnaces upgrade 20 64 -8 steps=2 block=ironfurnaces:gold_furnace slots=none,none,none
FAIL: the two upgrades did not move the contents into the replacement in the same operation: ironfurnaces upgrade 20 64 -8 steps=2 block=ironfurnaces:gold_furnace slots=none,none,none
EXIT:1
```

The reply is read inside the upgrade's own call, so `slots=none,none,none` is the defect itself:
the pre-fix transfer had queued the contents for a tick that had not come. (The earlier pre-fix
run of the same copy, before this assertion existed, failed one step later on the post-tick
`info` with `input=none fuel=none output=none` — the same loss seen from the player's side.)

**Green, in game, 0.2.2** (`../_shared/evidence/ironfurnaces-e2e/e2e-launcher.log`):

```
  ironfurnaces upgrade 20 64 -8 steps=2 block=ironfurnaces:gold_furnace slots=minecraft:raw_iron:2,minecraft:coal:1,minecraft:iron_ingot:1
  block(20,64,-8) after two upgrades in one tick: ironfurnaces:gold_furnace
  info(20,64,-8) after two upgrades in one tick: ironfurnaces gold 20 64 -8 burn=1228/1440 cook=33/120 input=minecraft:raw_iron:2 fuel=minecraft:coal:1 output=minecraft:iron_ingot:1
  item entities beside the twice-upgraded furnace: 0
```

The reply line is read inside the upgrade's own server call: the slots are already in the
replacement before any tick, and the burn/cook state moved with them. `SELFTEST=2` inverts that
assertion and the leg ends `EXIT:1` (`e2e-launcher-selftest-upgrades.log`).

### 2. Game mode first (reviewer finding 2)

`right_click` asks the game mode before it reads, replaces or consumes anything:
`may_replace(mode, may_use)` — `Spectator` never replaces; `Adventure` only where
`PlayerRef::may_use_item_at` (the check `ItemStack.useOn` runs) allows the item at that block;
`Survival`/`Creative` always may. The click's face normal is mapped to `BlockFace` for that call.

Red, unit (`sdk-test-red-0.2.2.log`): `assertion failed: !may_replace(GameMode::Spectator, None)`.

Green, in game (`e2e-launcher.log`) — one furnace, one held upgrade item, three modes:

```
  block(24,64,-8) after a spectator's click: ironfurnaces:copper_furnace
  info(24,64,-8): ironfurnaces copper 24 64 -8 burn=0/0 cook=0/180 input=minecraft:raw_iron:1 fuel=none output=none
  a spectator's upgrade item: 1 -> 1
  block(24,64,-8) after an adventure player's click: ironfurnaces:copper_furnace
  block(24,64,-8) after a survival player's click: ironfurnaces:iron_furnace
  a survival player's upgrade item: 1 -> 0
```

The survival click is the differential: the guard is not simply refusing everything. The adventure
click opened the furnace's own menu (`screen: container` in the transcript), which is the game's
own use, not the upgrade.

### 3. Upstream's NOTICE travels in the package (reviewer finding 3)

`NOTICE.txt` at the package root reproduces upstream's own `NOTICE.txt` verbatim (NeoForge,
MinecraftForge, Forge, Actually Additions, Extra Utilities 2, McJty, CoFH, Minecraft) and records
that Apache-2.0 section 4(d) is why it travels; `LICENSE-NOTICE.md` points at it. The SDK packs a
root file whose name starts with `NOTICE` (`horizon-mod-package`'s `source_file`), so it is inside
the archive, not only beside the source (`sdk-archive.txt`):

```
$ unzip -p ironfurnaces.hmod NOTICE.txt | grep -nE 'NeoForge|MinecraftForge|Forge|Actually Additions|Extra Utilities 2|McJty|CoFH|Minecraft'
14:- NeoForge
15:- MinecraftForge
16:- Forge
17:- Actually Additions
18:- Extra Utilities 2
19:- McJty
20:- CoFH
21:- Minecraft
$ unzip -p ironfurnaces.hmod LICENSE | head -2
Apache License
Version 2.0, January 2004
```

The e2e asserts the credits inside the archive before it launches the game (`e2e-launcher.log`
line 5), so a package rebuilt without them fails the leg.

### Harness addition (this slice)

`/ironfurnaces upgrade <pos> <steps>` (operator level 2, beside `/ironfurnaces info`): applies
`steps` upgrades of the jar's chain at one position **inside one server call** and reports the
replacement's slots as they are then. A headless client cannot deliver two clicks in one tick (a
click is a client input answered two ticks later), so this is the only way to exercise the
reviewer's ordering deterministically. It shares `apply_upgrade` with the click, consumes no item,
and `next_tier` follows the jar's registration order (diamond's first upgrade is emerald).

### Ladder (this slice)

| Step | Command (through `build-slot.sh`) | Result |
|---|---|---|
| red | `cargo test --offline` | **24 passed, 2 failed** — the two new tests (`sdk-test-red-0.2.2.log`) |
| tests | `cargo test --offline` | **28 passed, 0 failed** (`sdk-test.log`) |
| fmt | `rustfmt --check src/{lib,support}.rs` + `cargo fmt --check` | exit 0 (`sdk-fmt.log`) |
| clippy | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| build ×2 | `rm -f Cargo.lock && cargo horizon-mod build` (twice) | 281,821 B, `cmp` identical, sha256 `71590264…50d7` (`sdk-build.log`, `sdk-build-second.log`) |
| registry check | `cargo horizon-mod check ironfurnaces.hmod` | `ironfurnaces 0.2.2 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)`, 281821 bytes, `EXIT:0` (`sdk-check.log`, `sdk-check-json.log`) |
| fits, the pin | `cargo horizon-mod fits ironfurnaces.hmod --build 0b14f8a` | `0b14f8a  made for`, `EXIT:0` (`sdk-fits.log`) |
| fits, a different build | `cargo horizon-mod fits ironfurnaces.hmod` (newest nightly) | `0a875da  fits`, `EXIT:0` (`sdk-fits-other.log`) |
| in-game | `launcher-game.sh build` + `build-slot.sh run --step e2e -- tools/e2e.sh` | `PASS` + `EXIT:0` (`e2e-launcher.log`) |
| in-game red | the same leg on the pre-fix transfer | `FAIL` + `EXIT:1` (`e2e-launcher-prefix-red.log`) |
| control | the same leg on the unmodified Nightly | `EXIT:1` (the ids are unknown; `e2e-launcher-control.log`) |
| selftest | `SELFTEST=1` / `SELFTEST=2` | `EXIT:1` each (`e2e-launcher-selftest.log`, `-selftest-upgrades.log`) |
| log audit | `check-e2e-logs.py ironfurnaces-e2e --strict` | `logs=5 ok=5`, exit 0 |
| package audit | `verify-sdk-package.py ironfurnaces` | **1/1** |

### Traps this slice hit

1. **`launcher-game.sh` can export a stale build when two builds share a manifest.** Its
   `built_tag` falls back to the newest installed `native-*` version whose `horizon-mod.toml` is
   byte-identical to the port's, and it does **not** compare `src/`. The pre-fix scratch copy had
   the same manifest, so after the fix `build` exported the pre-fix game and the green leg failed
   with the bug — a real trap, not a flake. Remedy: delete the scratch copy's version tags
   (`rm -rf ~/.cache/horizon-e2e/root/data/horizon-launcher/versions/native-0b14f8a-<hash>`), or
   verify the export directly:
   `grep -c install_contents ~/.cache/horizon-e2e/0b14f8a/ironfurnaces/horizon-native-mods/ironfurnaces/src/support.rs`
   must be non-zero and `let _ = install_contents` must be absent.
2. **`cargo clippy`/`cargo test` write a `Cargo.lock`, and the packer ships a root `Cargo.lock`**
   (`source_file` accepts it). `rm -f Cargo.lock` right before each `cargo horizon-mod build`, and
   leave the port without one; the artifact is 487 entries with none.

### Coverage (parity rule)

| Family (jar) | Result |
|---|---|
| smelting with fuel, tier speed, coal-scaled burn | tested (`e2e-launcher.log`: netherite finishes before iron, iron later, `burn=…/…`) |
| no-fuel and non-smeltable controls | tested (both stay `cook=0/…`, no output) |
| vanilla furnace menu, 3 slots, result withdrawal | tested (`screen=container`, 3 slots, ingot withdrawn to the inventory) |
| tier upgrade by right-click, contents + progress kept, item consumed | tested (`block=iron_furnace`, `input=raw_iron:2`, item 1→0, 0 spilled) |
| two upgrades before a tick | tested (slots read inside the call, post-tick `info`, `SELFTEST=2` inverts it, and the pre-fix run fails) |
| spectator / adventure / survival guard | tested (three modes, survival is the differential) |
| `/ironfurnaces info` operator gate | tested (de-op refusal, then the readout as an operator) |
| third-party credits inside the package | tested (`unzip -p … NOTICE.txt` asserted in the leg) |
| `lit` block state | tested (the block states in every transcript carry `lit=true/false`) |
| carried recipes (8 tier + 10 upgrade + 2 furnaces) | **not exercised in game yet** — carried and validated by `horizon-port-pack check`, and the furnace smelts `minecraft:smelting` in game, but there is no crafting-table leg; next slice |
| comparator output | not tested (the container path is vanilla's) |
| augments, factory, generator, settings GUI, auto IO, redstone modes, XP | not ported (see the 0.1.0 gaps; unchanged) |
| silver / allthemodium / vibranium / unobtainium / rainbow tiers | not ported (need other mods' materials; assets carried inert) |
