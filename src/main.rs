use ggez::event::KeyCode;
use ggez::mint::Point2;
use ggez::mint::Vector2;
use ggez::{event, graphics, Context, GameResult};
use rand::{thread_rng, Rng};

type Point = Point2<f32>;
type Vector = Vector2<f32>;

const GRID_SIZE: (i16, i16) = (30, 20);
const GRID_CELL_SIZE: (i16, i16) = (32, 32);
const SCREEN_SIZE: (u32, u32) = (
    GRID_SIZE.0 as u32 * GRID_CELL_SIZE.0 as u32,
    GRID_SIZE.1 as u32 * GRID_CELL_SIZE.1 as u32,
);

fn create_rays(player: Point, walls: &[Wall]) -> Vec<Ray> {
    let mut rays = Vec::new();
    for a in 0..360 {
        let radians = (a as f32).to_radians();
        let dir = Vector {
            x: radians.sin(),
            y: radians.cos(),
        };
        let mut closest: Option<Ray> = None;
        let mut record = std::f32::MAX;
        for wall in walls {
            if let Some((d, ray)) = Ray::try_new(player, wall, dir) {
                if d < record {
                    record = d;
                    closest = Some(ray);
                }
                // rays.push(ray);
            }
        }
        if let Some(ray) = closest {
            rays.push(ray);
        }
    }
    rays
}

fn can_cast_ray(t: f32, u: f32) -> bool {
    t > 0. && t < 1. && u > 0.
}

struct Player {
    pos: Point,
}

impl Player {
    fn new(x: u32, y: u32) -> Self {
        let x = x as f32;
        let y = y as f32;
        let pos = Point { x, y };
        Self { pos }
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.pos.x = x;
        self.pos.y = y;
    }

    fn draw(&self, meshbuilder: &mut graphics::MeshBuilder) -> GameResult {
        meshbuilder.circle(
            graphics::DrawMode::fill(),
            self.pos,
            10.0,
            0.1,
            graphics::Color::new(1.0, 1.0, 1.0, 1.0),
        )?;
        Ok(())
    }
}

struct Wall {
    pos1: Point,
    pos2: Point,
}

impl Wall {
    fn new(x1: u32, y1: u32, x2: u32, y2: u32) -> Self {
        let x1 = x1 as f32;
        let y1 = y1 as f32;
        let pos1 = [x1, y1].into();
        let x2 = x2 as f32;
        let y2 = y2 as f32;
        let pos2 = [x2, y2].into();
        Self { pos1, pos2 }
    }

    fn random() -> Self {
        let range_x = 50..SCREEN_SIZE.0 - 50;
        let range_y = 50..SCREEN_SIZE.1 - 50;
        let x1 = thread_rng().gen_range(range_x.clone());
        let y1 = thread_rng().gen_range(range_y.clone());
        let x2 = thread_rng().gen_range(range_x);
        let y2 = thread_rng().gen_range(range_y);
        Self::new(x1, y1, x2, y2)
    }

    fn draw(&self, meshbuilder: &mut graphics::MeshBuilder) -> GameResult {
        meshbuilder.line(
            &[self.pos1, self.pos2],
            1.0,
            graphics::Color::new(1.0, 1.0, 1.0, 1.0),
        )?;
        Ok(())
    }
}

struct Ray {
    player: Point,
    line: Point,
}

impl Ray {
    fn try_new(player: Point, wall: &Wall, dir: Vector) -> Option<(f32, Self)> {
        let x1 = wall.pos1.x;
        let y1 = wall.pos1.y;
        let x2 = wall.pos2.x;
        let y2 = wall.pos2.y;

        let x3 = player.x;
        let y3 = player.y;
        let x4 = player.x + dir.x;
        let y4 = player.y + dir.y;
        let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        if den == 0. {
            return None;
        }
        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / den;
        let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / den;
        if can_cast_ray(t, u) {
            let x = x1 + t * (x2 - x1);
            let y = y1 + t * (y2 - y1);
            let line = Point { x, y };
            return Some((u, Ray { player, line }));
        }
        None
    }

    fn draw(&self, meshbuilder: &mut graphics::MeshBuilder) -> GameResult {
        meshbuilder.line(
            &[self.player, self.line],
            2.0,
            graphics::Color::new(1.0, 1.0, 1.0, 0.3),
        )?;
        Ok(())
    }
}

struct GameState {
    player: Player,
    walls: Vec<Wall>,
    rays: Vec<Ray>,
}

impl GameState {
    pub fn new() -> GameResult<Self> {
        let player = Player::new(SCREEN_SIZE.0 / 2, SCREEN_SIZE.1 / 2);
        let w = SCREEN_SIZE.0;
        let h = SCREEN_SIZE.1;
        let mut walls = Vec::new();
        walls.insert(0, Wall::new(0, 0, w, 0));
        walls.insert(0, Wall::new(0, h, w, h));
        walls.insert(0, Wall::new(0, 0, 0, h));
        walls.insert(0, Wall::new(w, 0, w, h));
        walls.push(Wall::random());
        walls.push(Wall::random());
        walls.push(Wall::random());
        walls.push(Wall::random());
        walls.push(Wall::random());
        walls.push(Wall::random());
        let rays = create_rays(player.pos, &walls);
        Ok(GameState {
            player,
            walls,
            rays,
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.rays = create_rays(self.player.pos, &self.walls);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let mut meshbuilder = graphics::MeshBuilder::new();
        self.player.draw(&mut meshbuilder)?;
        for wall in self.walls.iter() {
            wall.draw(&mut meshbuilder)?;
        }
        for ray in self.rays.iter() {
            ray.draw(&mut meshbuilder)?;
        }

        let mesh = meshbuilder.build(ctx)?;
        graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;

        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _keycode: KeyCode,
        _keymod: ggez::input::keyboard::KeyMods,
        _repeat: bool,
    ) {
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        self.player.set_pos(x + dx, y + dy);
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        for _ in 0..4 {
            self.walls.remove(0);
        }
        let w = width as u32;
        let h = height as u32;
        self.walls.insert(0, Wall::new(0, 0, w, 0));
        self.walls.insert(0, Wall::new(0, h, w, h));
        self.walls.insert(0, Wall::new(0, 0, 0, h));
        self.walls.insert(0, Wall::new(w, 0, w, h));
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ggez::ContextBuilder::new("ray tracing", "Cowboy")
        .window_setup(ggez::conf::WindowSetup::default().title("Ray Tracking"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32),
        )
        .build()
        .expect("Failed to build ggez context");

    let state = GameState::new()?;
    event::run(ctx, event_loop, state)
}
