use macroquad::prelude::Vec2;
use super::verlet::Verlet;

pub struct Solver {
    verlets: Vec<Verlet>,
    gravity: Vec2,
    contraint_center: Vec2,
    contraint_radius: f32,
    substep: u32,
}

impl Solver {
    pub fn new(positions: &[Vec2], gravity: Vec2, contraint_center: Vec2, contraint_radius: f32, substep: u32) -> Self {
        let verlets = positions
            .iter()
            .map(|&pos| Verlet::new(pos, None))
            .collect();

        Solver {
            verlets,
            gravity,
            contraint_center,
            contraint_radius,
            substep
        }
    }

    pub fn update(&mut self, dt: f32) {
        let substep_dt = dt / self.substep as f32;
        for _ in 0..self.substep {
            self.apply_gravity();
            self.apply_contraints(substep_dt);
            // self.solve_collisions(substep_dt);
            self.update_positions(substep_dt);
        }
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

    fn apply_contraints(&mut self, dt: f32) {
        for verlet in &mut self.verlets {
            let center_dist_vec = self.contraint_center - verlet.get_position();
            let center_dist = center_dist_vec.length();

            if center_dist > self.contraint_radius - verlet.get_radius() {
                let velocity = verlet.get_velocity(dt);
                let reflect_vector = - 2.0 * velocity.dot(center_dist_vec) / center_dist_vec.dot(center_dist_vec) * center_dist_vec;
                verlet.add_velocity(reflect_vector, dt);
            }
        }
    }

    // fn solve_collisions(&mut self, dt: f32) {
    //     let verlet_count = self.verlets.len();
    //     let coefficient_of_restitution = 0.75;

    //     for i in 0..verlet_count {
    //         for j in i + 1..verlet_count {
    //             let (left, right) = self.verlets.split_at_mut(j);
    //             let verlet1 = &mut left[i];
    //             let verlet2 = &mut right[0];
                
    //             let normal = verlet1.get_position() - verlet2.get_position(); // This is the distance vector between the two verlets which is also the normal vector to the plane of collison

    //             let verlet1_proj = verlet1.
    //             }
    //         }
    //     }
    // }

    // fn solve_collisions(&mut self, dt: f32) {
    //     let verlet_count = self.verlets.len();
    //     let collision_coefficient = 0.75;
    //     for i in 0..verlet_count {
    //         for j in i + 1..verlet_count {
    //             let (left, right) = self.verlets.split_at_mut(j);
    //             let verlet1 = &mut left[i];
    //             let verlet2 = &mut right[0];
                
    //             let dist_vec = verlet2.get_position() - verlet1.get_position();
    //             let dist = dist_vec.length();
    //             let min_dist = verlet1.get_radius() + verlet2.get_radius();
    
    //             if dist < min_dist {
    //                 let dist_unit_vec = dist_vec / dist;
    //                 let overlap: f32 = min_dist - dist;
                    
    //                 let mass_ratio_1 = verlet1.get_radius() / (verlet1.get_radius() + verlet2.get_radius());
    //                 let mass_ratio_2 = verlet2.get_radius() / (verlet1.get_radius() + verlet2.get_radius());
    //                 let delta = 0.5 * collision_coefficient * overlap;
                
    //                 // Convert position changes to accelerations
    //                 verlet1.accelerate(-dist_unit_vec * (mass_ratio_2 * delta));
    //                 verlet2.accelerate(dist_unit_vec * (mass_ratio_1 * delta));
                        
    //                 // Velocity reflection
    //                 let v1 = verlet1.get_velocity(dt);
    //                 let v2 = verlet2.get_velocity(dt);
                    
    //                 // Calculate relative velocity
    //                 let rel_velocity = v2 - v1;
                    
    //                 // Calculate impulse
    //                 let normal_velocity = rel_velocity.dot(dist_unit_vec);
    //                 if normal_velocity < 0.0 {  // Only reflect if objects are moving toward each other
    //                     let impulse = -2.0 * normal_velocity * collision_coefficient;
                        
    //                     // Apply impulse in opposite directions
    //                     verlet1.add_velocity(-dist_unit_vec * (impulse * mass_ratio_2), dt);
    //                     verlet2.add_velocity(dist_unit_vec * (impulse * mass_ratio_1), dt);
    //                 }
    //             }
    //         }
    //     }
    // }

    pub fn get_positions(&self) -> Vec<Vec2> {
        self.verlets.iter()
            .map(|verlet| verlet.get_position())
            .collect()
    }

    pub fn add_position(&mut self, position: Vec2) {
        self.verlets.push(Verlet::new(position, None));
    }
    pub fn add_positions(&mut self, positions: &[Vec2]) {
        let new_verlets = positions
            .iter()
            .map(|&pos| Verlet::new(pos, None))
            .collect::<Vec<Verlet>>();
            
        self.verlets.extend(new_verlets);
    }

    pub fn get_verlets(&self) -> &Vec<Verlet> {
        &self.verlets
    }

}