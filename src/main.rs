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
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
}
use prelude::*;

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_title("Dungeon crawl master")
        .with_fps_cap(30.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png",8,8)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(SCREEN_HEIGHT*2, SCREEN_HEIGHT*2, "terminal8x8.png")
        .build()?;

    main_loop(context, State::new())
}

struct State {
    ecs: World,
    resources: Resources,

    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule
}

impl State {
    fn new() -> Self {
        let mut world = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let map_builder = MapBuilder::new(&mut rng);
        let player_position = map_builder.player_start;

        spawn_player(&mut world, player_position);

        map_builder
            .rooms
            .iter()
            .skip(1)
            .map(|r| r.center())
            .for_each(|pos| spawn_monster(&mut world, &mut rng, pos));

        resources.insert(map_builder.map);
        resources.insert(Camera::new(player_position));
        resources.insert(TurnState::AwaitingInput);

        Self {
            ecs: world,
            resources: resources,
            input_systems: build_input_schedule(),
            player_systems: build_player_schedule(),
            monster_systems: build_monster_schedule()
        }
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
        self.resources.insert(Point::from_tuple(context.mouse_pos()));

        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::AwaitingInput => {
                self.input_systems.execute(&mut self.ecs, &mut self.resources);
            },
            TurnState::PlayerTurn => {
                self.player_systems.execute(&mut self.ecs, &mut self.resources);
            },
            TurnState::MonsterTurn => {
                self.monster_systems.execute(&mut self.ecs, &mut self.resources);
            }
        }

        render_draw_buffer(context).expect("Render error");
        //TODO Render draw buffer
        //TODO CommandBuffer
    }
}
