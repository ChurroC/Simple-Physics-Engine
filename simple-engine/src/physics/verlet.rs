use glam::DVec2;


#[derive(Clone, Debug)]
pub struct Verlet {
    position: DVec2,
    last_position: DVec2,
    acceleration: DVec2,
    last_acceleration: DVec2,
    radius: f64,
    density: f64,
    last_dt: f64,
}

impl Verlet {
    pub fn new(position: DVec2) -> Self {
        Verlet {
            position,
            last_position: position,
            acceleration: DVec2::ZERO,
            last_acceleration: DVec2::ZERO,
            radius: 9.0,
            density: 1.0,
            last_dt: 0.0,
        }
    }
    pub fn new_with_radius(position: DVec2, radius: f64) -> Self {
        let radius = radius.into();
        Verlet {
            position,
            last_position: position,
            acceleration: DVec2::ZERO,
            last_acceleration: DVec2::ZERO,
            radius: radius,
            density: 1.0,
            last_dt: 0.0,
        }
    }
    pub fn new_with_velocity(position: DVec2, velocity: DVec2, dt: f64) -> Self {
        Verlet {
            position,
            last_position: position - velocity * dt,  // Set this directly
            acceleration: DVec2::ZERO,
            last_acceleration: DVec2::ZERO,
            radius: 9.0,
            density: 1.0,
            last_dt: dt, // Set this directly
        }
    }
    
    pub fn get_radius(&self) -> f64 {
        self.radius
    }

    pub fn get_mass(&self) -> f64 {
        self.density * std::f64::consts::PI * self.radius * self.radius
    }

    pub fn add_acceleration(&mut self, acceleration: DVec2){
        self.acceleration += acceleration;
    }

    pub fn get_position(&self) -> DVec2 {
        self.position  // DVec2 is Copy, so this creates a copy automatically
    }

    pub fn get_velocity(&self) -> DVec2 {
        if self.last_dt == 0.0 {
            DVec2::ZERO  // Return zero velocity for the first frame
        } else {
            (self.position - self.last_position) / self.last_dt
        }
    }

    pub fn set_velocity(&mut self, velocity: DVec2, dt: f64) {
        self.last_position = self.position - velocity * dt;
    }

    pub fn add_velocity(&mut self, velocity: DVec2, dt: f64) {
        self.last_position -= velocity * dt;
    }

    pub fn set_position(&mut self, position: DVec2) {
        self.position = position;
    }

    pub fn get_acceleration(&self) -> DVec2 {
        self.last_acceleration
    }
    
    pub fn get_interpolated_position(&self, alpha: f64) -> DVec2 {
        self.last_position + (self.position - self.last_position) * alpha
    }

    pub fn update_position(&mut self, dt: f64){
        let displacement = self.position - self.last_position;
        self.last_position = self.position;
        
        self.position += self.acceleration * dt * dt;
        self.position += displacement;

        self.last_acceleration = self.acceleration;
        self.last_dt = dt;
        self.acceleration = DVec2::ZERO; // Reset acceleration applied at this frame
    }
}