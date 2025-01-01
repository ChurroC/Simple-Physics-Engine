use macroquad::prelude::*;
pub mod verlet_object;

#[macroquad::main("Game")]
async fn main() {
    loop {
        // Press Ctrl+R to let bacon reload
        if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::R) {
            std::process::exit(0);
        }

        draw_circle(200.0, 150.0, 50.0, BLUE);

        next_frame().await;
    }
}