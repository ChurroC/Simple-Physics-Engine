#![allow(dead_code)]
use macroquad::prelude::*;

mod physics {
    pub mod solver;
    pub mod verlet;  // Make sure to add this!
}
use physics::solver::Solver;

#[macroquad::main("Game")]
async fn main() {
    // Initialize screen dimensions
    let screen_width = screen_width();
    let screen_height = screen_height();

    // Calculate constraint radius
    let constraint_radius = screen_height.min(screen_width) / 2.0 - 50.0;

    let subset = 10;
    let mut solver = Solver::new(
        &[
            
        ],
        Vec2::new(0.0, 500.0),
        Vec2::new(screen_width / 2.0, screen_height / 2.0),
        constraint_radius,
        subset,
    );

    let mut last_time = get_time();
    
    loop {
        let current_time = get_time();
        let delta_time = (current_time - last_time) as f32;
        last_time = current_time;

        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius, 1.0, WHITE);  // Draw constraint circle

        if is_mouse_button_pressed(MouseButton::Left) {
            solver.add_position(Vec2::new(mouse_position().0, mouse_position().1));  // Add new position at mouse position
        }

        let physics_start = get_time();
        solver.update(delta_time);
        let physics_end = get_time();
        let physics_duration = physics_end - physics_start;

    
        // Draw all verlet objects
        for pos in solver.get_positions() {
            draw_circle(pos.x, pos.y, 10.0, BLUE);
        }

        // Enhanced debug display
        draw_text(
            &format!(
                "FPS: {}\nFrame Time: {:.4}ms\nPhysics Time: {:.4}ms", 
                get_fps(),
                delta_time * 1000.0,
                physics_duration * 1000.0
            ),
            20.0,
            30.0,
            30.0,
            WHITE
        );
        next_frame().await;
    }
}