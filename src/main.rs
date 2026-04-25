use ggez::{event, graphics, input::keyboard::{KeyCode, KeyInput}, Context, GameResult};
use ggez::graphics::Drawable;
use rand::Rng;
use std::collections::LinkedList;

const GRID_SIZE: i32 = 20;
const CELL_SIZE: f32 = 30.0;
const BOARD_SIZE: f32 = GRID_SIZE as f32 * CELL_SIZE;
const PADDING: f32 = 50.0;
const BORDER_WIDTH: f32 = 5.0;
const SCREEN_WIDTH: f32 = BOARD_SIZE + 2.0 * PADDING;
const SCREEN_HEIGHT: f32 = BOARD_SIZE + 2.0 * PADDING + 100.0;

#[derive(Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Game {
    snake: LinkedList<Point>,
    direction: Direction,
    food: Point,
    score: u32,
    game_over: bool,
    last_update: f64,
    update_interval: f64,
}

impl Game {
    fn new(_ctx: &mut Context) -> GameResult<Game> {
        let mut snake = LinkedList::new();
        snake.push_back(Point { x: GRID_SIZE / 2, y: GRID_SIZE / 2 });
        let food = Game::spawn_food(&snake);
        Ok(Game {
            snake,
            direction: Direction::Right,
            food,
            score: 0,
            game_over: false,
            last_update: 0.0,
            update_interval: 0.15, // seconds
        })
    }

    fn spawn_food(snake: &LinkedList<Point>) -> Point {
        let mut rng = rand::thread_rng();
        let mut tries = 0;
        loop {
            let x = rng.gen_range(0..GRID_SIZE);
            let y = rng.gen_range(0..GRID_SIZE);
            let point = Point { x, y };
            if !snake.contains(&point) {
                return point;
            }
            tries += 1;
            if tries > 1000 {
                // Fallback
                return point;
            }
        }
    }

    fn update_game(&mut self) {
        if self.game_over {
            return;
        }

        // Move snake
        let head = *self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => Point { x: head.x, y: head.y - 1 },
            Direction::Down => Point { x: head.x, y: head.y + 1 },
            Direction::Left => Point { x: head.x - 1, y: head.y },
            Direction::Right => Point { x: head.x + 1, y: head.y },
        };

        // Check wall collision
        if new_head.x < 0 || new_head.x >= GRID_SIZE || new_head.y < 0 || new_head.y >= GRID_SIZE {
            self.game_over = true;
            return;
        }

        // Check self collision
        if self.snake.contains(&new_head) {
            self.game_over = true;
            return;
        }

        self.snake.push_front(new_head);

        // Check food
        if new_head == self.food {
            self.score += 1;
            self.food = Game::spawn_food(&self.snake);
        } else {
            self.snake.pop_back();
        }
    }
}

