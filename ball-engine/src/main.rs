#![allow(dead_code)]
use macroquad::prelude::{clear_background, draw_circle, draw_circle_lines, draw_line, draw_text, get_fps, is_key_pressed, is_mouse_button_down, mouse_position, next_frame, screen_height, screen_width, Color, KeyCode, MouseButton, BLACK, WHITE, ORANGE, RED};
use glam::{vec2, Vec2};

mod physics {
    pub mod solver;
    pub mod verlet; 
}
use physics::{solver::Solver, verlet::Verlet};

use std::time::{SystemTime, UNIX_EPOCH};

use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::vec;
use serde_json::{json, Value};

#[macroquad::main("Game")]
async fn main() {
    // Initialize screen dimensions
    let screen_width = screen_width();
    let screen_height = screen_height();

    // Calculate constraint radius
    let constraint_radius = screen_height.min(screen_width) / 2.0 - 50.0;

    let mut solver = Solver::new(
        &[
            // Verlet::new_with_radius(vec2(0.0, 0.0), 20.0),
            // Verlet::new_with_radius(vec2(70.0, 0.0), 20.0),
        ],
        vec2(0.0, -500.0),
        constraint_radius,
        8,
        8.0 * 2.5,
    );
    if let Err(e) = solver.load_colors("colors.bin") {
        println!("Error loading colors: {}", e);
    }

    let dt = 16;  // 1 / 60.0 = 16.6 ms
    let mut accumulator = 0;

    let mouse_drops_per_ms = 100;
    let mut mouse_drop_accumulator = 0;

    let ball_drop_per_ms = 10;
    let mut ball_drop_accumlator = 0;

    let mut last_time = get_time();
    let mut total_time: u128 = 0;
    
    // let mut print_data = false;

    let fps_threshold: i32 = 60;
    let measurement_frames: i32 = 30; // Number of frames to confirm slowdown
    let mut slow_frames_accumulator: i32 = 0;
    let mut balls_til_60_fps: usize = 0;
    
    loop {
        let current_time = get_time();
        let frame_time = current_time - last_time;
        let fps = get_fps();
        last_time = current_time;
        
        accumulator += frame_time;
        mouse_drop_accumulator += frame_time;

        while accumulator >= dt {
            solver.update(dt as f32 / 1000.0);
            accumulator -= dt;
            total_time += dt;
            ball_drop_accumlator += 1;

            if ball_drop_accumlator >= ball_drop_per_ms && !solver.is_container_full() {
                let mut ball = Verlet::new_with_radius(vec2(0.15 * screen_width, screen_height * 2.0 / 7.0), 8.0);
                ball.set_velocity(vec2(0.0, -10.0), dt as f32 / 1000.0);
                solver.add_position(ball);
                ball_drop_accumlator = 0;
            }

            // let time_check = 3 * 1000;
            // if total_time >= time_check && !print_data {
            //     for i in 0..1 {
            //         let verlet = &solver.get_verlets()[i];
                    
            //         let alpha_goal = time_check as f32 + 200.0;
            //         let alpha = (alpha_goal - total_time as f32) / (dt as f32);

            //         let (x, y) = verlet.get_position().into();
            //         let (x_inter, y_inter) = verlet.get_interpolated_position(alpha).into();

            //         let data = json!({
            //             "read_time": {
            //                 "time": total_time as f32 / 1000.0,
            //                 "x": x,
            //                 "y": y,
            //             },
            //             "goal_time": {
            //                 "time": alpha_goal / 1000.0,
            //                 "x_inter": x_inter,
            //                 "y_inter": y_inter,
            //             }
            //         });

            //         write_data(format!("{i}-ball"), data);
            //     }
            //     print_data = true;
            // }
        }

        if is_mouse_button_down(MouseButton::Left) {
            if mouse_drop_accumulator >= mouse_drops_per_ms {
                let origin = vec2(screen_width / 2.0, screen_height / 2.0);
                let position = origin + vec2(mouse_position().0, mouse_position().1) - vec2(screen_width / 2.0, screen_height / 2.0) * vec2(1.0, -1.0);
                solver.add_position(Verlet::new(position));  // Add new position at mouse position
                mouse_drop_accumulator = 0;
            };
        }
        if is_mouse_button_down(MouseButton::Right) {
            if let Err(e) = solver.color_from_image("churros.png") {
                println!("Error loading image: {}", e);
            }
        }
        if is_key_pressed(KeyCode::S) {
            if let Err(e) = solver.save_state("simulation_state.bin") {
                println!("Failed to save state: {}", e);
            } else {
                println!("State saved successfully!");
            }
        }
        if is_key_pressed(KeyCode::L) {
            match Solver::load_state("simulation_state.bin") {
                Ok(loaded_solver) => {
                    solver = loaded_solver;
                    println!("State loaded successfully!");
                }
                Err(e) => println!("Failed to load state: {}", e),
            }
        }
        if is_key_pressed(KeyCode::C) {
            if let Err(e) = solver.save_colors("colors.bin") {
                println!("Error saving colors: {}", e);
            } else {
                println!("Colors saved successfully!");
            }
        }
        if is_key_pressed(KeyCode::V) {
            if let Err(e) = solver.load_colors("colors.bin") {
                println!("Error loading colors: {}", e);
            } else {
                println!("Colors loaded successfully!");
            }
        }
        
        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius, 1.0, WHITE);  // Draw constraint circle
        
        let alpha = accumulator as f32 / dt as f32;
        for verlet in solver.get_verlets() {
            // This is since the solver imagines the ball at being shows at 0, 0
            let origin = vec2(screen_width / 2.0, screen_height / 2.0);
            let interpolated_pos = origin + verlet.get_interpolated_position(alpha)  * vec2(1.0, -1.0);
            let (x, y) = interpolated_pos.into();
            draw_circle(x, y, verlet.get_radius(), Color::from_rgba(
                verlet.get_color().x as u8,
                verlet.get_color().y as u8,
                verlet.get_color().z as u8,
                255
            ));
            // draw_arrow(
            //     interpolated_pos,
            //     interpolated_pos + verlet.get_velocity() * vec2(1.0, -1.0) / 5.0,
            //     ORANGE
            // );
            // draw_arrow(
            //     interpolated_pos,
            //     interpolated_pos + verlet.get_acceleration() * vec2(1.0, -1.0) / 5.0,
            //     RED
            // );
        }

        if get_fps() < fps_threshold && balls_til_60_fps == 0 {
            slow_frames_accumulator += 1;
            if slow_frames_accumulator >= measurement_frames {
                balls_til_60_fps = solver.get_verlets().len();
            }
        } else if balls_til_60_fps == 0 {
            slow_frames_accumulator = 0;
        }
        draw_texts(
            &[
                &format!(
                    "FPS: {fps:.0}",
                ),
                &format!(
                    "time: {:.3}", total_time as f32 / 1000.0
                ),
                &format!(
                    "Verlets: {}", solver.get_verlets().len()
                ),
                &format!(
                    "60 fps ball count: {balls_til_60_fps}"
                ),
            ],
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
    let arrowhead_angle = (30.0 as f32).to_radians();

    let left_arrowhead = vec2(
        end.x - arrowhead_length * (direction.x * arrowhead_angle.cos() - direction.y * arrowhead_angle.sin()),
        end.y - arrowhead_length * (direction.x * arrowhead_angle.sin() + direction.y * arrowhead_angle.cos()),
    );

    let right_arrowhead = vec2(
        end.x - arrowhead_length * (direction.x * arrowhead_angle.cos() + direction.y * arrowhead_angle.sin()),
        end.y - arrowhead_length * (-direction.x * arrowhead_angle.sin() + direction.y * arrowhead_angle.cos()),
    );

    // Draw the arrowhead
    draw_line(end.x, end.y, left_arrowhead.x, left_arrowhead.y, 2.0, color);
    draw_line(end.x, end.y, right_arrowhead.x, right_arrowhead.y, 2.0, color);
}

fn draw_texts(texts: &[&str], x: f32, y: f32, size: f32, color: Color) {
    for (i, text) in texts.iter().enumerate() {
        draw_text(text, x, y + i as f32 * size, size, color);
    }
}

fn get_time() -> u128 {
    return SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
}

fn write_data(index: String, data: Value) {
    let mut contents = String::new();
    let data = match File::open("output.json") {
        Ok(mut file) => {
            file.read_to_string(&mut contents).expect("Failed to read file");
            // Check if the file is empty
            if contents.trim().is_empty() {
                // Initialize with an empty object containing a "ball" array
                json!({ index: [data] })
            } else {
                // Parse the JSON data
                let mut json_data: Value = serde_json::from_str(&contents).expect("Failed to parse JSON");

                if let Value::Object(ref mut object) = json_data {
                    if let Some(Value::Array(ref mut array)) = object.get_mut(&index) {
                        // If the index already exists and is an array, append the data to it
                        array.push(data);
                    } else {
                        // If the index doesn't exist or isn't an array, create a new array with the data
                        object.insert(index, json!([data]));
                    }
                    json_data
                } else {
                    // If the JSON isn't an object, create a new one with an array at the specified index
                    json!({ index: [data] })
                }
            }
        }
        Err(_) => {
            // If the file doesn't exist, create it with an empty object containing a "ball" array
            json!({ index: [data] })
        }
    };

    let mut file = File::create("output.json").expect("Failed to create file");
    let json_string = serde_json::to_string_pretty(&data).expect("Failed to serialize JSON");
    file.write_all(json_string.as_bytes()).expect("Failed to write to file");

    println!("Determinism test complete");
}
