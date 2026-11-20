//! Game-side glue for Iron Furnaces (SDK format).
//!
//! [`crate::runtime`] is the pure truth: the tier table, the cook and burn scaling and the
//! furnace tick, with its tests. This module is the only place that reads the game and writes it
//! back: the block entity declaration, its three-slot container, one server tick, the tier-upgrade
//! click and the `/ironfurnaces info` readout.
//!
//! The retired `.hnmod`'s `support.rs` and its eight per-hook helper files are folded in here; the
//! mapping is in `notes.md` under `## Migration to the SDK format`.

use std::sync::{Mutex, OnceLock};

use mod_api::{
    BlockEntityRef, BlockEntityTickEvent, BlockUpdateFlags, Command, CommandArgumentValue,
    CommandInvocation, Compound, Direction, Flow, Hand, Host, HostSide, HostTransaction, ITEMS,
    Identifier, InteractionResult, ItemPlace, ItemResource, ItemStack, LevelRef, ModContainer,
    RecipeInput, RecipeMatch, Registrar, RunModCommandEvent, ServerHost, Tag, Text,
};

use crate::runtime::{FUEL, INPUT, OUTPUT, SLOTS, State, Tick, Tier, upgrade_target};

/// The vanilla `furnace` class every tier block uses as its template, and the menu its container
/// shows.
const VANILLA_FURNACE: &str = "minecraft:furnace";
/// The two items of `AbstractFurnaceBlockEntity`'s wet-sponge branch (input and fuel).
const WET_SPONGE: &str = "minecraft:wet_sponge";
const BUCKET: &str = "minecraft:bucket";
/// The bucket a wet sponge leaves in the fuel slot.
const WATER_BUCKET: &str = "minecraft:water_bucket";

/// What a queued upgrade must restore into the new tier's block entity after the block swap: its
/// contents and its smelting state.
struct Pending {
    target: &'static str,
    items: Vec<Option<ItemStack>>,
    state: State,
}

/// Upgrades waiting for their new block entity's first tick, by dimension and position: the block
/// entity a player's hook writes is not authoritative, so the restore runs from the block entity's
/// own tick.
static PENDING: Mutex<Vec<(String, [i32; 3], Pending)>> = Mutex::new(Vec::new());

fn pending() -> std::sync::MutexGuard<'static, Vec<(String, [i32; 3], Pending)>> {
    PENDING.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Take the upgrade queued for one dimension and position, if any: the first tick of the new
/// block entity is the only place the restore is authoritative.
fn take_pending(
    queue: &mut Vec<(String, [i32; 3], Pending)>,
    dimension: &str,
    pos: [i32; 3],
) -> Option<Pending> {
    let index = queue
        .iter()
        .position(|(dim, at, _)| dim == dimension && *at == pos)?;
    Some(queue.remove(index).2)
}

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

/// Declare what the game answers for the eight blocks: one block entity type holding all of them,
/// the Part that ticks it, and the three-slot furnace container on every tier.
pub fn declare(r: &mut dyn Registrar) {
    let names = bare_names();
    let kind = r.block_entity_type("furnace").blocks(&names).build();
    for tier in Tier::ALL {
        let block = r.block_key(tier.block_id());
        r.block_container(&block, container());
    }
    // `block_entity_part` attaches the Part to every block of the type, so a `block_part`
    // per block here would attach it twice and the game refuses a duplicate
    // ("attached twice, by ironfurnaces and ironfurnaces").
    r.block_entity_part(&kind, crate::Furnace)
        .expect("the furnaces' Part attaches to the type this mod just declared");
}

/// The block ids' bare names, as `block_entity_type(..).blocks(..)` takes them: the type's id is
/// this mod's, so a bare name means `<mod id>:<name>`.
fn bare_names() -> Vec<&'static str> {
    Tier::ALL
        .iter()
        .map(|tier| {
            let id = tier.block_id();
            id.split_once(':').map_or(id, |(_, path)| path)
        })
        .collect()
}

