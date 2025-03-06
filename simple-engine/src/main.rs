#![allow(dead_code)]

use macroquad::prelude::{clear_background, draw_circle, draw_circle_lines, draw_line, draw_text, get_fps, get_time, is_mouse_button_down, mouse_position, next_frame, screen_height, screen_width, MouseButton, BLACK, WHITE, ORANGE, RED, Color};
use glam::DVec2;

use core::time;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use serde_json::{json, Value};

use std::time::{SystemTime, UNIX_EPOCH};


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
    let constraint_radius = screen_height.min(screen_width) as f64 / 2.0 - 50.0;

    let mut solver = Solver::new(
        &[
            Verlet::new_with_radius(DVec2::new(0.0, 180.0),
            20.0),
        ],
        DVec2::new(0.0, -500.0),
        constraint_radius,
    );

    let dt: u128 = 1;  // Fixed dt in milliseconds (approximately 0.55ms)
    println!("dt: {}", dt);
    let ball_drop_dt = 100;         // 0.1 seconds in milliseconds
    let mouse_drop_dt = 100;  
    let (mut accumulator, mut ball_drop_accumulator, mut mouse_drop_accumulator): (u128, u128, u128) = (0, 0, 0);


    let mut last_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

    let mut total_time: u128 = 0;

    let mut balls_til_60_fps = 0;
    let fps_threshold: i32 = 60;
    let measurement_frames: i32 = 30; // Number of frames to confirm slowdown
    let mut slow_frames: i32 = 0;

    let mut determinism_done = false;
    
    loop {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let frame_time = current_time - last_time;
        println!("Frame time: {}\r", frame_time);
        last_time = current_time;
        
        accumulator += frame_time;
        ball_drop_accumulator += frame_time;
        mouse_drop_accumulator += frame_time;
        
        if is_mouse_button_down(MouseButton::Left) {
            if mouse_drop_accumulator >= mouse_drop_dt {
                let position = DVec2::new(mouse_position().0.into(), mouse_position().1.into()) - DVec2::new(screen_width as f64 / 2.0, screen_height as f64 / 2.0);
                solver.add_position(Verlet::new(position));  // Add new position at mouse position
                mouse_drop_accumulator = 0;
            };
        }
        
        if ball_drop_accumulator >= ball_drop_dt && !solver.is_container_full() {
            let mut ballz = Verlet::new_with_radius(DVec2::new(0.15 * (screen_width as f64),0.0), 20.0);
            ballz.set_velocity(DVec2::new(0.0, 200.0), dt as f64 / 1000.0);
            // solver.add_position(ballz);
            ball_drop_accumulator = 0;
        }

        if total_time >= 800 && !determinism_done {
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
                    let x_str = format!("{:.15}", solver.get_verlets()[0].get_position().x);
                    let y_str = format!("{:.15}", solver.get_verlets()[0].get_position().y);
                    let x_str_inter = format!("{:.15}", solver.get_verlets()[0].get_interpolated_position((0.9-total_time as f64) / 0.815).x);
                    let y_str_inter = format!("{:.15}", solver.get_verlets()[0].get_interpolated_position((0.9-total_time as f64) / 0.815).y);
                    let position_str = format!("{}, {}: {} --- {}, {}: {}", x_str, y_str, total_time, x_str_inter, y_str_inter, "0.815");
                    ball_array.push(json!(position_str));
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

        println!("wow");
        while accumulator >= dt {
            solver.update(dt as f64 / 1000.0);  // Convert to seconds only when passing to physics
            accumulator -= dt;
            total_time += dt;
        }
        println!("wow");
        
        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius as f32, 1.0, WHITE);  // Draw constraint circle
        
        let alpha = accumulator / dt;
        for verlet in solver.get_verlets() {
            // This is since the solver imagines the ball at being shows at 0, 0
            let origin = DVec2::new(screen_width as f64 / 2.0, screen_height as f64 / 2.0);
            let interpolated_pos = origin + verlet.get_interpolated_position(alpha as f64) * DVec2::new(1.0, -1.0);
            let x = interpolated_pos.x as f32;
            let y = interpolated_pos.y as f32;
            draw_circle(x, y, verlet.get_radius() as f32, Color::from_rgba(
                255, 255, 255, 255
            ));
            draw_arrow(
                interpolated_pos,
                interpolated_pos + verlet.get_velocity() * DVec2::new(1.0, -1.0) / 5.0,
                ORANGE
            );
            draw_arrow(
                interpolated_pos,
                interpolated_pos + verlet.get_acceleration() * DVec2::new(1.0, -1.0) / 5.0,
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
                "time: {total_time:.4}"
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

fn draw_arrow(start: DVec2, end: DVec2, color: Color) {
    // Draw the shaft of the arrow
    draw_line(start.x as f32, start.y as f32, end.x as f32, end.y as f32, 2.0, color);

    // Calculate the direction vector
    let direction = (end - start).normalize();

    // Calculate the points for the arrowhead
    let arrowhead_length = 10.0;
    let arrowhead_angle = (30.0 as f64).to_radians();

    let left_arrowhead = DVec2::new(
        end.x - arrowhead_length * (direction.x * arrowhead_angle.cos() - direction.y * arrowhead_angle.sin()),
        end.y - arrowhead_length * (direction.x * arrowhead_angle.sin() + direction.y * arrowhead_angle.cos()),
    );

    let right_arrowhead = DVec2::new(
        end.x - arrowhead_length * (direction.x * arrowhead_angle.cos() + direction.y * arrowhead_angle.sin()),
        end.y - arrowhead_length * (-direction.x * arrowhead_angle.sin() + direction.y * arrowhead_angle.cos()),
    );

    // Draw the arrowhead
    draw_line(end.x as f32, end.y as f32, left_arrowhead.x as f32, left_arrowhead.y as f32, 2.0, color);
    draw_line(end.x as f32, end.y as f32, right_arrowhead.x as f32, right_arrowhead.y as f32, 2.0, color);
}
