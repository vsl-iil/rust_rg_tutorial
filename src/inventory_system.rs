use specs::prelude::*;

use crate::{components::{CombatStats, Consumable, InBackpack, Name, Position, ProvidesHealing, WantsToDropItem, WantsToPickupItem, WantsToUseItem}, gamelog::GameLog};

pub struct InventorySystem {}

impl<'a> System<'a> for InventorySystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut log, mut wants_pickup, mut pos, name, mut backpack) = data;

        for pickup in wants_pickup.join() {
            pos.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack { owner: pickup.collected_by }).expect("Unable to insert backpack entry");
            
            if pickup.collected_by == *player_entity {
                let mut log_name = "something";
                if let Some(item_name) = name.get(pickup.item) {
                    log_name = &item_name.name;
                }
                log.entries.push(format!("You picked up {}.", log_name));
            }
        }

        wants_pickup.clear();
    }
}


pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
                        ReadStorage<'a, ProvidesHealing>,
                        ReadStorage<'a, Consumable>,
                        WriteStorage<'a, CombatStats>
                    );

 fn run(&mut self, data: Self::SystemData) {
    let (player_entity, mut gamelog, entities, mut want_use, names, healing, consumables, mut combat_stats) = data;

    for (entity, usable, stats) in (&entities, &want_use, &mut combat_stats).join() {
        let item_heals = healing.get(usable.item);
        match item_heals {
            None => {},
            Some(healer) => {
                stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                if entity == *player_entity {
                    let mut healer_name = "something";
                    if let Some(name) = names.get(usable.item) {
                        healer_name = &name.name;
                    }
                    gamelog.entries.push(format!("You drank the {}, healing {} hp.", healer_name, healer.heal_amount));
                    
                }
            }
        }

        if let Some(_) = consumables.get(usable.item) {
            entities.delete(usable.item).expect("Unable to delete consumable");
        }
    }

    want_use.clear();
 }
}


pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut log, entities, mut drop, mut pos, names, mut backpack) = data;

        for (entity, to_drop) in (&entities, &drop).join() {
            let mut dropper_pos = Position { x: 0, y: 0 };
            {
                let dropped_pos = pos.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            pos.insert(to_drop.item, dropper_pos).expect("Unable to insert drop position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                let mut item_name = "something";
                if let Some(name) = names.get(to_drop.item) {
                    item_name = &name.name;
                }
                log.entries.push(format!("You dropped the {}", item_name));
            }
        }

        drop.clear();
    }
}