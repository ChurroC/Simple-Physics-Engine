#![allow(dead_code)]
use macroquad::prelude::{clear_background, draw_circle, draw_circle_lines, draw_line, draw_text, get_fps, get_time, is_mouse_button_down, mouse_position, next_frame, screen_height, screen_width, MouseButton, BLACK, WHITE, ORANGE, RED, Color};
use glam::Vec2;

use std::fs::File;
use std::io::Write;
use std::io::Read;
use serde_json::{json, Value};

mod physics {
    pub mod solver;
    pub mod verlet;
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
            Verlet::new_with_radius(Vec2::new(0.0, 0.0),
            20.0),
        ],
        Vec2::new(0.0, -500.0),
        constraint_radius,
    );

    let dt: f32 = 1.0 / 60.0 / 10.0;  // Fixed 60 FPS physics update - With 8 subdivisions - used for all testing
    let ball_drop_dt = 0.1;
    let mouse_drop_dt = 0.1;
    let (mut accumulator, mut ball_drop_accumulator,mut mouse_drop_accumulator)  = (0.0, 0.0, 0.0);

    let mut last_time: f64 = get_time();

    let mut balls_til_60_fps = 0;
    let fps_threshold: i32 = 60;
    let measurement_frames: i32 = 30; // Number of frames to confirm slowdown
    let mut slow_frames: i32 = 0;

    let mut accumlator_determinism = 0.0;
    let mut determinism_done = false;
    
    loop {
        let current_time = get_time();
        let frame_time = (current_time - last_time) as f32;
        last_time = current_time;
        
        accumulator += frame_time;
        ball_drop_accumulator += frame_time;
        mouse_drop_accumulator += frame_time;
        accumlator_determinism += frame_time;
        
        if is_mouse_button_down(MouseButton::Left) {
            if mouse_drop_accumulator >= mouse_drop_dt {
                let position = Vec2::new(mouse_position().0, mouse_position().1) - Vec2::new(screen_width / 2.0, screen_height / 2.0);
                solver.add_position(Verlet::new(position));  // Add new position at mouse position
                mouse_drop_accumulator = 0.0;
            };
        }
        
        if ball_drop_accumulator >= ball_drop_dt && !solver.is_container_full() {
            let mut ballz = Verlet::new_with_radius(Vec2::new(0.15 * screen_width,0.0), 20.0);
            ballz.set_velocity(Vec2::new(0.0, 200.0), dt);
            // solver.add_position(ballz);
            ball_drop_accumulator = 0.0;
        }

        if accumlator_determinism >= 3.0 && !determinism_done {
            // Try to open the file and read its contents
            let mut contents = String::new();
            let mut data: Value = match File::open("output.json") {
                Ok(mut file) => {
                    file.read_to_string(&mut contents).expect("Failed to read file");
        
                    // Check if the file is empty
                    if contents.trim().is_empty() {
                        // Initialize with an empty object containing a "ball" array
                        json!({ "ball": [] })
                    } else {
                        // Parse the JSON data
                        serde_json::from_str(&contents).expect("Failed to parse JSON")
                    }
                }
                Err(_) => {
                    // If the file doesn't exist, create it with an empty object containing a "ball" array
                    json!({ "ball": [] })
                }
            };
            println!("{:?}", data);
        
            // Append the ball's position to the JSON array
            let (x, y) = solver.get_verlets()[0].get_position().into();
            if let Value::Object(ref mut object) = data {
                if let Some(Value::Array(ref mut ball_array)) = object.get_mut("ball") {
                    ball_array.push(json!(format!("{}, {}", x, y)));
                } else {
                    // If "ball" is not an array, replace it with an array containing the new position
                    object.insert("ball".to_string(), json!([format!("{}, {}", x, y)]));
                }
            }
            println!("{:?}", data);
        
            // Write the updated JSON data back to the file
            let mut file = File::create("output.json").expect("Failed to create file");
            let json_string = serde_json::to_string_pretty(&data).expect("Failed to serialize JSON");
            file.write_all(json_string.as_bytes()).expect("Failed to write to file");
        
            determinism_done = true;
            println!("Determinism test complete");
        }

        while accumulator >= dt {
            solver.update(dt);
            accumulator -= dt;
        }
        
        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius, 1.0, WHITE);  // Draw constraint circle
        
        let alpha = accumulator / dt;
        for verlet in solver.get_verlets() {
            // This is since the solver imagines the ball at being shows at 0, 0
            let origin = Vec2::new(screen_width / 2.0, screen_height / 2.0);
            let interpolated_pos = origin + verlet.get_interpolated_position(alpha) * Vec2::new(1.0, -1.0);
            let (x, y) = interpolated_pos.into();
            draw_circle(x, y, verlet.get_radius(), Color::from_rgba(
                255, 255, 255, 255
            ));
            draw_arrow(
                interpolated_pos,
                interpolated_pos + verlet.get_velocity() * Vec2::new(1.0, -1.0) / 5.0,
                ORANGE
            );
            draw_arrow(
                interpolated_pos,
                interpolated_pos + verlet.get_acceleration() * Vec2::new(1.0, -1.0) / 5.0,
                RED
            );
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
        draw_text(
            &format!(
                "Verlets: {}", 
                solver.get_verlets().len(),
            ),
            20.0,
            65.0,
            30.0,
            WHITE
        );
        draw_text(
            &format!(
                "dt: {:.4}", 
                dt,
            ),
            20.0,
            100.0,
            30.0,
            WHITE
        );
        draw_text(
            &format!(
                "time: {accumlator_determinism:.4}"
            ),
            20.0,
            135.0,
            30.0,
            WHITE
        );
        
        // In your main loop:
        if get_fps() < fps_threshold && balls_til_60_fps == 0 {  // Only track if we haven't found threshold
            slow_frames += 1;
            if slow_frames >= measurement_frames {
                balls_til_60_fps = solver.get_verlets().len();
            }
        } else if balls_til_60_fps == 0 {  // Only reset if we haven't found threshold
            slow_frames = 0;  // Reset if FPS recovers
        }
        draw_text(
            &format!(
                "60 fps ball count: {}", 
                balls_til_60_fps,
            ),
            20.0,
            170.0,
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
    let arrowhead_angle = (30.0 as f32).to_radians();

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
