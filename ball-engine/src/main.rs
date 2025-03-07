#![allow(dead_code)]
use macroquad::{color::{ORANGE, RED}, prelude::{clear_background, draw_circle, draw_circle_lines, draw_line, draw_text, get_fps, is_key_pressed, is_mouse_button_down, mouse_position, next_frame, screen_height, screen_width, Color, KeyCode, MouseButton, BLACK, WHITE}};
use glam::{vec2, Vec2};

mod physics {
    pub mod solver;
    pub mod verlet;  // Make sure to add this!
}
use physics::{solver::Solver, verlet::Verlet};

use std::time::{SystemTime, UNIX_EPOCH};

use std::fs::File;
use std::io::Write;
use std::io::Read;
use serde_json::{json, Value};


fn get_time() -> u128 {
    return SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
}

fn write_data(info1: String, info2: String) {
    let mut contents = String::new();
    let mut data: Value = match File::open("output.json") {
        Ok(mut file) => {
            file.read_to_string(&mut contents).expect("Failed to read file");

            // Check if the file is empty
            if contents.trim().is_empty() {
                // Initialize with an empty object containing a "ball" array
                json!({ "ball1": [], "ball2": [] })
            } else {
                // Parse the JSON data
                serde_json::from_str(&contents).expect("Failed to parse JSON")
            }
        }
        Err(_) => {
            // If the file doesn't exist, create it with an empty object containing a "ball" array
            json!({ "ball1": [], "ball2": [] })
        }
    };

    // Append the ball's position to the JSON array
    if let Value::Object(ref mut object) = data {
        if let Some(Value::Array(ref mut ball_array)) = object.get_mut("ball1") {
            ball_array.push(json!(info1));
        } else {
            // If "ball" is not an array, replace it with an array containing the new position
            object.insert("ball1".to_string(), json!([format!("womp womp")]));
        }
    }
    if let Value::Object(ref mut object) = data {
        if let Some(Value::Array(ref mut ball_array)) = object.get_mut("ball2") {
            ball_array.push(json!(info2));
        } else {
            // If "ball" is not an array, replace it with an array containing the new position
            object.insert("ball2".to_string(), json!([format!("womp womp")]));
        }
    }

    // Write the updated JSON data back to the file
    let mut file = File::create("output.json").expect("Failed to create file");
    let json_string = serde_json::to_string_pretty(&data).expect("Failed to serialize JSON");
    file.write_all(json_string.as_bytes()).expect("Failed to write to file");

    println!("Determinism test complete");
}


