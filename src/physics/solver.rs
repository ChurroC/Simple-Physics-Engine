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
            self.solve_collisions(substep_dt);
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
        let coefficient_of_restitution = 1.0;

        for i in 0..verlet_count {
            for j in i + 1..verlet_count {
                let (left, right) = self.verlets.split_at_mut(j);
                let verlet1 = &mut left[i];
                let verlet2 = &mut right[0];
                
                let collision_axis = verlet1.get_position() - verlet2.get_position(); // This is the distance vector between the two verlets which is also the collision_axis vector to the plane of collison
                let dist = collision_axis.length();
                let min_dist = verlet1.get_radius() + verlet2.get_radius();

                if dist < min_dist {
                    let vel1 = verlet1.get_velocity().project_onto(collision_axis);
                    let vel2 = verlet2.get_velocity().project_onto(collision_axis);
                    let m1 = verlet1.get_mass();
                    let m2 = verlet2.get_mass();

                    let vel1f = -(vel1 * (m1 - m2) + 2.0 * m2 *  vel2) / (m1 + m2);
                    let vel2f = (vel2 * (m2 - m1) + 2.0 * m1 *  vel1) / (m1 + m2);

                    verlet1.set_position(verlet1.get_position() + collision_axis.normalize() * (min_dist - dist));
                    verlet2.set_position(verlet2.get_position() -  collision_axis.normalize() * (min_dist - dist));


                    verlet1.set_velocity(vel1f * coefficient_of_restitution, dt);
                    verlet2.set_velocity(vel2f * coefficient_of_restitution, dt);
                }
            }
        }
    }

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