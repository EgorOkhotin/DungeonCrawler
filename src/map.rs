use crate::prelude::*;

const NUM_TILES: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
}

pub fn map_idx(x: i32, y: i32) -> usize {
    ((y * SCREEN_WIDTH) + x) as usize
}

impl Map {
    pub fn new() -> Self {
        Self {
            tiles: vec![TileType::Floor; NUM_TILES],
        }
    }

    pub fn in_bounds(&self, point: &Point) -> bool {
        point.y > 0 && point.y < SCREEN_HEIGHT && 
        point.x > 0 && point.x < SCREEN_WIDTH
    }

    pub fn can_enter_tile(&self, point: Point) -> bool {
        let idx = map_idx(point.x, point.y);

        self.in_bounds(&point) &&
        self.tiles[idx] == TileType::Floor
    }

    pub fn try_idx(&self, point: Point) -> Option<usize> {
        if !self.in_bounds(&point) {
            None
        } else {
            Some(map_idx(point.x, point.y))
        }
    }

    pub fn render(&mut self, context: &mut BTerm, camera: &Camera) {
        context.set_active_console(0);

        for y in camera.top_y..camera.bottom_y{
            for x in camera.left_x..camera.right_x {
                if self.in_bounds(&Point::new(x,y)) {
                    let idx = map_idx(x, y);

                    match self.tiles[idx] {
                        TileType::Floor => 
                            context.set(x - camera.left_x, y -camera.top_y, WHITE, BLACK, to_cp437('.')),
                        TileType::Wall => 
                            context.set(x - camera.left_x, y -camera.top_y, WHITE, BLACK, to_cp437('#')),
                    }
                }
            }
        }
    }
}
