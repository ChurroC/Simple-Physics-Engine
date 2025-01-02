use macroquad::prelude::Vec2;
use super::verlet::Verlet;

pub struct Solver {
    verlets: Vec<Verlet>,
    gravity: Vec2,
    contraint_center: Vec2,
    contraint_radius: f32,
}

impl Solver {
    pub fn new(positions: &[Vec2], gravity: Vec2, contraint_center: Vec2, contraint_radius: f32) -> Self {
        let verlets = positions
            .iter()
            .map(|&pos| Verlet::new(pos))
            .collect();

        Solver {
            verlets,
            gravity,
            contraint_center,
            contraint_radius
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.apply_gravity();
        self.update_positions(dt);
    }

    pub fn update_positions(&mut self, dt: f32) {
        for verlet in &mut self.verlets {
            verlet.update_positions(dt);
        }
    }

    pub fn apply_gravity(&mut self) {
        for verlet in &mut self.verlets {
            verlet.accelerate(self.gravity);
        }
    }

    pub fn get_positions(&self) -> Vec<Vec2> {
        self.verlets.iter()
            .map(|verlet| verlet.get_position())
            .collect()
    }

    pub fn add_positions(&mut self, positions: &[Vec2]) {
        let new_verlets = positions
            .iter()
            .map(|&pos| Verlet::new(pos))
            .collect::<Vec<Verlet>>();
            
        self.verlets.extend(new_verlets);
    }

    pub fn apply_contraints(&mut self) {

    }
}