use macroquad::prelude::Vec2;
use super::verlet::Verlet;

pub struct Solver {
    verlets: Vec<Verlet>,
    gravity: Vec2,
    constraint_center: Vec2,
    constraint_radius: f32,
}


impl Solver {
    pub fn new(verlets: &[Verlet], gravity: Vec2, constraint_center: Vec2, constraint_radius: f32) -> Self {
        Solver {
            verlets: verlets.iter().cloned().collect(),
            gravity,
            constraint_center,
            constraint_radius,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.apply_gravity();
        self.apply_constraints(dt);
        self.solve_collisions(dt);
        self.update_positions(dt);
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

    fn apply_constraints(&mut self, dt: f32) {
        for verlet in &mut self.verlets {
            let dist_to_vert = verlet.get_position() - self.constraint_center; // Or distance to verlet from center
            let dist = dist_to_vert.length();
            
            if dist > self.constraint_radius - verlet.get_radius() {
                let unit_normal = dist_to_vert.normalize();

                let vel = verlet.get_velocity();
                let v_normal = vel.project_onto(unit_normal);

                let correct_position = self.constraint_center + unit_normal * (self.constraint_radius - verlet.get_radius());
                verlet.set_position(correct_position);
                verlet.set_velocity( vel - 2.0 * v_normal, dt); // Just push the portion normal to the wall inverse
            }
        }
    }
    
    

    fn solve_collisions(&mut self, dt: f32) {
        let verlet_count = self.verlets.len();
        let coefficient_of_restitution = 0.75;

        for i in 0..verlet_count {
            for j in i + 1..verlet_count {
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
    }

    pub fn get_positions(&self) -> Vec<Vec2> {
        self.verlets.iter()
            .map(|verlet| verlet.get_position())
            .collect()
    }

    pub fn add_position(&mut self, verlet: Verlet) {
        self.verlets.push(verlet);
    }
    pub fn add_positions(&mut self, verlets: &[Verlet]) {
        self.verlets.extend(verlets.iter().cloned());
    }

    pub fn get_verlets(&self) -> &Vec<Verlet> {
        &self.verlets
    }

}