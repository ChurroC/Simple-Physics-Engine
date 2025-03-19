#![allow(dead_code)]

mod solver;
mod verlet;

use solver::Solver;
use verlet::Verlet;

use macroquad::prelude::{clear_background, draw_circle, draw_circle_lines, draw_text, get_fps, is_mouse_button_down, mouse_position, next_frame, screen_height, screen_width, Color, MouseButton, BLACK, RED, WHITE};
use glam::vec2;

use std::time::{SystemTime, UNIX_EPOCH};

#[macroquad::main("Game")]
async fn main() {
    let screen_width = screen_width();
    let screen_height = screen_height();

    let constraint_radius = screen_height.min(screen_width) / 2.0 - 50.0;

    let ball_size = 4.0;
    let mut solver = Solver::new(
        &[],
        vec2(0.0, -200.0),  // Gravity
        constraint_radius,
        8,
        ball_size * 2.5
    );

    let dt = 16;  // 1 / 60.0 = 16.6 ms
    let mut accumulator = 0;

    let mouse_drops_per_ms = 100;
    let mut mouse_drop_accumulator = 0;

    let ball_drop_per_frame = 10;
    let mut ball_drop_accumlator = 0;

    let mut last_time = get_time();
    let mut total_time: u128 = 0;

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

            if ball_drop_accumlator >= ball_drop_per_frame && !solver.is_container_full() {
                let mut ball = Verlet::new(vec2(0.15 * screen_width, screen_height * 2.0 / 7.0));
                ball.set_radius(ball_size);
                ball.set_velocity(vec2(0.0, -30.0), dt as f32 / 1000.0);
                solver.add_position(ball);
                ball_drop_accumlator = 0;
            }
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
        
        clear_background(BLACK);
        draw_circle_lines(screen_width / 2.0, screen_height / 2.0, constraint_radius, 1.0, WHITE);

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
            RED
        );

        next_frame().await;
    }
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