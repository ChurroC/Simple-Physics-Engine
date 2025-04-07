mod solver;
mod verlet;

use solver::Solver;
use verlet::Verlet;

use macroquad::prelude::{clear_background, draw_circle, draw_circle_lines, draw_text, get_fps, is_key_pressed, is_mouse_button_down, mouse_position, next_frame, screen_height, screen_width, Color, KeyCode, MouseButton, BLACK, RED, WHITE, GREEN, draw_line};
use glam::vec2;

use std::time::Instant;


#[macroquad::main("Game")]
async fn main() {
    let screen_width = screen_width();
    let screen_height = screen_height();

    let constraint_radius = screen_height.min(screen_width) / 2.0 - 50.0;

    let ball_size = 10.0;
    let mut solver = Solver::new(
        &[
            Verlet::new(vec2(0.0, 0.0)),  // Center
            Verlet::new(vec2(100.0, 0.0)),  // Center
            Verlet::new(vec2(0.0, 100.0)),  // Center
            Verlet::new(vec2(100.0, 100.0)),  // Center
        ],
        vec2(0.0, -100.0),  // Gravity
        constraint_radius,
        8,
        ball_size * 2.5,
        10000.0,
    );
    if let Err(e) = solver.load_colors("colors.bin") {
        println!("Error loading colors: {}", e);
    }

    let start_time = Instant::now();

    let dt = 4;  // 1 / 60.0 = 16.6 ms
    let mut accumulator = 0;

    let mouse_drops_per_ms = 100;
    let mut mouse_drop_accumulator = 0;

    let ball_drop_per_frame = 10;
    let mut ball_drop_accumlator = 0;

    let mut last_time = start_time.elapsed().as_millis();
    let mut total_time: u128 = 0;

    let fps_threshold: i32 = 60;
    let measurement_frames: i32 = 30; // Number of frames to confirm slowdown
    let mut slow_frames_accumulator: i32 = 0;
    let mut balls_til_60_fps: usize = 0;

    let mut angle_degree = 0;

    // CUBE
    // if solver.create_distance_constraints(&[
    //     (0, 1, 100.0),
    //     (1, 3, 100.0),
    //     (3, 2, 100.0),
    //     (2, 0, 100.0),
    //     (0, 3, 100.0),
    //     (1, 2, 100.0),
    // ]).is_err() {
    //     println!("Error creating distance constraint");
    // }
    
    // Create a cloth grid
    let grid_width = 10;
    let grid_height = 10;
    let spacing = 20.0; // Distance between particles

    // Create the particles
    let mut cloth_particles = Vec::new();
    for y in 0..grid_height {
        for x in 0..grid_width {
            let x_pos = (x as f32 * spacing) - (grid_width as f32 * spacing / 2.0);
            let y_pos = (y as f32 * spacing);
            
            let mut particle = Verlet::new(vec2(x_pos, y_pos));
            particle.set_radius(ball_size / 2.0); // Smaller radius for cloth
            
            // Optional: anchor the top row
            
            cloth_particles.push(particle);
        }
    }

    // Add all particles to the solver
    let start_index = solver.get_verlets().len();
    solver.add_positions(&mut cloth_particles);

    // Create structural constraints (horizontal and vertical)
    let mut constraints = Vec::new();
    for y in 0..grid_height {
        for x in 0..grid_width {
            let idx = start_index + y * grid_width + x;
            
            // Horizontal connections
            if x < grid_width - 1 {
                let right_idx = idx + 1;
                constraints.push((idx, right_idx, spacing));
            }
            
            // Vertical connections
            if y < grid_height - 1 {
                let bottom_idx = idx + grid_width;
                constraints.push((idx, bottom_idx, spacing));
            }
            
            // Optional: Diagonal connections for more stability
            if x < grid_width - 1 && y < grid_height - 1 {
                let bottom_right_idx = idx + grid_width + 1;
                constraints.push((idx, bottom_right_idx, spacing * 1.414)); // sqrt(2) ≈ 1.414
            }
            
            if x > 0 && y < grid_height - 1 {
                let bottom_left_idx = idx + grid_width - 1;
                constraints.push((idx, bottom_left_idx, spacing * 1.414));
            }
        }
    }

    solver.create_distance_constraints(&constraints).unwrap();
    
    loop {
        let current_time = start_time.elapsed().as_millis();
        let frame_time = current_time - last_time; // Maybe add a cap to stop death dpiral
        let fps = 1.0 / (frame_time as f32 / 1000.0); // Maybe implement smoothing FPS
        last_time = current_time;
        
        accumulator += frame_time;
        mouse_drop_accumulator += frame_time;

        while accumulator >= dt {
            solver.update(dt as f32 / 1000.0);
            accumulator -= dt;
            total_time += dt;
            ball_drop_accumlator += 1;

            // if ball_drop_accumlator >= ball_drop_per_frame && !solver.is_container_full() {
            //     for _ in 0..1 {
            //         let angle = angle_degree as f32 / 180.0 * std::f32::consts::PI;
            //         let angle_vec = vec2(angle.cos(), angle.sin());
            //         let mut ball = Verlet::new(constraint_radius * 0.98 * angle_vec);
            //         ball.set_radius(ball_size);
            //         ball.set_velocity(-100.0 * angle_vec, dt as f32 / 1000.0);
            //         solver.add_position(ball);
            //         angle_degree = (angle_degree % 360) + 3;
            //     }

            //     ball_drop_accumlator = 0;
            // }
        }

        if is_mouse_button_down(MouseButton::Left) {
            if mouse_drop_accumulator >= mouse_drops_per_ms {
                let position = (vec2(mouse_position().0, mouse_position().1) - vec2(screen_width / 2.0, screen_height / 2.0)) * vec2(1.0, -1.0);
                let mut ball = Verlet::new(position);
                ball.set_radius(ball_size);

                solver.add_position(ball);
                mouse_drop_accumulator = 0;
            };
        }
        
        if is_key_pressed(KeyCode::S) {
            if let Err(e) = solver.save_colors("colors.bin") {
                println!("Error saving colors: {}", e);
            } else {
                println!("Colors saved successfully!");
            }
        }
        if is_key_pressed(KeyCode::L) {
            if let Err(e) = solver.color_from_image("churros.png") {
                println!("Error loading image: {}", e);
            }
        }
        
        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius, 1.0, WHITE);

        let alpha = accumulator as f32 / dt as f32;
        for verlet in solver.get_verlets() {
            // This is since the solver imagines the ball at being shows at 0, 0
            let origin = vec2(screen_width / 2.0, screen_height / 2.0);
            let interpolated_pos = origin + verlet.get_interpolated_position(alpha) * vec2(1.0, -1.0);
            let (x, y) = interpolated_pos.into();
            draw_circle(x, y, verlet.get_radius(), Color::from_rgba(
                verlet.get_color().x as u8,
                verlet.get_color().y as u8,
                verlet.get_color().z as u8,
                255
            ));
        }
        for &(verlet1, verlet2, distance) in solver.get_contraints() {
            let origin = vec2(screen_width / 2.0, screen_height / 2.0);

            let inter_pos1 = origin + solver.get_verlets()[verlet1].get_interpolated_position(alpha) * vec2(1.0, -1.0);
            let (x1, y1) = inter_pos1.into();
            let inter_pos2 = origin + solver.get_verlets()[verlet2].get_interpolated_position(alpha) * vec2(1.0, -1.0);
            let (x2, y2) = inter_pos2.into();

            draw_line(x1, y1, x2, y2, 1.0, if (inter_pos1 - inter_pos2).length() < distance { RED } else { GREEN });
        }

        if get_fps() < fps_threshold && balls_til_60_fps == 0 {
            slow_frames_accumulator += 1;
            if slow_frames_accumulator >= measurement_frames {
                balls_til_60_fps = solver.get_verlets().len();
            }
        } else if balls_til_60_fps == 0 {
            slow_frames_accumulator = 0;
        }

        [
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
        ].iter().enumerate().for_each(|(i, text)| {
            draw_text(text, 20.0, 30.0 + 30.0 * i as f32, 20.0, RED);
        });

        next_frame().await;
    }
}