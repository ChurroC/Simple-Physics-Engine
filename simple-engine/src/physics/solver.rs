use super::verlet::Verlet;
use glam::DVec2;

pub struct Solver {
    verlets: Vec<Verlet>,
    gravity: DVec2,
    constraint_radius: f64,
}


impl Solver {
    pub fn new(verlets: &[Verlet], gravity: DVec2, constraint_radius: f64) -> Self {
        Solver {
            verlets: verlets.iter().cloned().collect(),
            gravity,
            constraint_radius,
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.apply_gravity();
        self.apply_wall_constraints(dt);
        let collisions = self.find_collisions_loop();
        self.solve_collisions(collisions, dt);
        self.update_positions(dt);
    }

    fn update_positions(&mut self, dt: f64) {
        for verlet in &mut self.verlets {
            verlet.update_position(dt);
        }
    }

    fn apply_gravity(&mut self) {
        for verlet in &mut self.verlets {
            verlet.add_acceleration(self.gravity);
        }
    }

    // More accurate bounce
    fn apply_wall_constraints(&mut self, dt: f64) {
        let coefficient_of_restitution = 1.0;
        let constraint_center= DVec2::new(0.0, 0.0);

        for verlet in &mut self.verlets {
            let dist_to_cen = verlet.get_position() - constraint_center; // Or distance to verlet from center
            let dist = dist_to_cen.length();
            
            if dist > self.constraint_radius - verlet.get_radius() {
                let dist_mag = dist_to_cen.normalize();

                let vel = verlet.get_velocity();
                let v_norm = vel.project_onto(dist_mag);

                let correct_position = constraint_center + dist_mag * (self.constraint_radius - verlet.get_radius());
                verlet.set_position(correct_position);
                verlet.set_velocity( (vel - 2.0 * v_norm) * coefficient_of_restitution, dt); // Just push the portion normal to the wall inverse
            }
        }
    }

    pub fn deterministic_normalize(&self, vec: DVec2) -> DVec2 {
        let length = (vec.x * vec.x + vec.y * vec.y).sqrt();
        if length > 1e-10 {  // Avoid division by very small numbers
            DVec2::new(vec.x / length, vec.y / length)
        } else {
            DVec2::ZERO
        }
    }
    
    // O(n^2)
    // 384 balls
    fn find_collisions_loop(&mut self) -> Vec<(usize, usize)> {
        let mut collisions: Vec<(usize, usize)> = Vec::new();

        let len  = self.verlets.len();
        for i in 0..len {
            for j in (i + 1)..len {
                let (verlet1, verlet2) = (&self.verlets[i], &self.verlets[j]);

                let collision_axis = verlet1.get_position() - verlet2.get_position();
                let dist = collision_axis.length();
                let min_dist = verlet1.get_radius() + verlet2.get_radius();

                if dist < min_dist {
                    collisions.push((i, j));
                }
            }
        }

        collisions
    }

    fn solve_collisions(&mut self, mut collisions: Vec<(usize, usize)>, dt: f64) {
        collisions.sort_by(|a, b| {
            if a.0 == b.0 {
                a.1.cmp(&b.1)
            } else {
                a.0.cmp(&b.0)
            }
        });
        
        // Force dt to have consistent precision
        let dt_str = format!("{:.15}", dt);
        let dt: f64 = dt_str.parse().unwrap();
        
        let coefficient_of_restitution = 0.93;

        for (i, j) in collisions {
            let (left, right) = self.verlets.split_at_mut(j);
            let verlet1 = &mut left[i];
            let verlet2 = &mut right[0];

            let collision_axis = verlet1.get_position() - verlet2.get_position(); // This is the distance vector between the two verlets which is also the collision_axis vector to the plane of collison
            let dist = collision_axis.length();
            let min_dist = verlet1.get_radius() + verlet2.get_radius();

            if dist < min_dist {
                let collision_normal = collision_axis.normalize();
                let collision_perp_normal = collision_axis.perp().normalize();
                let overlap = min_dist - dist;
                
                let vel1 = verlet1.get_velocity().project_onto(collision_normal);
                let vel1_perp = verlet1.get_velocity().project_onto(collision_perp_normal);
                let vel2 = verlet2.get_velocity().project_onto(collision_normal);
                let vel2_perp = verlet2.get_velocity().project_onto(collision_perp_normal);
                let m1 = verlet1.get_mass();
                let m2 = verlet2.get_mass();

                let vel1f = (vel1 * (m1 - m2) + 2.0 * m2 *  vel2) / (m1 + m2);
                let vel2f = (vel2 * (m2 - m1) + 2.0 * m1 *  vel1) / (m1 + m2);

                verlet1.set_position(verlet1.get_position() + collision_normal * overlap / 2.0);
                verlet2.set_position(verlet2.get_position() -  collision_normal * overlap / 2.0);

                // keeping the perp vel same and changing the collision vel
                verlet1.set_velocity((vel1_perp + vel1f) * coefficient_of_restitution, dt);
                verlet2.set_velocity((vel2_perp + vel2f) * coefficient_of_restitution, dt);
            }
        }
    }

    pub fn is_container_full(&self) -> bool {
        // Calculate total area of particles
        let total_particle_area: f64 = self.verlets
            .iter()
            .map(|v| std::f64::consts::PI * v.get_radius() * v.get_radius())
            .sum();
        
        // Calculate container area
        let container_area = std::f64::consts::PI * self.constraint_radius * self.constraint_radius;
        
        // Consider it full if particles take up more than X% of space
        // Note: Perfect circle packing is ~90.7% efficient
        let density = total_particle_area / container_area;
        density > 0.9 // or whatever threshold makes sense
    }

    pub fn get_positions(&self) -> Vec<DVec2> {
        self.verlets.iter()
            .map(|verlet| verlet.get_position())
            .collect()
    }
    pub fn add_position(&mut self, verlet: Verlet) {
        self.verlets.push(verlet);
    }
    pub fn add_positions(&mut self, verlets: &mut [Verlet]) {
        self.verlets.extend(verlets.iter().cloned());
    }

    pub fn get_verlets(&self) -> &Vec<Verlet> {
        &self.verlets
    }
}