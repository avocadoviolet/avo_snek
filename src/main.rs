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

#[derive(Clone, Copy, PartialEq)]
enum LevelType {
    Normal,
    Ghost,
    MultiFood,
    Speed,
}


struct Game {
    snake: LinkedList<Point>,
    direction: Direction,
    food: Point,
    score: u32,
    game_over: bool,
    last_update: f64,
    update_interval: f64,
    level: u32,
    level_type: LevelType,
    food_items: Vec<Point>, // Multiple food for level 10+
}

impl Game {
    fn new(_ctx: &mut Context) -> GameResult<Game> {
        let mut snake = LinkedList::new();
        snake.push_back(Point { x: GRID_SIZE / 2, y: GRID_SIZE / 2 });
        let food = Game::spawn_food(&snake);
        let level = 1;
        let level_type = Game::get_level_type(level);        
        let update_interval = 0.15 - (level as f64 - 1.0) * 0.01;
        let food_items = vec![food];
        Ok(Game {
            snake,
            direction: Direction::Right,
            food,
            score: 0,
            game_over: false,
            last_update: 0.0,
            update_interval,
            level,
            level_type,
            food_items,
        })
    }

    fn get_level_type(level: u32) -> LevelType {
        match level {
            5 => LevelType::Ghost,
            10 => LevelType::MultiFood,
            15 => LevelType::Speed,
            _ => LevelType::Normal,
        }
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

        // Special Level 5: Ghost mode - can walk through walls
        let is_ghost_mode = self.level_type == LevelType::Ghost;
        
        // Check wall collision (unless ghost mode)
        if !is_ghost_mode && (new_head.x < 0 || new_head.x >= GRID_SIZE || new_head.y < 0 || new_head.y >= GRID_SIZE) {
            self.game_over = true;
            return;
        }
        
        // Wrap around for ghost mode
        let mut wrapped_head = new_head;
        if is_ghost_mode {
            wrapped_head.x = (wrapped_head.x + GRID_SIZE) % GRID_SIZE;
            wrapped_head.y = (wrapped_head.y + GRID_SIZE) % GRID_SIZE;
        }

        // Check self collision
        if self.snake.contains(&wrapped_head) {
            self.game_over = true;
            return;
        }

        self.snake.push_front(wrapped_head);

        // Check food - check against all food items
        let mut ate_food = false;
        for (idx, &food) in self.food_items.iter().enumerate() {
            if wrapped_head == food {
                ate_food = true;
                self.food_items.remove(idx);
                break;
            }
        }

        if ate_food {
            self.score += 1;
            
            // Regenerate food
            if self.food_items.is_empty() {
                self.food = Game::spawn_food(&self.snake);
                self.food_items = vec![self.food];
                
                // Level 10: Multi-food mode - spawn multiple food
                if self.level_type == LevelType::MultiFood {
                    let num_foods = 1 + (self.level / 5) as usize;
                    for _ in 1..num_foods {
                        if let Some(new_food) = self.try_spawn_food(50) {
                            self.food_items.push(new_food);
                        }
                    }
                }
            }
            
            // Level up at 20 points
            if self.score % 2 == 0 {
                self.level += 1;
                self.level_type = Game::get_level_type(self.level);                
                self.update_interval = (0.15 - (self.level as f64 - 1.0) * 0.01).max(0.05);
                if self.level_type == LevelType::Speed {
                    self.update_interval *= 0.5;
                }
            }
        } else {
            self.snake.pop_back();
        }
    }
    