impl event::EventHandler<ggez::GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let now = ctx.time.time_since_start().as_secs_f64();
        if now - self.last_update > self.update_interval {
            self.update_game();
            self.last_update = now;
        }
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        if self.game_over {
            if input.keycode == Some(KeyCode::R) {
                // Restart
                *self = Game::new(_ctx)?;
            }
            return Ok(());
        }
        let new_direction = match input.keycode {
            Some(KeyCode::Up) | Some(KeyCode::W) => Direction::Up,
            Some(KeyCode::Down) | Some(KeyCode::S) => Direction::Down,
            Some(KeyCode::Left) | Some(KeyCode::A) => Direction::Left,
            Some(KeyCode::Right) | Some(KeyCode::D) => Direction::Right,
            _ => return Ok(()),
        };
        // Prevent reversing
        if (self.direction == Direction::Up && new_direction != Direction::Down) ||
           (self.direction == Direction::Down && new_direction != Direction::Up) ||
           (self.direction == Direction::Left && new_direction != Direction::Right) ||
           (self.direction == Direction::Right && new_direction != Direction::Left) {
            self.direction = new_direction;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from_rgb(50, 10, 70)); // Darker purple background

        // Draw the window border (black)
        let window_border_rect = graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
        let window_border_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(5.0),
            window_border_rect,
            graphics::Color::BLACK,
        )?;
        canvas.draw(&window_border_mesh, graphics::DrawParam::default());

        // Draw the board background (same darker purple)
        let board_rect = graphics::Rect::new(PADDING, PADDING + 50.0, BOARD_SIZE, BOARD_SIZE);
        let board_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            board_rect,
            graphics::Color::from_rgb(50, 10, 70), // Same darker purple
        )?;
        canvas.draw(&board_mesh, graphics::DrawParam::default());

        // Draw the board border (bright green)
        let border_rect = graphics::Rect::new(
            PADDING - BORDER_WIDTH / 2.0,
            PADDING + 50.0 - BORDER_WIDTH / 2.0,
            BOARD_SIZE + BORDER_WIDTH,
            BOARD_SIZE + BORDER_WIDTH,
        );
        let border_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(BORDER_WIDTH),
            border_rect,
            graphics::Color::GREEN, // Bright green
        )?;
        canvas.draw(&border_mesh, graphics::DrawParam::default());

        // Draw snake
        for (i, &segment) in self.snake.iter().enumerate() {
            let x = PADDING + segment.x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
            let y = PADDING + 50.0 + segment.y as f32 * CELL_SIZE + CELL_SIZE / 2.0;
            let color = if i == 0 { graphics::Color::from_rgb(0, 200, 0) } else { graphics::Color::GREEN }; // Head slightly different
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [x, y],
                CELL_SIZE / 2.0 - 2.0,
                0.1,
                color,
            )?;
            canvas.draw(&circle, graphics::DrawParam::default());

            // Tongue for head
            if i == 0 {
                let tongue_end = match self.direction {
                    Direction::Up => [x, y - CELL_SIZE / 2.0],
                    Direction::Down => [x, y + CELL_SIZE / 2.0],
                    Direction::Left => [x - CELL_SIZE / 2.0, y],
                    Direction::Right => [x + CELL_SIZE / 2.0, y],
                };
                let tongue = graphics::Mesh::new_line(
                    ctx,
                    &[[x, y], tongue_end],
                    2.0,
                    graphics::Color::RED,
                )?;
                canvas.draw(&tongue, graphics::DrawParam::default());
                // Eye
                let eye_pos = match self.direction {
                    Direction::Up => [x + 3.0, y - 3.0],
                    Direction::Down => [x + 3.0, y + 3.0],
                    Direction::Left => [x - 3.0, y - 3.0],
                    Direction::Right => [x + 3.0, y - 3.0],
                };
                let eye = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    eye_pos,
                    2.0,
                    0.1,
                    graphics::Color::BLACK,
                )?;
                canvas.draw(&eye, graphics::DrawParam::default());            }
        }

        // Draw food (avocado shape: pear-like polygon)
        let food_x = PADDING + self.food.x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
        let food_y = PADDING + 50.0 + self.food.y as f32 * CELL_SIZE + CELL_SIZE / 2.0;
        let points = [
            [food_x, food_y - CELL_SIZE / 2.0], // top narrow
            [food_x + CELL_SIZE / 8.0, food_y - CELL_SIZE / 4.0],
            [food_x + CELL_SIZE / 3.0, food_y - CELL_SIZE / 8.0], // wider at middle
            [food_x + CELL_SIZE / 3.0, food_y + CELL_SIZE / 4.0],
            [food_x, food_y + CELL_SIZE / 2.0], // bottom point
            [food_x - CELL_SIZE / 3.0, food_y + CELL_SIZE / 4.0],
            [food_x - CELL_SIZE / 3.0, food_y - CELL_SIZE / 8.0],
            [food_x - CELL_SIZE / 8.0, food_y - CELL_SIZE / 4.0],
        ];
        let food_mesh = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::fill(),
            &points,
            graphics::Color::GREEN,
        )?;
        canvas.draw(&food_mesh, graphics::DrawParam::default());

        // Draw the title
        let title_text = graphics::Text::new("Avo Snek Game");
        canvas.draw(
            &title_text,
            graphics::DrawParam::default()
                .dest([SCREEN_WIDTH / 2.0 - 100.0, 20.0])
                .color(graphics::Color::GREEN) // Bright green
                .scale([2.0, 2.0]),
        );

        // Draw score
        let score_text = graphics::Text::new(format!("Score: {}", self.score));
        canvas.draw(
            &score_text,
            graphics::DrawParam::default()
                .dest([20.0, SCREEN_HEIGHT - 40.0])
                .color(graphics::Color::WHITE)
                .scale([1.5, 1.5]),
        );

        if self.game_over {
            let game_over_text = graphics::Text::new("Game Over! Press R to Restart");
            let dims = game_over_text.dimensions(ctx);
            let scale = 2.0;
            let x_pos = if let Some(rect) = dims {
                (SCREEN_WIDTH - rect.w as f32 * scale) / 2.0
            } else {
                SCREEN_WIDTH / 2.0 - 150.0
            };
            canvas.draw(
                &game_over_text,
                graphics::DrawParam::default()
                    .dest([x_pos, SCREEN_HEIGHT / 2.0])
                    .color(graphics::Color::RED)
                    .scale([scale, scale]),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("avo_snek", "avocado")
        .window_setup(ggez::conf::WindowSetup::default().title("Avo Snek"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT));
    let (mut ctx, event_loop) = cb.build()?;
    let game = Game::new(&mut ctx)?;
    event::run(ctx, event_loop, game);
}
