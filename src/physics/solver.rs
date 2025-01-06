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
                // First idea I have is to somehow reflect the velocity vector across the tangent line formed by the circular wall
                // What I though of is to find the projection of the velocity vector to the radius line (center_dist_vec) which is a the normal portion of the velocity vector
                // Then we could subtract the normal portion to get the tangent portion of the velocity vector then do it again to get the reflected vector
                // verlet.get_velocity(dt) - 2.0 * verlet.get_velocity(dt).dot(center_dist_vec) / center_dist_vec.dot(center_dist_vec) * center_dist_vec
                // Easiest way to understand in just an x and y axis is to imagien the velocity vector is (1, -1) and the center_dist_vec is the y where I add (0, 1) to the velocity twice to get (1, 1) which is reflected across the y axis here and not the tangent axis
                let reflect_vector = - 2.0 * velocity.dot(center_dist_vec) / center_dist_vec.dot(center_dist_vec) * center_dist_vec;
                verlet.add_velocity(reflect_vector, dt);
                // println!("{}", verlet.get_velocity(dt) + reflect_vector);

                // Or I could just rotate using matrix multiplication
                // At first I thought of using a roation of matrix to rotate it by 90 degress but it only works for some sections of the circle
                // I need to reflect it across the tangent line which is perpendicular to the radius line
                // let center_dist_unit_vec = center_dist_vec / center_dist;
                // let reflect_matrix = Mat2::from_cols(
                //     Vec2::new(1.0 - 2.0 * center_dist_unit_vec.x * center_dist_unit_vec.x, -2.0 * center_dist_unit_vec.x * center_dist_unit_vec.y),
                //     Vec2::new(-2.0 * center_dist_unit_vec.x * center_dist_unit_vec.y, 1.0 - 2.0 * center_dist_unit_vec.y * center_dist_unit_vec.y)
                // );
                // let reflect_vector = reflect_matrix * verlet.get_velocity(dt);
                // verlet.set_velocity(reflect_vector, dt);
                // println!("{}", reflect_vector);

                // Also turns out that there is an actual formula for this which is the reflection formula
                // V - 2(dot(V, N))N
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