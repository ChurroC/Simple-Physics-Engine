#![allow(dead_code)]
use macroquad::prelude::{clear_background, draw_circle, draw_circle_lines, get_time, is_key_pressed, is_mouse_button_down, mouse_position, next_frame, screen_height, screen_width, draw_text, draw_line, get_fps, BLACK, Color, MouseButton, KeyCode, WHITE};
use glam::{vec2, Vec2};

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
            Verlet::new_with_radius(Vec2::new(screen_width/2.0, screen_height/2.0),
            50.0),
            Verlet::new_with_radius(Vec2::new(screen_width/2.0, screen_height/2.0),
            5.0)
        ],
        Vec2::new(0.0, 500.0),
        constraint_radius,
    );
    if let Err(e) = solver.load_colors("colors.bin") {
        println!("Error loading colors: {}", e);
    }

    let dt = 1.0 / 60.0 / 20.0;  // Fixed 60 FPS physics update - With 8 subdivisions - used for all testing
    let ball_drop_dt = 0.1;
    let mouse_drop_dt = 0.1;
    let (mut accumulator, mut ball_drop_accumulator,mut mouse_drop_accumulator)  = (0.0, 0.0, 0.0);

    let mut last_time: f64 = get_time();

    let mut balls_til_60_fps = 0;
    let fps_threshold: i32 = 60;
    let measurement_frames: i32 = 30; // Number of frames to confirm slowdown
    let mut slow_frames: i32 = 0;

    // This is too force the simulation forward
    // for _ in 0..((60.0 / dt) as i32) {
    //     solver.update(dt);
        
    //     ball_drop_accumulator += dt;
    //     if ball_drop_accumulator >= ball_drop_dt && !solver.is_container_full() {
    //         solver.add_position(Verlet::new_with_velocity(Vec2::new(1.0/2.2 * screen_width, screen_height / 8.0),
    //                 Vec2::new(0.0, 200.0), dt));

    //         ball_drop_accumulator = 0.0;
    //     }
    // }
    
    loop {
        let current_time = get_time();
        let frame_time = (current_time - last_time) as f32;
        last_time = current_time;
        
        accumulator += frame_time;
        ball_drop_accumulator += frame_time;
        mouse_drop_accumulator += frame_time;
        
        if is_mouse_button_down(MouseButton::Left) {
            if mouse_drop_accumulator >= mouse_drop_dt {
                let position = vec2(mouse_position().0, mouse_position().1) - vec2(screen_width / 2.0, screen_height / 2.0);
                solver.add_position(Verlet::new(position));  // Add new position at mouse position
                mouse_drop_accumulator = 0.0;
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
            // solver.add_position(Verlet::new_with_velocity(Vec2::new(screen_width / 2.0, screen_height / 2.0),
            //         500.0 * Vec2::new(angle.cos(), angle.sin()), dt));
            let mut ballz = Verlet::new_with_radius(Vec2::new(0.15 * screen_width,0.0), 20.0);
            ballz.set_velocity(Vec2::new(0.0, 200.0), dt);
            solver.add_position(ballz);
            ball_drop_accumulator = 0.0;
        }

        while accumulator >= dt {
            solver.update(dt);
            accumulator -= dt;
            
            // if solver.is_container_full() {
                // println!("{}", solver.is_container_full());
            //     solver.apply_rainbow_gradient();
            // }
        }
        
        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius, 1.0, WHITE);  // Draw constraint circle
        
        let alpha = accumulator / dt;
        for verlet in solver.get_verlets() {
            // This is since the solver imagines the ball at being shows at 0, 0
            let origin = vec2(screen_width / 2.0, screen_height / 2.0);
            let interpolated_pos = origin + verlet.get_interpolated_position(alpha);
            let (x, y) = interpolated_pos.into();
            draw_circle(x, y, verlet.get_radius(), Color::from_rgba(
                verlet.get_color().x as u8,
                verlet.get_color().y as u8,
                verlet.get_color().z as u8,
                255
            ));
            // draw_arrow(
            //     interpolated_pos,
            //     interpolated_pos + verlet.get_velocity() / 5.0,
            //     ORANGE
            // );
            // draw_arrow(
            //     interpolated_pos,
            //     interpolated_pos + verlet.get_acceleration() / 5.0,
            //     RED
            // );
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
