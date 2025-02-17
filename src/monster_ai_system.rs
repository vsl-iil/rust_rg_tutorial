use rltk::Point;
use specs::{ReadStorage, System};
use specs::prelude::*;

use crate::components::{Monster, Position, Viewshed, WantsToMelee};
use crate::map::Map;
use crate::RunState;

pub struct MonsterAI { }

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, WantsToMelee>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player_pos, player_entity, runstate, entities, mut viewshed, monster, mut position, mut want_melee) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, viewshed, _monster, pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
            if viewshed.visible_tiles.contains(&*player_pos) {
                // AI
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);

                if distance < 1.5 {
                    want_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Unable to insert attack on player");
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &*map
                    );
                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }

            }
        }
    }
}