use macroquad::prelude::*;


mod physics {
    pub mod solver;
    pub mod verlet;  // Make sure to add this!
}
use physics::solver::Solver;

#[macroquad::main("Game")]
async fn main() {
    // Multiple positions 
    let mut solver = Solver::new(&[
        Vec2::new(0.0, 0.0),
        Vec2::new(200.0, 200.0),
        Vec2::new(400.0, 300.0)
    ],
    Vec2::new(0.0, 1000.0),
    Vec2::new(screen_width() / 2.0, screen_height() / 2.0),
    screen_height().min(screen_width()) / 2.0 - 50.0
    );

    // Add single position
    solver.add_positions(&[Vec2::new(150.0, 150.0)]);

    // Add multiple positions
    solver.add_positions(&[
        Vec2::new(300.0, 300.0),
        Vec2::new(400.0, 400.0)
    ]);

    loop {
        clear_background(BLACK);

        solver.update(1.0/get_fps() as f32);  // Update physics
    
        // Draw all verlet objects
        for pos in solver.get_positions() {
            draw_circle(pos.x, pos.y, 10.0, BLUE);
        }

        next_frame().await;
    }
}