#[macroquad::main("Game")]
async fn main() {
    // Initialize screen dimensions
    let screen_width = screen_width();
    let screen_height = screen_height();

    // Calculate constraint radius
    let constraint_radius = screen_height.min(screen_width) / 2.0 - 50.0;

    let mut solver = Solver::new(
        &[
            Verlet::new_with_radius(vec2(screen_width/3.0, 0.0),
            50.0),
            Verlet::new_with_radius(vec2(screen_width/3.0, screen_height/3.0),
            20.0),
            Verlet::new_with_radius(vec2(-screen_width/3.0, 0.0),
            50.0),
            Verlet::new_with_radius(vec2(-screen_width/3.0, -screen_height/3.0),
            20.0)
        ],
        vec2(0.0, -500.0),
        constraint_radius,
        77
    );
    if let Err(e) = solver.load_colors("colors.bin") {
        println!("Error loading colors: {}", e);
    }

    let dt = 16;  // 1 / 60.0 = 16.6 ms
    let ball_drop_dt = 100;
    let mouse_drop_dt = 100;
    let (mut accumulator, mut ball_drop_accumulator, mut mouse_drop_accumulator)  = (0, 0, 0);

    let mut last_time = get_time();
    let mut total_time: u128 = 0;
    let mut determinism_done = false;

    let mut balls_til_60_fps: usize = 0;
    let fps_threshold: i32 = 60;
    let measurement_frames: i32 = 30; // Number of frames to confirm slowdown
    let mut slow_frames: i32 = 0;

    // This is too force the simulation forward
    // for _ in 0..((60.0 / dt) as i32) {
    //     solver.update(dt);
        
    //     ball_drop_accumulator += dt;
    //     if ball_drop_accumulator >= ball_drop_dt && !solver.is_container_full() {
    //         solver.add_position(Verlet::new_with_velocity(vec2(1.0/2.2 * screen_width, screen_height / 8.0),
    //                 vec2(0.0, 200.0), dt));

    //         ball_drop_accumulator = 0.0;
    //     }
    // }
    
    loop {
        let current_time = get_time();
        let frame_time = current_time - last_time;
        last_time = current_time;
        
        accumulator += frame_time;
        ball_drop_accumulator += frame_time;
        mouse_drop_accumulator += frame_time;
        
        if is_mouse_button_down(MouseButton::Left) {
            if mouse_drop_accumulator >= mouse_drop_dt {
                let position = vec2(mouse_position().0, mouse_position().1) - vec2(screen_width / 2.0, screen_height / 2.0);
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
        
        if ball_drop_accumulator >= ball_drop_dt && !solver.is_container_full() {
            // let angle = rand::gen_range(0.0, std::f32::consts::TAU);
            // solver.add_position(Verlet::new_with_velocity(vec2(screen_width / 2.0, screen_height / 2.0),
            //         500.0 * vec2(angle.cos(), angle.sin()), dt));
            println!("screen_width: {}, screen_height: {}", screen_width, screen_height);
            let mut ball = Verlet::new_with_radius(vec2(0.15 * screen_width, screen_height * 2.0 / 7.0), 20.0);
            ball.set_velocity(vec2(0.0, 5.0), dt as f32 / 1000.0);
            solver.add_position(ball);
            ball_drop_accumulator = 0;
        }

        while accumulator >= dt {
            solver.update(dt as f32 / 1000.0);
            accumulator -= dt;
            total_time += dt;
            
            // if solver.is_container_full() {
                // println!("{}", solver.is_container_full());
            //     solver.apply_rainbow_gradient();
            // }
            let time_check = 1 * 1000;
            if total_time >= time_check && !determinism_done {
                let verlet = & solver.get_verlets()[0];
                let x_str = format!("{:.15}", verlet.get_position().x);
                let y_str = format!("{:.15}", verlet.get_position().y);
                let alpha_goal = time_check as f32 + 500.0;
                let alpha = (alpha_goal - total_time as f32) / (dt as f32);
                println!("{}", alpha_goal - total_time as f32);
                println!("{}", alpha);
                let x_str_inter = format!("{:.15}", verlet.get_interpolated_position(alpha).x);
                let y_str_inter = format!("{:.15}", verlet.get_interpolated_position(alpha).y);
                let position_str1 = format!("{}, {}: {} --- {}, {}: {}", x_str, y_str, total_time as f32 / 1000.0, x_str_inter, y_str_inter, alpha_goal / 1000.0);

                
                let verlet = & solver.get_verlets()[1];
                let x_str = format!("{:.15}", verlet.get_position().x);
                let y_str = format!("{:.15}", verlet.get_position().y);
                let alpha_goal = time_check as f32 + 500.0;
                let alpha = (alpha_goal - total_time as f32) / (dt as f32);
                println!("{}", alpha_goal - total_time as f32);
                println!("{}", alpha);
                let x_str_inter = format!("{:.15}", verlet.get_interpolated_position(alpha).x);
                let y_str_inter = format!("{:.15}", verlet.get_interpolated_position(alpha).y);
                let position_str2 = format!("{}, {}: {} --- {}, {}: {}", x_str, y_str, total_time as f32 / 1000.0, x_str_inter, y_str_inter, alpha_goal / 1000.0);

                write_data(position_str1, position_str2);
                determinism_done = true;
            }
        }
        
        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius, 1.0, WHITE);  // Draw constraint circle
        
        let alpha = accumulator as f32 / dt as f32;
        for verlet in solver.get_verlets() {
            // This is since the solver imagines the ball at being shows at 0, 0
            let origin = vec2(screen_width / 2.0, screen_height / 2.0);
            let interpolated_pos = origin + verlet.get_interpolated_position(alpha)  * Vec2::new(1.0, -1.0);
            let (x, y) = interpolated_pos.into();
            draw_circle(x, y, verlet.get_radius(), Color::from_rgba(
                verlet.get_color().x as u8,
                verlet.get_color().y as u8,
                verlet.get_color().z as u8,
                255
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
            135.0,
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
