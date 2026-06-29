use crate::content::{ModKind, SHOP_ITEMS, ShopItem};
use crate::ecs::{GameState, ShipMods};

pub fn item_level(mods: &ShipMods, kind: ModKind) -> u8 {
    match kind {
        ModKind::Lance => mods.lance,
        ModKind::Nova => mods.nova_max,
        ModKind::Aegis => mods.aegis,
        ModKind::Seeker => mods.seeker,
        ModKind::Magnet => mods.magnet,
        ModKind::Hull => mods.hull,
        ModKind::Rapid => mods.rapid,
        ModKind::Repair => 0,
    }
}

pub fn maxed(game: &GameState, item: &ShopItem) -> bool {
    if item.kind == ModKind::Repair {
        return game.shields >= game.max_shields;
    }
    item_level(&game.mods, item.kind) >= item.max_level
}

pub fn current_cost(game: &GameState, item: &ShopItem) -> u32 {
    if item.kind == ModKind::Repair {
        let missing = (game.max_shields - game.shields).max(0) as u32;
        return item.base_cost + missing.saturating_sub(1) * item.cost_step;
    }
    item.base_cost + item_level(&game.mods, item.kind) as u32 * item.cost_step
}

pub fn can_buy(game: &GameState, item: &ShopItem) -> bool {
    !maxed(game, item) && game.credits >= current_cost(game, item)
}

pub fn buy(game: &mut GameState, index: usize) {
    let item = &SHOP_ITEMS[index];
    if !can_buy(game, item) {
        return;
    }
    game.credits -= current_cost(game, item);
    match item.kind {
        ModKind::Lance => game.mods.lance += 1,
        ModKind::Nova => {
            game.mods.nova_max += 1;
            game.nova_charges = game.mods.nova_max;
        }
        ModKind::Aegis => game.mods.aegis += 1,
        ModKind::Seeker => game.mods.seeker += 1,
        ModKind::Magnet => game.mods.magnet += 1,
        ModKind::Hull => {
            game.mods.hull += 1;
            game.max_shields += 1;
            game.shields += 1;
        }
        ModKind::Rapid => game.mods.rapid += 1,
        ModKind::Repair => game.shields = game.max_shields,
    }
}