/// Input, fuel and output, reachable from every face, shown as the vanilla furnace menu.
fn container() -> ModContainer {
    let all = vec![INPUT, FUEL, OUTPUT];
    ModContainer {
        slots: SLOTS,
        faces: [
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all.clone(),
            all,
        ],
        inputs: vec![INPUT, FUEL],
        outputs: vec![OUTPUT],
        menu: Some(VANILLA_FURNACE.to_owned()),
    }
}

// ---------------------------------------------------------------------------
// The saved state
// ---------------------------------------------------------------------------

fn int(nbt: &Compound, key: &str) -> i32 {
    match nbt.get(&key.into()) {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    }
}

fn set_int(nbt: &mut Compound, key: &str, value: i32) {
    nbt.insert(key.into(), Tag::Int(value));
}

/// The saved furnace state (`BurnTime`, `BurnTimeTotal`, `CookTime`, `CookTimeTotal`).
fn read_state(nbt: &Compound) -> State {
    State {
        burn: int(nbt, "BurnTime"),
        burn_total: int(nbt, "BurnTimeTotal"),
        progress: int(nbt, "CookTime"),
        total: int(nbt, "CookTimeTotal"),
    }
}

fn write_state(nbt: &mut Compound, state: State) {
    set_int(nbt, "BurnTime", state.burn);
    set_int(nbt, "BurnTimeTotal", state.burn_total);
    set_int(nbt, "CookTime", state.progress);
    set_int(nbt, "CookTimeTotal", state.total);
}

// ---------------------------------------------------------------------------
// The tick
// ---------------------------------------------------------------------------

/// One server tick of a tier furnace: apply a queued upgrade, then run the pure smelting rules
/// against the game's own smelting recipes and fuel values.
pub fn tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let pos = event.pos;
    let dimension = host.dimension();
    let Some(level) = host.level(dimension) else {
        return;
    };
    // The container declares three slots and the furnace menu four data slots; a block entity made
    // before a slot count changed is padded rather than indexed out of bounds.
    if event.entity.items.len() < SLOTS {
        event.entity.items.resize(SLOTS, None);
    }
    if event.entity.data.len() < 4 {
        event.entity.data.resize(4, 0);
    }
    {
        let entity = &mut *event.entity;
        apply_pending(
            &mut *host,
            level,
            dimension,
            pos,
            &mut entity.nbt,
            &mut entity.items,
        );
    }
    let Some(tier) = block_id(&*host, level, pos)
        .as_deref()
        .and_then(Tier::from_block)
    else {
        return;
    };
    let before = read_state(&event.entity.nbt);

    let recipe = event
        .entity
        .items
        .get(INPUT)
        .and_then(|slot| slot.as_ref())
        .and_then(|stack| smelting(&*host, level, stack));
    let recipe_time = recipe.as_ref().map(|matched| matched.processing_time);
    let result = recipe.as_ref().map(|matched| matched.output.clone());

    let stored = event
        .entity
        .items
        .get(OUTPUT)
        .and_then(|slot| slot.as_ref());
    let output_ok = match (&result, stored) {
        (Some(_), None) => true,
        (Some(result), Some(stored)) => {
            let max = host.max_stack_size(result.item()).unwrap_or(64).min(64);
            stored.same_item(result)
                && stored.same_components(result)
                && stored.count().saturating_add(result.count()) <= max
        }
        (None, _) => false,
    };
    let fuel_time = event
        .entity
        .items
        .get(FUEL)
        .and_then(|slot| slot.as_ref())
        .and_then(|fuel| host.item_burn_time(fuel.item()))
        .map_or(0, |ticks| ticks.max(0) as u32);
    let input_present = event
        .entity
        .items
        .get(INPUT)
        .is_some_and(|slot| slot.as_ref().is_some_and(|stack| stack.count() > 0));

    let mut state = before;
    let step = crate::runtime::tick(
        &mut state,
        Tick {
            base_ticks: tier.base_ticks(),
            has_input: input_present,
            recipe_time,
            fuel_time,
            output_ok,
        },
    );

    if step.consume_fuel {
        consume_fuel(&mut *host, level, pos, &mut event.entity.items);
    }
    if step.smelt
        && let Some(result) = &result
    {
        smelt_one(&mut *host, &mut event.entity.items, result);
    }

    write_state(&mut event.entity.nbt, state);
    // The vanilla furnace menu's data slots: the flame's two and the arrow's two.
    event.entity.data[0] = state.burn;
    event.entity.data[1] = state.burn_total;
    event.entity.data[2] = state.progress;
    event.entity.data[3] = state.total;
    sync_lit(&mut *host, pos, state.is_burning());
}

