use macroquad::prelude::Vec2;
use super::verlet::Verlet;

pub struct Solver {
    verlets: Vec<Verlet>,
    gravity: Vec2,
    constraint_center: Vec2,
    constraint_radius: f32,
    substep: u32,
}

impl Solver {
    pub fn new(verlets: &[Verlet], gravity: Vec2, constraint_center: Vec2, constraint_radius: f32, substep: u32) -> Self {
        Solver {
            verlets: verlets.iter().cloned().collect(),
            gravity,
            constraint_center,
            constraint_radius,
            substep
        }
    }

    pub fn update(&mut self, dt: f32) {
        let substep_dt = dt / self.substep as f32;
        for _ in 0..self.substep {
            self.apply_gravity();
            self.apply_constraints(substep_dt);
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
            verlet.add_acceleration(self.gravity);
        }
    }

    /*
        
    * Apply    
     */
    fn apply_constraints(&mut self, dt: f32) {
        for verlet in &mut self.verlets {
            let normal = verlet.get_position() - self.constraint_center; // Or distance to verlet from center
            let dist = normal.length();
            
            if dist > self.constraint_radius - verlet.get_radius() {
                /*
                    v_1f = [v_1 (m_1 - m_2) + 2 m_2 v_2]/(m_1 + m_2)
                    as m2 goes to infinty since a wall with infinte mass
                    m_1 - m_2 = -m_2
                    m_1 + m_2 = m_2
                    v_2 = 0
                    v_1f = -v_1

                    so the change in velocity is -2v_1 since v_1f - v_1 = -v_1 - v_1 = -2v_1
                 */
                
                // Project v in the direction of the normal from wall
                // projection is
                // v_normal = [(v . n) / (n . n)] * n / n. n 
                // v_normal = [(v . n) / |n|] * n / |n|
                // v_normal = [|v| |n| cos(theta) / |n|] * n / |n|
                // v_normal = |v| cos(theta) * n / |n|
                // v_normal = v cos(theta) * n_unit
                // Makes sense cause we get v cos of angle whic his projected onto the normal vector
                // Then we multiple by normal vector to vector distance
                // So easy way to show in code is prob v_normal = v . n * |n|^2

                // Since we get value of v on on using dot product v . n then project the length onto the the unit vector of v
                // So we could just do 
                let vel = verlet.get_velocity();
                let v_normal = vel.project_onto(normal);
                
                
                // Calculate acceleration needed for velocity change
                // For a complete reversal of normal velocity: Δv = -2v_normal
                // a = Δv/Δt where Δt is our simulation timestep
                let delta_v = -2.0 * v_normal;
                // We need this change in velocity over the next frame
                let accel = delta_v / dt;
                
                // Position correction (must satisfy constraint exactly)
                let penetration = normal - (self.constraint_radius - verlet.get_radius());
                let position_correction = penetration.project_onto(normal);
                
                // Apply acceleration and position correction
                verlet.add_acceleration(accel + position_correction);
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
        self.verlets.push(Verlet::new(position));
    }
    pub fn add_positions(&mut self, positions: &[Vec2]) {
        let new_verlets = positions
            .iter()
            .map(|&pos| Verlet::new(pos))
            .collect::<Vec<Verlet>>();
            
        self.verlets.extend(new_verlets);
    }

    pub fn get_verlets(&self) -> &Vec<Verlet> {
        &self.verlets
    }

}