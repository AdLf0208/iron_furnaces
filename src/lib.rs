//! Iron Furnaces for Horizon — eight tiered fuel furnaces (copper … netherite).
//!
//! Ported from Iron Furnaces 4.3.2 (`Qelifern/IronFurnacesNeoForge` branch 4.3.2, Apache-2.0).
//! [`runtime`] is the pure truth — the tier table, the cook/burn scaling and the furnace tick —
//! and is byte-identical to the retired `.hnmod` port's file. [`support`] is the glue: the block
//! entity, its three-slot container, the tick, the tier-upgrade items and the `/ironfurnaces info`
//! readout. This file is the one `impl Hooks`.
//!
//! # Where the retired container's hooks went
//!
//! The 2026-11-17 mod-API merge (#16941, launcher ≥ 0.2.70) retired `.hnmod`: the plan, the
//! injection points and the per-hook helper files are gone. A hook is a typed method here, and a
//! moment that belongs to one block, item or block entity type is a **Part** attached to it.
//!
//! | `.hnmod` hook (API) | here |
//! |---|---|
//! | the `content_root` injection (AE2 pattern) | gone: one flat `.hmod` carries `assets/` and `data/` itself |
//! | `block_entity` (15) | the block entity **type** declared in [`Mod::register`]; the type starts with the empty compound the old hook returned |
//! | `block_entity_ticks` (15) | `support::Furnace` (`BlockEntityPart::block_entity_ticks`) |
//! | `block_entity_container` (16/17) | `Registrar::block_container` in [`Mod::register`] |
//! | `block_entity_tick` (17) | `BlockEntityPart::block_entity_tick` on `support::Furnace` |
//! | `right_click_block` (21) | [`Mod::right_click_block`] → `support::right_click` |
//! | `register_commands` (8) | [`Mod::register_commands`] → `support::declare_commands` |
//! | `perform_command` (8) | [`Mod::run_mod_command`] → `support::run_command` (typed arguments) |

pub mod runtime;
pub mod support;

use mod_api::{
    self, BlockEntityPart, BlockEntityTickEvent, BlockEntityTicksEvent, BlockPart, Flow, Hooks,
    HostSide, RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent,
    ServerHost,
};

/// The mod keeps no state of its own: a furnace's burn/cook state lives in its block entity's
/// saved compound, and the upgrade queue is a `static` in `support`.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare what the game answers for the eight blocks (`EntityBlock` plus NeoForge's
    /// `RegisterCapabilitiesEvent`): one block entity type holding all of them, the Part that
    /// ticks it, and the three-slot furnace container on every tier.
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        support::declare(r);
    }

    /// A right click with a tier-upgrade item: swap the furnace for the next tier before the
    /// furnace's own use opens its menu.
    fn right_click_block(
        &self,
        host: HostSide<'_>,
        event: &mut mod_api::RightClickBlockEvent<'_>,
    ) -> Flow {
        support::right_click(host, event)
    }

    /// `/ironfurnaces info <pos>` (the harness readout).
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        support::declare_commands(event.commands);
    }

    /// Run `/ironfurnaces info <pos>` and take it.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::run_command(host, event)
    }
}

/// The Part the game asks about every tier furnace: it ticks on the server, each tick runs the
/// pure smelting rules, and the `lit` state follows.
pub struct Furnace;

#[mod_api::horizon_part]
impl BlockPart for Furnace {}

#[mod_api::horizon_part]
impl BlockEntityPart for Furnace {
    /// A tier furnace's block entity ticks on the server (`EntityBlock.getTicker`).
    fn block_entity_ticks(&self, _host: &dyn mod_api::Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: apply a queued upgrade, then smelt.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        support::tick(host, event);
    }
}
