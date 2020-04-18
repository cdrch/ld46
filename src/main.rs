use legion::prelude::*;
use rand::{self, Rng};
use tetra::audio::Sound;
use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::{self, Color, DrawParams, Font, Text, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::window;
use tetra::{Context, ContextBuilder, Event, State};

const SCREEN_WIDTH: i32 = 1920;
const SCREEN_HEIGHT: i32 = 1080;
const CELL_SIZE: i32 = 96;
const CELL_BORDER: i32 = 1;

fn main() -> tetra::Result {
    ContextBuilder::new("LD46", SCREEN_WIDTH, SCREEN_HEIGHT)
        .resizable(true)
        .maximized(true)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}

// === ECS Management ===

#[derive(Debug, PartialEq)]
struct EntityInfo {
    name: str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Health {
    hp: i32,
    last_damaged_by: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Static;

pub fn create_systems() -> Vec<Box<dyn Schedulable>> {
    vec![
        // input_system(),
        // move_system(),
        // spawn_enemy_system(),
        // collider_system(),
        simple_test_system(),
    ]
}

pub fn simple_test_system() -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("MoveSystem")
        .with_query(<(Write<Position>)>::query())
        .build(move |_commands, world, _resource, queries| {
            for (_entity, mut pos) in queries.iter_entities_mut(&mut *world) {
                pos.x += 1;
                pos.y += 1;
            } 
        })
}

// === Scene Management ===

trait Scene {
    fn update(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result<Transition>;
}

enum Transition {
    None,
    Push(Box<dyn Scene>),
    Pop,
}

struct GameState {
    universe: Universe,
    pub world: World,
    pub executor: Executor,
    pub resources: Resources,

    scenes: Vec<Box<dyn Scene>>,
    scaler: ScreenScaler,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let universe = Universe::new();
        let mut world = universe.create_world();
        let executor = Executor::new(vec![]);
        let mut resources = Resources::default();


        let initial_scene = TestScene::new(ctx, &mut world)?;

        Ok(GameState {
            universe,
            world,
            executor,
            resources,
            scenes: vec![Box::new(initial_scene)],
            scaler: ScreenScaler::with_window_size(
                ctx,
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                ScalingMode::ShowAll,
            )?,
        })
    }

    pub fn set_systems(&mut self, systems: Vec<Box<dyn Schedulable>>) {
        self.executor = Executor::new(systems);
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.executor.execute(&mut self.world, &mut self.resources);

        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.update(ctx, &mut self.world)? {
                Transition::None => {}
                Transition::Push(s) => {
                    self.scenes.push(s);
                }
                Transition::Pop => {
                    self.scenes.pop();
                }
            },
            None => window::quit(ctx),
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());
        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.draw(ctx, &mut self.world)? {
                Transition::None => {}
                Transition::Push(s) => {
                    self.scenes.push(s);
                }
                Transition::Pop => {
                    self.scenes.pop();
                }
            },
            None => window::quit(ctx),
        }

        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);
        graphics::draw(ctx, &self.scaler, Vec2::new(0.0, 0.0));

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
        }

        Ok(())
    }
}

// === Test Scene ===

struct TestScene {
    title_text: Text,
    test_text: Text,
}

impl TestScene {
    fn new(ctx: &mut Context, world: &mut World) -> tetra::Result<TestScene> {

        // Create entities with `Position` and `Velocity` data
        world.insert(
            (),
            (0..999).map(|_| {
                (
                    Position { x: 0, y: 0 },
                    Health {
                        hp: 0,
                        last_damaged_by: 0,
                    },
                )
            }),
        );

        // Create entities with `Position` data and a tagged with `Model` data and as `Static`
        // Tags are shared across many entities, and enable further batch processing and filtering use cases
        world.insert((Static,), (0..999).map(|_| (Position { x: 0, y: 0 },)));

        Ok(TestScene {
            title_text: Text::new("Test Scene", Font::default(), 72.0),
            test_text: Text::new("This is some test text.\n\nYay Ludum Dare!\nThe 46th one!\nThat's this one!\nWill I succeed?\nI better.\n\nHere we go...", Font::default(), 32.0),
        })
    }
}

impl Scene for TestScene {
    fn update(&mut self, ctx: &mut Context, mut world: &mut World) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Space) {
            return Ok(Transition::Push(Box::new(TestScene2::new(ctx,world)?)));
        }

        for (_entity) in world.iter_entities() {
            if (self.test_text.content().len() > 100) {
                self.test_text.set_content("");
            }

            self.test_text.content_mut().push_str(&"\n".to_string());
            self.test_text.content_mut().push_str(&_entity.to_string());  
        } 
        


        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result<Transition> {
        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        graphics::draw(ctx, &self.title_text, Vec2::new(16.0, 16.0));
        graphics::draw(ctx, &self.test_text, Vec2::new(16.0, 96.0));

        Ok(Transition::None)
    }
}

// === Test Scene 2 ===

struct TestScene2 {
    title_text: Text,
    test_text: Text,
}

impl TestScene2 {
    fn new(ctx: &mut Context, world: &mut World) -> tetra::Result<TestScene2> {
        Ok(TestScene2 {
            title_text: Text::new("Test Scene 2", Font::default(), 72.0),
            test_text: Text::new("This is some test text.\n\nYay Ludum Dare!\nThe 46th one!\nThat's this one!\nWill I succeed?\nI better.\n\nHere we go...", Font::default(), 32.0),
        })
    }
}

impl Scene for TestScene2 {
    fn update(&mut self, ctx: &mut Context, mut world: &mut World) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Space) {
            Ok(Transition::Push(Box::new(TestScene::new(ctx, &mut world)?)))
        } else {
            Ok(Transition::None)
        }
    }

    fn draw(&mut self, ctx: &mut Context, world: &mut World) -> tetra::Result<Transition> {
        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        graphics::draw(ctx, &self.title_text, Vec2::new(16.0, 16.0));
        graphics::draw(ctx, &self.test_text, Vec2::new(16.0, 96.0));

        Ok(Transition::None)
    }
}
