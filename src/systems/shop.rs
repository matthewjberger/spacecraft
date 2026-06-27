use crate::content::{ModKind, SHOP_ITEMS, ShopItem};
use crate::ecs::{GameState, ShipMods};

pub fn item_level(mods: &ShipMods, kind: ModKind) -> u8 {
    match kind {
        ModKind::Hull => mods.hull,
        ModKind::Rapid => mods.rapid,
        ModKind::Damage => mods.damage,
        ModKind::Magnet => mods.magnet,
        ModKind::Lance => mods.lance,
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
        ModKind::Hull => {
            game.mods.hull += 1;
            game.max_shields += 1;
            game.shields += 1;
        }
        ModKind::Rapid => game.mods.rapid += 1,
        ModKind::Damage => game.mods.damage += 1,
        ModKind::Magnet => game.mods.magnet += 1,
        ModKind::Lance => game.mods.lance += 1,
        ModKind::Repair => game.shields = game.max_shields,
    }
}
