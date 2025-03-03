use rltk::{Algorithm2D, BaseMap, FontCharType, Point, RandomNumberGenerator, Rltk, RGB};
use serde::{Deserialize, Serialize};
use specs::{Entity, World};

use super::rect::*;
use std::cmp::{min, max};

pub const MAPWIDTH:  usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub puddles: Vec<i32>,
    pub depth: i32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * MAPWIDTH) + x as usize
    }

    pub fn new_map_rooms_and_corridors(new_depth: i32) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: vec![],
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![vec![]; MAPCOUNT],
            puddles: vec![0; MAPCOUNT],
            depth: new_depth
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, MAPWIDTH as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, MAPHEIGHT as i32 - h - 1) - 1;

            let new_room = Rect::new(x, y, w, h);

            let mut room_ok = true;
            for other_room in map.rooms.iter() {
                room_ok &= !other_room.intersect(&new_room);

            }

            if room_ok {
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();

                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.apply_room_to_map(&new_room);
                map.rooms.push(new_room);
            }
        }

        let (stair_x, stair_y) = map.rooms.iter()
                                          .last()
                                          .expect("No rooms were generated?")
                                          .center();

        let idx = map.xy_idx(stair_x, stair_y);
        map.tiles[idx] = TileType::DownStairs;

        map
    }

    pub fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1+1..=room.y2 {
            for x in room.x1+1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < MAPCOUNT {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > (MAPWIDTH-1) as i32 
        || y < 1 || y > (MAPHEIGHT-1) as i32 {
            return false;
        }
        let idx = self.xy_idx(x, y);

        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for tile in self.tile_content.iter_mut() {
            tile.clear();
        }
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut x = 0;
    let mut y = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph: FontCharType;
            let mut fg: RGB;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.3, 0.3, 0.3);
                }
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0.8, 0.8, 0.95);
                }
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0.8, 0.8, 0.95);
                }
            }
            if !map.visible_tiles[idx] { 
                let mut darkest = f32::min(fg.b, f32::min(fg.g, fg.r));
                if darkest > 0.3 {
                    darkest -= 0.3;
                }
                fg.b = darkest;
                fg.g = darkest;
                fg.r = darkest;
            }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        x += 1;
        if x > map.width - 1 {
            x = 0;
            y += 1;
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(MAPWIDTH, MAPHEIGHT)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % MAPWIDTH as i32;
        let y = idx as i32 / MAPWIDTH as i32;
        let w = MAPWIDTH;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0));}      // W
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0));}      // E
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0));}      // N
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0));}      // S
        if self.is_exit_valid(x-1, y+1) { exits.push((idx-1+w, 1.0));} // SW
        if self.is_exit_valid(x+1, y-1) { exits.push((idx+1-w, 1.0));} // NE
        if self.is_exit_valid(x-1, y-1) { exits.push((idx-1-w, 1.0));} // NW
        if self.is_exit_valid(x+1, y+1) { exits.push((idx+1+w, 1.0));} // SE

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = MAPWIDTH;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);

        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}