    fn try_spawn_food(&self, max_tries: i32) -> Option<Point> {
        let mut rng = rand::thread_rng();
        for _ in 0..max_tries {
            let x = rng.gen_range(0..GRID_SIZE);
            let y = rng.gen_range(0..GRID_SIZE);
            let point = Point { x, y };
            if !self.snake.contains(&point) && !self.food_items.contains(&point) {
                return Some(point);
            }
        }
        None
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
        let bg_color = if self.level_type != LevelType::Normal {
            graphics::Color::from_rgb(200, 150, 220) // Light purple for special levels
        } else {
            graphics::Color::from_rgb(50, 10, 70) // Darker purple
        };
        let mut canvas = graphics::Canvas::from_frame(ctx, bg_color);

        // Draw the window border (black)
        let window_border_rect = graphics::Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT);
        let window_border_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(5.0),
            window_border_rect,
            graphics::Color::BLACK,
        )?;
        canvas.draw(&window_border_mesh, graphics::DrawParam::default());

        // Draw the board background
        let board_rect = graphics::Rect::new(PADDING, PADDING + 50.0, BOARD_SIZE, BOARD_SIZE);
        let board_bg = if self.level_type != LevelType::Normal {
            graphics::Color::from_rgb(200, 150, 220) // Light purple for special
        } else {
            graphics::Color::from_rgb(50, 10, 70) // Darker purple
        };
        let board_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            board_rect,
            board_bg,
        )?;
        canvas.draw(&board_mesh, graphics::DrawParam::default());

        // Draw the board border (bright green)
        let border_rect = graphics::Rect::new(
            PADDING,
            PADDING + 50.0,
            BOARD_SIZE,
            BOARD_SIZE,
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
            let color = if self.level_type != LevelType::Normal {
                if i == 0 { graphics::Color::from_rgb(100, 20, 150) } else { graphics::Color::from_rgb(50, 10, 70) } // Dark purple for special
            } else {
                if i == 0 { graphics::Color::from_rgb(0, 200, 0) } else { graphics::Color::GREEN } // Normal green
            };
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [x, y],
                CELL_SIZE / 2.0 - 2.0,
                0.1,
                color,
            )?;
            canvas.draw(&circle, graphics::DrawParam::default());

            // Forked tongue and eye for head
            if i == 0 {
                let (tongue_main, tongue_fork1, tongue_fork2) = match self.direction {
                    Direction::Up => (
                        [x, y - CELL_SIZE / 2.0 - CELL_SIZE / 4.0],
                        [x - CELL_SIZE / 4.0, y - CELL_SIZE / 2.0 - CELL_SIZE / 3.0],
                        [x + CELL_SIZE / 4.0, y - CELL_SIZE / 2.0 - CELL_SIZE / 3.0],
                    ),
                    Direction::Down => (
                        [x, y + CELL_SIZE / 2.0 + CELL_SIZE / 4.0],
                        [x - CELL_SIZE / 4.0, y + CELL_SIZE / 2.0 + CELL_SIZE / 3.0],
                        [x + CELL_SIZE / 4.0, y + CELL_SIZE / 2.0 + CELL_SIZE / 3.0],
                    ),
                    Direction::Left => (
                        [x - CELL_SIZE / 2.0 - CELL_SIZE / 4.0, y],
                        [x - CELL_SIZE / 2.0 - CELL_SIZE / 3.0, y - CELL_SIZE / 4.0],
                        [x - CELL_SIZE / 2.0 - CELL_SIZE / 3.0, y + CELL_SIZE / 4.0],
                    ),
                    Direction::Right => (
                        [x + CELL_SIZE / 2.0 + CELL_SIZE / 4.0, y],
                        [x + CELL_SIZE / 2.0 + CELL_SIZE / 3.0, y - CELL_SIZE / 4.0],
                        [x + CELL_SIZE / 2.0 + CELL_SIZE / 3.0, y + CELL_SIZE / 4.0],
                    ),
                };

                // Main tongue
                let tongue_main_line = graphics::Mesh::new_line(
                    ctx,
                    &[[x, y], tongue_main],
                    2.0,
                    graphics::Color::RED,
                )?;
                canvas.draw(&tongue_main_line, graphics::DrawParam::default());

                // Fork 1 (outward)
                let tongue_fork1_line = graphics::Mesh::new_line(
                    ctx,
                    &[tongue_main, tongue_fork1],
                    2.0,
                    graphics::Color::RED,
                )?;
                canvas.draw(&tongue_fork1_line, graphics::DrawParam::default());

                // Fork 2 (outward)
                let tongue_fork2_line = graphics::Mesh::new_line(
                    ctx,
                    &[tongue_main, tongue_fork2],
                    2.0,
                    graphics::Color::RED,
                )?;
                canvas.draw(&tongue_fork2_line, graphics::DrawParam::default());

                // Eye (bigger and spaced from tongue)
                let eye_pos = match self.direction {
                    Direction::Up => [x - 5.0, y - 5.0],
                    Direction::Down => [x - 5.0, y + 5.0],
                    Direction::Left => [x - 5.0, y - 5.0],
                    Direction::Right => [x + 5.0, y - 5.0],
                };
                let eye = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    eye_pos,
                    3.5,
                    0.1,
                    graphics::Color::BLACK,
                )?;
                canvas.draw(&eye, graphics::DrawParam::default());
            }
        }

        // Draw food (avocado shape: two stacked circles with brown border, green interior, brown pit)
        for &food in &self.food_items {
            let food_x = PADDING + food.x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
            let food_y = PADDING + 50.0 + food.y as f32 * CELL_SIZE + CELL_SIZE / 2.0;
            
            // Brown outer border (larger circles)
            let top_brown = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [food_x, food_y - CELL_SIZE / 6.0],
                CELL_SIZE / 3.0,
                0.1,
                graphics::Color::from_rgb(139, 69, 19), // Brown
            )?;
            canvas.draw(&top_brown, graphics::DrawParam::default());
            
            let bottom_brown = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [food_x, food_y + CELL_SIZE / 6.0],
                CELL_SIZE / 2.3,
                0.1,
                graphics::Color::from_rgb(139, 69, 19), // Brown
            )?;
            canvas.draw(&bottom_brown, graphics::DrawParam::default());
            
            // Green interior
            let top_green = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [food_x, food_y - CELL_SIZE / 6.0],
                CELL_SIZE / 3.5,
                0.1,
                graphics::Color::GREEN,
            )?;
            canvas.draw(&top_green, graphics::DrawParam::default());
            
            let bottom_green = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [food_x, food_y + CELL_SIZE / 6.0],
                CELL_SIZE / 2.6,
                0.1,
                graphics::Color::GREEN,
            )?;
            canvas.draw(&bottom_green, graphics::DrawParam::default());
            
            // Brown pit in the center
            let pit = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [food_x, food_y],
                CELL_SIZE / 6.0,
                0.1,
                graphics::Color::from_rgb(139, 69, 19), // Brown
            )?;
            canvas.draw(&pit, graphics::DrawParam::default());
        }

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
        let score_text = graphics::Text::new(format!("Score: {}  Level: {}", self.score, self.level));
        canvas.draw(
            &score_text,
            graphics::DrawParam::default()
                .dest([20.0, SCREEN_HEIGHT - 40.0])
                .color(graphics::Color::WHITE)
                .scale([1.5, 1.5]),
        );

        // Draw special level indicator
        if self.level_type != LevelType::Normal && !self.game_over {
            let level_text = match self.level_type {
                LevelType::Ghost => "GHOST MODE - Walk Through Walls!",
                LevelType::MultiFood => "MULTI-FOOD MODE!",
                LevelType::Speed => "EXTREME SPEED!",
                LevelType::Normal => "",
            };
            let special_text = graphics::Text::new(level_text);
            canvas.draw(
                &special_text,
                graphics::DrawParam::default()
                    .dest([SCREEN_WIDTH / 2.0 - 100.0, 75.0])
                    .color(graphics::Color::YELLOW)
                    .scale([1.0, 1.0]),
            );
        }

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