/// The smelting recipe for `stack`, through the game's own matcher (a data pack's smelting recipe
/// included).
fn smelting(host: &dyn Host, level: LevelRef, stack: &ItemStack) -> Option<RecipeMatch> {
    static SMELTING: OnceLock<Option<Identifier>> = OnceLock::new();
    let kind = SMELTING
        .get_or_init(|| Identifier::parse("minecraft:smelting").ok())
        .as_ref()?;
    let input = RecipeInput::Single(stack.clone());
    level.recipe_for(host, kind, &input).ok().flatten()
}

/// Restore a queued upgrade into the new tier's block entity, when it is that tier.
fn apply_pending(
    host: &mut dyn ServerHost,
    level: LevelRef,
    dimension: &str,
    pos: [i32; 3],
    nbt: &mut Compound,
    items: &mut Vec<Option<ItemStack>>,
) {
    let Some(upgrade) = take_pending(&mut pending(), dimension, pos) else {
        return;
    };
    if block_id(&*host, level, pos).as_deref() != Some(upgrade.target) {
        return;
    }
    *items = upgrade.items;
    if items.len() < SLOTS {
        items.resize(SLOTS, None);
    }
    write_state(nbt, upgrade.state);
}

/// The block id at `pos`, or none outside a loaded block.
fn block_id(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Option<String> {
    level.block(host, pos).map(|key| key.id(host))
}

/// `Item.getCraftingRemainder`: take one fuel item and keep what it leaves (a lava bucket leaves
/// an empty bucket), dropping the remainder when the slot cannot hold it.
fn consume_fuel(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    items: &mut [Option<ItemStack>],
) {
    let Some(fuel) = items.get(FUEL).and_then(|slot| slot.clone()) else {
        return;
    };
    let remainder = host.item_crafting_remainder(&fuel);
    let Some(slot) = items.get_mut(FUEL) else {
        return;
    };
    let left = slot
        .as_ref()
        .map_or(0, |stack| stack.count().saturating_sub(1));
    if left == 0 {
        *slot = remainder;
    } else if let Some(mut stack) = slot.clone()
        && stack.set_count(left).is_ok()
    {
        *slot = Some(stack);
        if let Some(remainder) = remainder {
            let _ = level.spawn_item(
                &mut *host,
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + 1.0,
                f64::from(pos[2]) + 0.5,
                remainder,
                10,
            );
        }
    }
}

/// One item into the result, the wet-sponge rule beside it (`AbstractFurnaceBlockEntity` empties a
/// bucket of water over a wet sponge in the fuel slot).
fn smelt_one(host: &mut dyn ServerHost, items: &mut [Option<ItemStack>], result: &ItemStack) {
    let wet_sponge = items
        .get(INPUT)
        .and_then(|slot| slot.as_ref())
        .is_some_and(|stack| stack.item().id(&*host) == WET_SPONGE);
    let bucket = items
        .get(FUEL)
        .and_then(|slot| slot.as_ref())
        .is_some_and(|stack| stack.item().id(&*host) == BUCKET);
    if wet_sponge
        && bucket
        && let Some(key) = item_key(&*host, WATER_BUCKET)
    {
        items[FUEL] = Some(ItemStack::of(key));
    }
    match items.get_mut(OUTPUT).and_then(|slot| slot.as_mut()) {
        Some(output) => {
            let _ = output.set_count(output.count().saturating_add(result.count()));
        }
        None => items[OUTPUT] = Some(result.clone()),
    }
    if let Some(Some(mut input)) = items.get(INPUT).cloned() {
        let left = input.count().saturating_sub(1);
        if left == 0 {
            items[INPUT] = None;
        } else if input.set_count(left).is_ok() {
            items[INPUT] = Some(input);
        }
    }
}

/// `lit` follows `isBurning`, as `AbstractFurnaceBlockEntity.setLit` writes its block state. The
/// property write is a no-op when it already holds the value, so no read is needed.
fn sync_lit(host: &mut dyn ServerHost, pos: [i32; 3], lit: bool) {
    let _ = host.set_block_property(pos, "lit", if lit { "true" } else { "false" });
}

// ---------------------------------------------------------------------------
// The tier-upgrade click
// ---------------------------------------------------------------------------

/// `ItemUpgrade.useOn`: right-click a compatible furnace with an upgrade item to replace it with
/// the next tier, keeping its contents and smelting state, consuming one item. It runs before the
/// furnace's own use, which would otherwise open its menu.
pub fn right_click(host: HostSide<'_>, event: &mut mod_api::RightClickBlockEvent<'_>) -> Flow {
    if event.cancelled || event.click.hand != Hand::Main {
        return Flow::Continue;
    }
    let click = event.click;
    let pos = click.pos;
    let Some(held) = click.stack.cloned() else {
        return Flow::Continue;
    };
    let HostSide::Server(server) = host else {
        return Flow::Continue;
    };
    let Some(player) = server.player(click.player) else {
        return Flow::Continue;
    };
    let Some(level) = player.level(&*server) else {
        return Flow::Continue;
    };
    let Some(source) = block_id(&*server, level, pos) else {
        return Flow::Continue;
    };
    let Some(target) = upgrade_target(&source, &held.item().id(&*server)) else {
        return Flow::Continue;
    };
    let Some(current) = level.block_state(&*server, pos) else {
        return Flow::Continue;
    };
    let Some(next) = server
        .block_state_string(current)
        .and_then(|state| target_state_name(&state, target))
        .and_then(|state| server.block_state_id(&state))
    else {
        return Flow::Continue;
    };
    let (items, state) = capture(&mut *server, level, pos);
    if !level
        .set_block(&mut *server, pos, next, upgrade_flags())
        .unwrap_or(false)
    {
        return Flow::Continue;
    }
    pending().push((
        server.dimension().to_owned(),
        pos,
        Pending {
            target,
            items,
            state,
        },
    ));
    // One upgrade item, unless the player is creative (their hand gives without losing any).
    let place = ItemPlace::PlayerHand {
        player: click.player,
        hand: click.hand,
    };
    let resource = ItemResource::of(&held);
    let _ = server
        .transaction(|host, tx| -> Result<u32, ()> { Ok(place.extract(host, tx, &resource, 1)) });
    *event.cancellation_result = InteractionResult::Success;
    Flow::Cancel
}

/// The flags of an upgrade's block replacement: neighbours and clients see the new block, and the
/// removed block entity's side effects are skipped, so its container does **not** spill its three
/// slots onto the floor while [`apply_pending`] restores that same snapshot into the new tier. A
/// spill plus the restore is the duplicated inventory the reviewer found: every upgrade minted a
/// second copy of everything the furnace held.
fn upgrade_flags() -> BlockUpdateFlags {
    BlockUpdateFlags::NEIGHBORS
        | BlockUpdateFlags::CLIENTS
        | BlockUpdateFlags::SKIP_BLOCK_ENTITY_SIDE_EFFECTS
}

/// The source block's contents and smelting state. A vanilla furnace keeps only its three slots
/// (upstream starts the new tier with empty ones, and this port keeps them); a mod furnace keeps
/// its saved state too.
fn capture(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
) -> (Vec<Option<ItemStack>>, State) {
    let block = block_id(&*host, level, pos);
    if block
        .as_deref()
        .is_some_and(|id| Tier::from_block(id).is_some())
    {
        let Some(entity) = host.block_entity(level, pos) else {
            return (vec![None; SLOTS], State::default());
        };
        let state = entity
            .data(&*host)
            .map_or_else(State::default, |nbt| read_state(&nbt));
        let items = (0..SLOTS as u32)
            .map(|slot| entity.item_slot(&*host, slot).ok().flatten())
            .collect();
        return (items, state);
    }
    if block.as_deref() == Some(VANILLA_FURNACE) {
        // The vanilla furnace's `WorldlyContainer` sides: up reaches the input, the horizontal
        // faces the fuel, down the output.
        return (
            vec![
                furnace_slot(host, level, pos, Direction::Up),
                furnace_slot(host, level, pos, Direction::North),
                furnace_slot(host, level, pos, Direction::Down),
            ],
            State::default(),
        );
    }
    (vec![None; SLOTS], State::default())
}

/// Slot 0 of the item handler a face of the block at `pos` reaches, as a stack.
fn furnace_slot(
    host: &mut dyn ServerHost,
    level: LevelRef,
    pos: [i32; 3],
    side: Direction,
) -> Option<ItemStack> {
    let handler = level.capability(host, &ITEMS, pos, Some(side))?;
    let amount = handler.amount(host, 0);
    handler.resource(host, 0)?.to_stack(amount)
}

/// The state an upgrade writes: the target block, the source's `facing`, unlit and with the jar's
/// `jovial`/`type` cosmetic states back at 0.
fn target_state_name(source_state: &str, target: &str) -> Option<String> {
    let facing = facing_of(source_state)?;
    Some(format!(
        "{target}[facing={facing},jovial=0,lit=false,type=0]"
    ))
}

/// The `facing` property of a serialized block state, when it has one.
fn facing_of(state: &str) -> Option<&str> {
    let (_, properties) = state.split_once('[')?;
    let properties = properties.strip_suffix(']')?;
    properties.split(',').find_map(|entry| {
        let (key, value) = entry.split_once('=')?;
        (key == "facing").then_some(value)
    })
}

/// The item key of an id, when this build knows it.
fn item_key(host: &dyn Host, id: &str) -> Option<mod_api::ItemKey> {
    host.item_key(&Identifier::parse(id).ok()?)
}

// ---------------------------------------------------------------------------
// The readout command
// ---------------------------------------------------------------------------

/// `/ironfurnaces info <pos>`: the harness readout. It names another furnace's input, fuel and
/// output anywhere in the level, so it is registered at the operator level `/data get block`
/// needs (2, game master) and its body asks again before reading anything.
pub fn declare_commands(commands: &mut Vec<Command>) {
    commands.push(Command::op(
        "ironfurnaces info <pos:block_pos>",
        OPERATOR_LEVEL,
    ));
}

/// Run `/ironfurnaces info <pos>` and take it.
pub fn run_command(host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
    let invocation = event.command;
    let Some(root) = invocation.literals.first().map(String::as_str) else {
        return Flow::Continue;
    };
    if root != "ironfurnaces" {
        return Flow::Continue;
    }
    let words: Vec<&str> = invocation.literals.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["ironfurnaces", "info"] => {
            if !may_read(event.source.permission()) {
                say(
                    host,
                    &format!(
                        "ironfurnaces info needs an operator (permission level {OPERATOR_LEVEL})"
                    ),
                );
                return Flow::Cancel;
            }
            let Some(pos) = block_pos(invocation, "pos") else {
                say(host, "Usage: /ironfurnaces info <x> <y> <z>");
                return Flow::Cancel;
            };
            let line = readout(host, pos);
            say(host, &line);
        }
        _ => say(host, "Usage: /ironfurnaces info <x> <y> <z>"),
    }
    Flow::Cancel
}

