use macroquad::prelude::Vec2;

pub struct Verlet {
    position: Vec2,
    old_position: Vec2,
    acceleration: Vec2,
}

impl Verlet {
    pub fn new(position: Vec2) -> Self {
        Verlet {
            position,
            old_position: position,
            acceleration: Vec2::ZERO,
        }
    }

    pub fn get_position(&self) -> Vec2 {
        self.position  // Vec2 is Copy, so this creates a copy automatically
    }

    pub fn update_positions(&mut self, dt: f32){
        // V(n-1) * dt = P(n) - P(n-1)
        let velocity = self.position - self.old_position;
        // P(n-1) = P(n)
        self.old_position = self.position; 
        // Verlet integration - P(n+1) = 2P(n) - P(n-1) + a(n) * dt^2 = P(n) + V(n) + a(n) * dt^2
        self.position += velocity + self.acceleration * dt * dt;

        self.acceleration = Vec2::ZERO;
    }

    pub fn accelerate(&mut self, acceleration: Vec2){
        self.acceleration += acceleration;
    }
}