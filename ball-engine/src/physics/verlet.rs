use glam::{Vec2, Vec4};
use serde::{Serialize, Deserialize};
use rand::Rng;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Verlet {
    id: usize,
    position: Vec2,
    last_position: Vec2,
    acceleration: Vec2,
    last_acceleration: Vec2,
    radius: f32,
    density: f32,
    last_dt: f32,
    color: Vec4,
    is_sleeping: bool,
    sleep_timer: f32,
}

impl Verlet {
    pub fn new(position: Vec2) -> Self {
        let mut rng = rand::thread_rng();
        Verlet {
            id: 0,
            position,
            last_position: position,
            acceleration: Vec2::ZERO,
            last_acceleration: Vec2::ZERO,
            radius: 9.0,
            density: 1.0,
            last_dt: 0.0,
            color: Vec4::new(rng.gen_range(0.0..256.0), rng.gen_range(0.0..256.0), rng.gen_range(0.0..256.0), 1.0),
            is_sleeping: false,
            sleep_timer: 0.0,
        }
    }
    pub fn new_with_radius(position: Vec2, radius: f32) -> Self {
        let mut rng = rand::thread_rng();
        let radius = radius.into();
        Verlet {
            id: 0,
            position,
            last_position: position,
            acceleration: Vec2::ZERO,
            last_acceleration: Vec2::ZERO,
            radius: radius,
            density: 1.0,
            last_dt: 0.0,
            color: Vec4::new(rng.gen_range(0.0..256.0), rng.gen_range(0.0..256.0), rng.gen_range(0.0..256.0), 1.0),
            is_sleeping: false,
            sleep_timer: 0.0,
        }
    }
    pub fn new_with_velocity(position: Vec2, velocity: Vec2, dt: f32) -> Self {
        let mut rng = rand::thread_rng();
        Verlet {
            id: 0,
            position,
            last_position: position - velocity * dt,  // Set this directly
            acceleration: Vec2::ZERO,
            last_acceleration: Vec2::ZERO,
            radius: 9.0,
            density: 1.0,
            last_dt: dt, // Set this directly
            color: Vec4::new(rng.gen_range(0.0..256.0), rng.gen_range(0.0..256.0), rng.gen_range(0.0..256.0), 1.0),
            is_sleeping: false,
            sleep_timer: 0.0,
        }
    }
    
    pub fn get_color(&self) -> Vec4 {
        self.color
    }
    
    pub fn set_color(&mut self, color: Vec4) {
        self.color = color;
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
    
    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn get_mass(&self) -> f32 {
        self.density * std::f32::consts::PI * self.radius * self.radius
    }

    pub fn add_acceleration(&mut self, acceleration: Vec2){
        self.acceleration += acceleration;
    }

    pub fn get_position(&self) -> Vec2 {
        self.position  // Vec2 is Copy, so this creates a copy automatically
    }

    pub fn get_velocity(&self) -> Vec2 {
        if self.last_dt == 0.0 {
            Vec2::ZERO  // Return zero velocity for the first frame
        } else {
            (self.position - self.last_position) / self.last_dt
        }
    }

    pub fn set_velocity(&mut self, velocity: Vec2, dt: f32) {
        self.last_position = self.position - velocity * dt;
    }

    pub fn add_velocity(&mut self, velocity: Vec2, dt: f32) {
        self.last_position -= velocity * dt;
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn get_acceleration(&self) -> Vec2 {
        self.last_acceleration
    }
    
    pub fn get_interpolated_position(&self, alpha: f32) -> Vec2 {
        self.last_position + (self.position - self.last_position) * alpha
    }

    pub fn is_sleeping(&self) -> bool {
        self.is_sleeping
    }
    
    pub fn wake_up(&mut self) {
        self.is_sleeping = false;
        self.sleep_timer = 0.0;
    }
    
    pub fn try_sleep(&mut self, velocity_threshold: f32, time_threshold: f32, dt: f32) {
        if self.is_sleeping {
            return;
        }
        
        let velocity = self.get_velocity().length();
        
        if velocity < velocity_threshold {
            self.sleep_timer += dt;
            if self.sleep_timer >= time_threshold {
                self.is_sleeping = true;
            }
        } else {
            self.sleep_timer = 0.0;
        }
    }

    pub fn update_position(&mut self, dt: f32){
        if self.is_sleeping {
            self.last_acceleration = self.acceleration;
            self.last_dt = dt;
            self.acceleration = Vec2::ZERO; // Reset acceleration applied at this frame
            return;
        }

        let displacement = self.position - self.last_position;
        self.last_position = self.position;
        

        self.position += self.acceleration * dt * dt;
        self.position += displacement;

        self.last_acceleration = self.acceleration;
        self.last_dt = dt;
        self.acceleration = Vec2::ZERO; // Reset acceleration applied at this frame
    }
}