/// The tier, position, burn/cook counters and the three slots, as `/ironfurnaces info` prints them.
fn readout(host: &mut dyn ServerHost, pos: [i32; 3]) -> String {
    let Some(level) = host.level(host.dimension()) else {
        return NO_FURNACE.to_owned();
    };
    let Some(tier) = block_id(&*host, level, pos)
        .as_deref()
        .and_then(Tier::from_block)
    else {
        return NO_FURNACE.to_owned();
    };
    let Some(entity) = host.block_entity(level, pos) else {
        return NO_FURNACE.to_owned();
    };
    let state = entity
        .data(&*host)
        .map_or_else(State::default, |nbt| read_state(&nbt));
    format!(
        "ironfurnaces {} {} {} {} burn={}/{} cook={}/{} input={} fuel={} output={}",
        tier.label(),
        pos[0],
        pos[1],
        pos[2],
        state.burn,
        state.burn_total,
        state.progress,
        state.total,
        slot_text(entity, &*host, INPUT),
        slot_text(entity, &*host, FUEL),
        slot_text(entity, &*host, OUTPUT),
    )
}

/// What the readout says where no tier furnace stands: one generic line, naming no block.
const NO_FURNACE: &str = "No Iron Furnace there.";

/// The level `/ironfurnaces info` is registered at: 2, the game master level `/data get block`
/// needs.
const OPERATOR_LEVEL: u8 = 2;

