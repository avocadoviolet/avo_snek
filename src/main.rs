use ggez::{event, graphics, Context, GameResult};

const GRID_SIZE: i32 = 20;
const CELL_SIZE: f32 = 30.0;
const BOARD_SIZE: f32 = GRID_SIZE as f32 * CELL_SIZE;
const PADDING: f32 = 50.0;
const BORDER_WIDTH: f32 = 5.0;
const SCREEN_WIDTH: f32 = BOARD_SIZE + 2.0 * PADDING;
const SCREEN_HEIGHT: f32 = BOARD_SIZE + 2.0 * PADDING + 100.0; // Extra for title

struct Game {
    // For now, just the field
}

impl Game {
    fn new(_ctx: &mut Context) -> GameResult<Game> {
        Ok(Game {})
    }
}

impl event::EventHandler<ggez::GameError> for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Game logic will go here later
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

        // Draw the title (bright green)
        let title_text = graphics::Text::new("Avo Snek Game");
        canvas.draw(
            &title_text,
            graphics::DrawParam::default()
                .dest([SCREEN_WIDTH / 2.0 - 100.0, 20.0])
                .color(graphics::Color::GREEN) // Bright green
                .scale([2.0, 2.0]),
        );

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
