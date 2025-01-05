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
            .map(|&pos| Verlet::new(pos, None))
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
        self.apply_contraints(dt);
        self.update_positions(dt);
    }

    fn update_positions(&mut self, dt: f32) {
        for verlet in &mut self.verlets {
            verlet.update_position(dt);
        }
    }

    fn apply_gravity(&mut self) {
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
            .map(|&pos| Verlet::new(pos, None))
            .collect::<Vec<Verlet>>();
            
        self.verlets.extend(new_verlets);
    }

    fn apply_contraints(&mut self, dt: f32) {
        for verlet in &mut self.verlets {
            let center_dist_vec = self.contraint_center - verlet.get_position();
            let center_dist = center_dist_vec.length();

            if center_dist > self.contraint_radius - verlet.get_radius() {
                let center_dist_unit_vec = center_dist_vec / center_dist; // This is basically dividing the vector by its length giving us the unit vector
                
                // Using law of reflection when the ball hits the wall it should reflect back of the tangent line formed by the circular wall
                // Basically need to rotate the velocity vector by 90 degrees
                // I could either try projecting the velocity vector onto the tangent line and then subtracting it from the velocity vector

                // For this method we could find the vector that is perp to tangent line or the same direction as radius vector
                // We prob need to project the vector so thx Mr.Grattoni
                // proj between the rad and the velocity vector = (rad . velocity) / (rad . rad) * rad 

                // Or I could just rotate using matrix multiplication
                println!("");


                let test = verlet.get_velocity(dt) - 2.0 * verlet.get_velocity(dt).dot(center_dist_vec) / center_dist_vec.dot(center_dist_vec) * center_dist_vec;
                println!("{}", test);
                println!("{}", test.length());

                // verlet.set_velocity((center_dist_unit_vec * verlet.get_velocity(dt))/
                //                                 (center_dist_unit_vec * center_dist_unit_vec) * center_dist_unit_vec, dt);v
                let vel = verlet.get_velocity(dt);
                let perp = Vec2::new(-center_dist_unit_vec.y, center_dist_unit_vec.x);
                println!("2 {}", 2.0 * (vel.x * perp.x + vel.y * perp.y) * perp - vel);
                println!("2 {}", (2.0 * (vel.x * perp.x + vel.y * perp.y) * perp - vel).length());
                verlet.set_velocity(2.0 * (vel.x * perp.x + vel.y * perp.y) * perp - vel, dt);
            }
        }
    }
}