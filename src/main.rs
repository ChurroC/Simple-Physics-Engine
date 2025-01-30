#![allow(dead_code)]
use macroquad::prelude::*;

mod physics {
    pub mod solver;
    pub mod verlet;  // Make sure to add this!
}
use physics::{solver::Solver, verlet::Verlet};

#[macroquad::main("Game")]
async fn main() {
    // Initialize screen dimensions
    let screen_width = screen_width();
    let screen_height = screen_height();

    // Calculate constraint radius
    let constraint_radius = screen_height.min(screen_width) / 2.0 - 50.0;

    let mut solver = Solver::new(
        &[
            Verlet::new(Vec2::new(screen_width / 2.0, screen_height / 2.0)),
            Verlet::new(Vec2::new(screen_width / 2.0, 0.0)),
        ],
        Vec2::new(0.0, 100.0),
        Vec2::new(screen_width / 2.0, screen_height / 2.0),
        constraint_radius,
    );

    let dt = 1.0 / 60.0 / 8.0;  // Fixed 60 FPS physics update - With 8 subdivisions
    let mut accumulator = 0.0;
    // let mut ball_drop_accumulator = 0.0;
    let mut last_time: f64 = get_time();
    
    loop {
        let current_time = get_time();
        let frame_time = (current_time - last_time) as f32;
        last_time = current_time;
        accumulator += frame_time;
        ball_drop_accumulator += frame_time;
        
        if is_mouse_button_pressed(MouseButton::Left) {
            solver.add_position(Verlet::new(Vec2::new(mouse_position().0, mouse_position().1)));  // Add new position at mouse position
        }
        
        // if ball_drop_accumulator >= 0.1 {
        //     let angle = rand::gen_range(0.0, std::f32::consts::TAU);
        //     solver.add_position(Verlet::new_with_velocity(Vec2::new(screen_width / 2.0, screen_height / 2.0),
        //             500.0 * Vec2::new(angle.cos(), angle.sin()), dt));
        //     ball_drop_accumulator = 0.0;
        // }

        while accumulator >= dt {
            solver.update(dt);
            accumulator -= dt;
        }
        
        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius, 1.0, WHITE);  // Draw constraint circle
        for verlet in solver.get_verlets() {
            let (x, y) = verlet.get_position().into();
            draw_circle(x, y, 10.0,  Color::from_rgba(verlet.get_color().x as u8, verlet.get_color().y as u8, verlet.get_color().z as u8, 255));
            draw_arrow(verlet.get_position(), verlet.get_position() + verlet.get_velocity() / 5.0, ORANGE);
            draw_arrow(verlet.get_position(), verlet.get_position() + verlet.get_acceleration() / 5.0, RED);
        }


    

        // Enhanced debug display
        draw_text(
            &format!(
                "FPS: {}", 
                get_fps(),
            ),
            20.0,
            30.0,
            30.0,
            WHITE
        );
        next_frame().await;
    }
}

fn draw_arrow(start: Vec2, end: Vec2, color: Color) {
    // Draw the shaft of the arrow
    draw_line(start.x, start.y, end.x, end.y, 2.0, color);

    // Calculate the direction vector
    let direction = (end - start).normalize();

    // Calculate the points for the arrowhead
    let arrowhead_length = 10.0;
    let arrowhead_angle = 30.0f32.to_radians();

    let left_arrowhead = Vec2::new(
        end.x - arrowhead_length * (direction.x * arrowhead_angle.cos() - direction.y * arrowhead_angle.sin()),
        end.y - arrowhead_length * (direction.x * arrowhead_angle.sin() + direction.y * arrowhead_angle.cos()),
    );

    let right_arrowhead = Vec2::new(
        end.x - arrowhead_length * (direction.x * arrowhead_angle.cos() + direction.y * arrowhead_angle.sin()),
        end.y - arrowhead_length * (-direction.x * arrowhead_angle.sin() + direction.y * arrowhead_angle.cos()),
    );

    // Draw the arrowhead
    draw_line(end.x, end.y, left_arrowhead.x, left_arrowhead.y, 2.0, color);
    draw_line(end.x, end.y, right_arrowhead.x, right_arrowhead.y, 2.0, color);
}
