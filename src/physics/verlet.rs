use macroquad::prelude::Vec2;

pub struct Verlet {
    position: Vec2,
    old_position: Vec2,
    acceleration: Vec2,
    radius: f32,
}

impl Verlet {
    pub fn new(position: Vec2, radius: impl Into<Option<f32>>) -> Self {
        let radius = radius.into();
        Verlet {
            position,
            old_position: position,
            acceleration: Vec2::ZERO,
            radius: radius.unwrap_or(10.0),
        }
    }
    

    pub fn get_position(&self) -> Vec2 {
        self.position  // Vec2 is Copy, so this creates a copy automatically
    }

    pub fn get_velocity(&self, dt: f32) -> Vec2 {
        (self.position - self.old_position) / dt
    }

    pub fn set_velocity(&mut self, velocity: Vec2, dt: f32) {
        // get_velocity = [position - old_position] / dt= [position - (position - velocity * dt)] / dt = velocity * dt / dt = velocity
        self.old_position = self.position - (velocity * dt);
    }

    pub fn add_velocity(&mut self, velocity: Vec2, dt: f32) {
        // Minusing doesn't make sense until you connect to get_velocity
        // get_velocity = position - old_position = position - (position - velocity) = velocity
        self.old_position -= velocity * dt;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn update_position(&mut self, dt: f32){
        let displacement = self.position - self.old_position; // V(n-1) * dt = P(n) - P(n-1)
        self.old_position = self.position; // P(n-1) = P(n)
        self.position += displacement + self.acceleration * dt * dt; // Verlet integration - P(n+1) = 2P(n) - P(n-1) + a(n) * dt^2 = P(n) + V(n) + a(n) * dt^2
        
        self.acceleration = Vec2::ZERO; // Reset acceleration applied at this frame
    }

    pub fn accelerate(&mut self, acceleration: Vec2){
        self.acceleration += acceleration;
    }
}