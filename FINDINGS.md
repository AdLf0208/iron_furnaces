# Iron Furnaces — upstream findings

Ported from **Iron Furnaces 4.3.2** (`ironfurnaces-neoforge-1.21.1-4.3.2.jar`,
SHA-1 `9a40f949ec9794cdef79bdabb61a12a918613194`, SHA-256
`97c0d7c323fda4aa852fdce69bf1250a40323bb9d34ef6c87790e07334d1c2de`,
576,866 B), Apache-2.0 → **rank 0**. Minecraft 1.21.1 / NeoForge 21.1.265.

- Pack record (Crash Assistant `modlist.json`, ATM-10 `main`):
  `{"jarName":"ironfurnaces-neoforge-1.21.1-4.3.2.jar","modId":"ironfurnaces",
  "name":"Iron Furnaces","version":"4.3.2","curseForgeHash":1526878118,
  "modrinthHash":"9a40f949ec9794cdef79bdabb61a12a918613194"}`. The jar was
  fetched from Modrinth
  (`https://cdn.modrinth.com/data/yPlaLxD1/versions/WZ25JeYB/ironfurnaces-neoforge-1.21.1-4.3.2.jar`)
  and its SHA-1 matches the pack record.
- Upstream source: **`Qelifern/IronFurnacesNeoForge`**, branch `4.3.2`
  (commit `41fde020636b79154e1c454a893962c639d89e1b`), `MC 1.21.1`,
  `mod_version=4.3.2`. The older `Qelifern/IronFurnaces` repo only goes to
  1.20.1 and was used for nothing. Read from
  `ports/_shared/upstream/ironfurnaces-neoforge` (analysis only).

## Scope for this port (slice 1)

The eight furnace tiers that have vanilla-only recipes in the jar:
**copper, iron, gold, diamond, emerald, obsidian, crystal, netherite**.
The jar also ships `silver`, `allthemodium`, `vibranium`, `unobtainium` and
`million` (rainbow) furnaces; those need other mods' materials and are deferred
(assets stay in the jar, inert).

Machine shape: block entity, 3 item slots (input/fuel/output), vanilla
`minecraft:furnace` menu (3 slots + 4 data slots), smelting with fuel, tier
speed, lit state, and the tier-upgrade items. Augments (3 extra slots, factory,
generator), the wireless heater, the settings GUI, auto IO, XP and the
spooky/xmas cosmetics are deferred.

## Behaviour read from upstream (`src/main/java/ironfurnaces/`)

`tileentity/furnaces/BlockIronFurnaceTileBase.java`, `Config.java`,
`blocks/furnaces/BlockIronFurnaceBase.java`,
`items/upgrades/ItemUpgrade.java`, the tier tiles and `init/Registration.java`.

| Area | Upstream | This port |
|---|---|---|
| slots | `INPUT=0`, `FUEL=1`, `OUTPUT=2`, augments 3–5, generator 6, factory 7–18 | 0/1/2 only |
| menu | own container (augments + settings); flame `data[0]`/`data[1]`, arrow `data[2]`/`data[3]` | vanilla `minecraft:furnace` menu, 3 slots, same four data slots |
| base cook time | `Config.<tier>FurnaceSpeed` (200-tick recipe): copper 180, iron 160, gold 120, diamond 80, emerald 40, obsidian 20, crystal 40, netherite 5 | same constants |
| recipe time scaling | `getSpeed = regular / (200.0 / recipe.getCookingTime())`, `max(1, …)`, integer truncation | `max(1, base * recipe_time / 200)`, integer truncation |
| fuel burn | `furnaceBurnTime = getBurnTime(fuel, recipeType) * getCookTime() / 200` | `fuel_burn * cook_ticks / 200`, integer truncation |
| tick | decrement burn; if not burning and a valid recipe + fuel, light and consume one fuel; if burning and valid, `cookTime++`; at `totalCookTime` smelt and reset; else `cookTime = 0`; when not burning and `cookTime > 0`, `clamp(cookTime - 2, 0, total)` | same |
| can smelt | input non-empty, recipe present, result non-empty, output empty or same item + components with room up to `min(maxStack, 64)` | same |
| fuel | `EventHooks.getItemBurnTime` (vanilla burn time), crafting remainder (a lava bucket → empty bucket) | `Providers::burn_time` + crafting remainder |
| lit | `lit = furnaceBurnTime > 0`; light emission 14 when lit | `lit` via `native_set_block_property`; template light 13 (deviation) |
| comparator | container fullness over slots 0/1/2 (settings-gated) | the container path's comparator |
| break | drops contents + stored-recipe XP | contents drop (container path); XP deferred |
| upgrade | `ItemUpgrade(from, to)`: right-click on `from`, replace with `to` (keeping facing), drop/hand-over the contents, **copy burn/cook/settings/inventory** to the new block entity, consume 1 unless creative | our tiers: same, preserving items + burn/cook; vanilla-furnace source deferred |
| upgrade chains | iron: furnace→iron; iron2: copper→iron; copper: furnace→copper; gold: iron→gold; gold2: silver→gold; diamond: gold→diamond; emerald: diamond→emerald; obsidian: emerald→obsidian; obsidian2: crystal→obsidian; crystal: diamond→crystal; netherite: obsidian→netherite | the 8 chain items whose both ends are in scope: iron, iron2, copper, gold, diamond, emerald, obsidian, obsidian2, crystal, netherite (10 items; gold2/silver are out of scope) |
| config defaults | tier 0/1/2 energy 80k/200k/1M, generation values; not relevant to a fuel furnace | not carried |

### Tier table (upstream defaults)

| id | base cook ticks (200-tick recipe) | tier |
|---|---:|---:|
| `copper_furnace` | 180 | 0 |
| `iron_furnace` | 160 | 0 |
| `gold_furnace` | 120 | 1 |
| `diamond_furnace` | 80 | 2 |
| `emerald_furnace` | 40 | 2 |
| `obsidian_furnace` | 20 | 2 |
| `crystal_furnace` | 40 | 2 |
| `netherite_furnace` | 5 | 2 |

So the e2e's "faster tier smelts sooner" claim is exact: e.g. a 200-tick
vanilla smelting recipe takes 160 ticks on iron and 5 on netherite.

## Data carried from the jar (no invented assets)

- 14 blockstates / 93 block models / 44 item models / 163 block textures /
  29 item textures / 16 GUI textures / `lang/en_us.json` (only the keys the
  slice uses are kept; the rest stay inert).
- 8 block loot tables (self-drop, `copy_components` custom name), 8+10 recipes,
  1 advancement, 1 item tag (`netherite_upgrade_crafting`).
- Recipes use NeoForge `c:*` tags; 26.3 ships none, so the port supplies the
  tags its recipes reference from vanilla items/blocks.

## Known differences / gaps

- Only 8 tiers; silver/allthemodium/vibranium/unobtainium/million deferred.
- Augments, factory, generator, wireless heater, auto IO, settings GUI,
  redstone modes and XP are deferred.
- Vanilla-furnace → copper/iron upgrades are not usable (the vanilla furnace's
  entity slots are not exposed by a named hook); upgrades among the port's own
  tiers are.
- Light 13 when lit (furnace template) vs upstream 14.
- Menu title is `block.ironfurnaces.<tier>` (the container name); the furnace
  template's `open_menu` path supplies it.
