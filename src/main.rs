#![warn(clippy::all, clippy::pedantic)]
mod camera;
mod components;
mod map;
mod map_builder;
mod spawner;
mod systems;
mod turn_state;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;

    pub use crate::camera::*;
    pub use crate::components::*;
    pub use crate::map::*;
    pub use crate::map_builder::*;
    pub use crate::spawner::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;

    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 4;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 4;

    pub const TEXTURE_TILE_DIMENSION: i32 = 32;
    pub const UI_TILE_DIMENSION: i32 = 8;
}
use prelude::*;

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_title("Dungeon crawl master")
        .with_fps_cap(30.0)
        .with_dimensions(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2)
        .with_tile_dimensions(TEXTURE_TILE_DIMENSION, TEXTURE_TILE_DIMENSION)
        .with_resource_path("resources/")
        .with_font(
            "dungeonfont.png",
            TEXTURE_TILE_DIMENSION,
            TEXTURE_TILE_DIMENSION,
        )
        .with_font("terminal8x8.png", UI_TILE_DIMENSION, UI_TILE_DIMENSION)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .build()?;

    main_loop(context, State::new())
}

struct State {
    ecs: World,
    resources: Resources,

    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule,
}

impl State {
    fn new() -> Self {
        let mut world = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let map_builder = MapBuilder::new(&mut rng);
        let player_position = map_builder.player_start;
        spawn_player(&mut world, player_position);
        spawn_amulet_of_yala(&mut world, map_builder.amulet_start);

        map_builder
            .monster_spawns
            .iter()
            //.skip(1)
            //.map(|r| r.center())
            .for_each(|pos| spawn_entity(&mut world, &mut rng, *pos));

        resources.insert(map_builder.map);
        resources.insert(Camera::new(player_position));
        resources.insert(TurnState::AwaitingInput);
        resources.insert(map_builder.theme);

        Self {
            ecs: world,
            resources: resources,
            input_systems: build_input_schedule(),
            player_systems: build_player_schedule(),
            monster_systems: build_monster_schedule(),
        }
    }

    fn game_over(&mut self, context: &mut BTerm) {
        context.set_active_console(2);
        context.print_color_centered(2, RED, BLACK, "Your quest has ended.");
        context.print_color_centered(
            4,
            WHITE,
            BLACK,
            "Slain by a monster, your hero's journey has come to a \
        premature end.",
        );

        context.print_color_centered(
            5,
            WHITE,
            BLACK,
            "The Amulet of Yala remains unclaimed, and your home town \
        is not saved.",
        );
        context.print_color_centered(
            8,
            YELLOW,
            BLACK,
            "Don't worry, you can always try again with a new hero.",
        );
        context.print_color_centered(9, GREEN, BLACK, "Press 1 to play again.");

        if let Some(VirtualKeyCode::Key1) = context.key {
            self.reset_game_state();
        }
    }

    fn victory(&mut self, context: &mut BTerm) {
        context.set_active_console(2);
        context.print_color_centered(2, GREEN, BLACK, "You have won!");
        context.print_color_centered(
            4,
            WHITE,
            BLACK,
            "You put on the Amulet of Yala and feel its power course through \
your veins.",
        );
        context.print_color_centered(
            5,
            WHITE,
            BLACK,
            "Your town is saved, and you can return to your normal life.",
        );
        context.print_color_centered(
            7,
            GREEN,
            BLACK,
            "Press 1 to \
play again.",
        );
        if let Some(VirtualKeyCode::Key1) = context.key {
            self.reset_game_state();
        }
    }

    fn reset_game_state(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut self.ecs, map_builder.player_start);
        spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start);
        map_builder
            .monster_spawns
            .iter()
            //.skip(1)
            //.map(|r| r.center())
            .for_each(|pos| spawn_entity(&mut self.ecs, &mut rng, *pos));
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut BTerm) {
        context.set_active_console(0);
        context.cls();
        context.set_active_console(1);
        context.cls();
        context.set_active_console(2);
        context.cls();

        self.resources.insert(context.key);

        context.set_active_console(0);
        self.resources
            .insert(Point::from_tuple(context.mouse_pos()));

        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::AwaitingInput => {
                self.input_systems
                    .execute(&mut self.ecs, &mut self.resources);
            }
            TurnState::PlayerTurn => {
                self.player_systems
                    .execute(&mut self.ecs, &mut self.resources);
            }
            TurnState::MonsterTurn => {
                self.monster_systems
                    .execute(&mut self.ecs, &mut self.resources);
            }
            TurnState::GameOver => {
                self.game_over(context);
            }
            TurnState::Victory => {
                self.victory(context);
            }
        }

        render_draw_buffer(context).expect("Render error");
        //TODO Render draw buffer
        //TODO CommandBuffer
    }
}
