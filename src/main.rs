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

// === Scene Management ===

trait Scene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition>;
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition>;
}

enum Transition {
    None,
    Push(Box<dyn Scene>),
    Pop,
}

struct GameState {
    scenes: Vec<Box<dyn Scene>>,
    scaler: ScreenScaler,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let initial_scene = TestScene::new(ctx)?;

        Ok(GameState {
            scenes: vec![Box::new(initial_scene)],
            scaler: ScreenScaler::with_window_size(
                ctx,
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                ScalingMode::ShowAll,
            )?,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.update(ctx)? {
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
            Some(active_scene) => match active_scene.draw(ctx)? {
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
    fn new(ctx: &mut Context) -> tetra::Result<TestScene> {
        // Create a world to store our entities
        let universe = Universe::new();
        let mut world = universe.create_world();

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
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Space) {
            Ok(Transition::Push(Box::new(TestScene2::new(ctx)?)))
        } else {
            Ok(Transition::None)
        }

        // Create a query which finds all `Position` and `Velocity` components
        let mut query = <(Write<Position>, Read<Health>)>::query();

        // Iterate through all entities that match the query in the world
        for (mut pos, vel) in query.iter(&mut world) {
            pos.x += vel.dx;
            pos.y += vel.dy;
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
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
    fn new(ctx: &mut Context) -> tetra::Result<TestScene2> {
        Ok(TestScene2 {
            title_text: Text::new("Test Scene 2", Font::default(), 72.0),
            test_text: Text::new("This is some test text.\n\nYay Ludum Dare!\nThe 46th one!\nThat's this one!\nWill I succeed?\nI better.\n\nHere we go...", Font::default(), 32.0),
        })
    }
}

impl Scene for TestScene2 {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        if input::is_key_pressed(ctx, Key::Space) {
            Ok(Transition::Push(Box::new(TestScene::new(ctx)?)))
        } else {
            Ok(Transition::None)
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        graphics::draw(ctx, &self.title_text, Vec2::new(16.0, 16.0));
        graphics::draw(ctx, &self.test_text, Vec2::new(16.0, 96.0));

        Ok(Transition::None)
    }
}
