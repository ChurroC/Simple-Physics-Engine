use macroquad::prelude::Vec2;

struct VerletObject {
    pub position: Vec2,
    pub old_position: Vec2,
    pub acceleration: Vec2,
}

impl VerletObject {
    // Constructor (like a "new" method in C++)
    fn new(position: Vec2) -> Self {
        VerletObject {
            position,
            old_position: position,  // Start with no velocity
            acceleration: Vec2::ZERO,
        }
    }

    fn update_positions(&mut self, dt: f32){
        let velocity = self.position - self.old_position;
        self.old_position = self.position;
        self.position += velocity + self.acceleration * dt * dt;
    }
}