/// Whether a command source may read a furnace's contents: an operator, or the console (which
/// always carries level 4).
fn may_read(permission: mod_api::PermissionLevel) -> bool {
    permission.level() >= OPERATOR_LEVEL
}

/// One slot as `id:count`, or `none`.
fn slot_text(entity: BlockEntityRef, host: &dyn Host, index: usize) -> String {
    match entity.item_slot(host, index as u32) {
        Ok(Some(stack)) => format!("{}:{}", stack.item().id(host), stack.count()),
        _ => "none".to_owned(),
    }
}

/// A command's feedback line.
fn say(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}

/// The `pos` block-position argument of a command invocation.
fn block_pos(invocation: &CommandInvocation, name: &str) -> Option<[i32; 3]> {
    match invocation.argument(name)? {
        CommandArgumentValue::BlockPos(pos) => Some(*pos),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tier_names_are_the_eight_ported_furnaces() {
        assert_eq!(
            bare_names(),
            vec![
                "copper_furnace",
                "iron_furnace",
                "gold_furnace",
                "diamond_furnace",
                "emerald_furnace",
                "obsidian_furnace",
                "crystal_furnace",
                "netherite_furnace",
            ]
        );
    }

    #[test]
    fn the_container_is_the_three_slot_furnace_menu() {
        let declared = container();
        assert_eq!(declared.slots, 3);
        assert_eq!(declared.inputs, vec![0, 1]);
        assert_eq!(declared.outputs, vec![2]);
        assert_eq!(declared.menu.as_deref(), Some("minecraft:furnace"));
        for face in declared.faces {
            assert_eq!(face, vec![0, 1, 2], "every face reaches all three slots");
        }
    }

    #[test]
    fn the_saved_state_round_trips_through_the_upstream_keys() {
        let mut nbt = Compound::new();
        let state = State {
            burn: 1258,
            burn_total: 1280,
            progress: 23,
            total: 160,
        };
        write_state(&mut nbt, state);
        assert_eq!(read_state(&nbt), state);
        // An empty compound (a freshly declared block entity) reads as not burning.
        assert_eq!(read_state(&Compound::new()), State::default());
    }

    #[test]
    fn the_saved_state_uses_the_upstream_key_names() {
        let mut nbt = Compound::new();
        write_state(&mut nbt, State::default());
        for key in ["BurnTime", "BurnTimeTotal", "CookTime", "CookTimeTotal"] {
            assert!(
                matches!(nbt.get(&key.into()), Some(Tag::Int(_))),
                "{key} is saved as an int"
            );
        }
    }

    #[test]
    fn the_facing_is_read_from_a_serialized_state() {
        assert_eq!(
            facing_of("ironfurnaces:copper_furnace[facing=west,jovial=0,lit=true,type=0]"),
            Some("west")
        );
        assert_eq!(
            facing_of("minecraft:furnace[facing=north,lit=false]"),
            Some("north")
        );
        assert_eq!(facing_of("minecraft:stone"), None);
    }

    #[test]
    fn the_upgraded_state_keeps_the_facing_and_clears_lit_jovial_and_type() {
        let source = "ironfurnaces:copper_furnace[facing=west,jovial=0,lit=true,type=1]";
        assert_eq!(
            target_state_name(source, "ironfurnaces:iron_furnace").as_deref(),
            Some("ironfurnaces:iron_furnace[facing=west,jovial=0,lit=false,type=0]")
        );
        // A vanilla furnace has no jovial/type properties, but its facing still carries over.
        assert_eq!(
            target_state_name(
                "minecraft:furnace[facing=east,lit=true]",
                "ironfurnaces:iron_furnace"
            )
            .as_deref(),
            Some("ironfurnaces:iron_furnace[facing=east,jovial=0,lit=false,type=0]")
        );
        assert_eq!(
            target_state_name("minecraft:stone", "ironfurnaces:iron_furnace"),
            None
        );
    }

    #[test]
    fn a_slot_reads_as_id_and_count_or_none() {
        assert_eq!(slot_label(None), "none");
        assert_eq!(
            slot_label(Some(("minecraft:raw_iron", 2))),
            "minecraft:raw_iron:2"
        );
    }

    /// `slot_text`'s format without a host: the same string the readout prints.
    fn slot_label(slot: Option<(&str, u32)>) -> String {
        slot.map_or_else(|| "none".to_owned(), |(id, count)| format!("{id}:{count}"))
    }

    #[test]
    fn an_upgrade_replaces_the_block_without_spilling_the_old_container() {
        // The removed block entity's spill is what duplicated the inventory: the snapshot is
        // restored into the new tier, so anything the old one drops is a second copy.
        let flags = upgrade_flags();
        assert!(
            flags.bits() & BlockUpdateFlags::SKIP_BLOCK_ENTITY_SIDE_EFFECTS.bits() != 0,
            "the replacement must skip the removed container's spill"
        );
        assert!(
            flags.bits() & BlockUpdateFlags::NEIGHBORS.bits() != 0,
            "the replacement still updates its neighbours"
        );
        assert!(
            flags.bits() & BlockUpdateFlags::CLIENTS.bits() != 0,
            "the replacement still reaches clients"
        );
    }

    #[test]
    fn the_readout_needs_an_operator() {
        use mod_api::PermissionLevel;
        assert!(!may_read(PermissionLevel::All));
        assert!(!may_read(PermissionLevel::Moderators));
        assert!(may_read(PermissionLevel::Gamemasters));
        assert!(may_read(PermissionLevel::Admins));
        assert!(may_read(PermissionLevel::Owners));
        assert_eq!(OPERATOR_LEVEL, 2);
    }

    #[test]
    fn a_queued_upgrade_is_taken_only_at_its_own_dimension_and_position() {
        let mut queue = vec![(
            "minecraft:overworld".to_owned(),
            [1, 64, 2],
            Pending {
                target: "ironfurnaces:iron_furnace",
                items: vec![None; SLOTS],
                state: State::default(),
            },
        )];
        assert!(take_pending(&mut queue, "minecraft:the_end", [1, 64, 2]).is_none());
        assert!(take_pending(&mut queue, "minecraft:overworld", [2, 64, 2]).is_none());
        assert_eq!(queue.len(), 1, "another position's upgrade is left queued");
        assert!(take_pending(&mut queue, "minecraft:overworld", [1, 64, 2]).is_some());
        assert!(queue.is_empty(), "the taken upgrade leaves the queue");
    }
}
