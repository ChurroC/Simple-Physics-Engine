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
                println!("");
                // First idea I have is to somehow reflect the velocity vector across the tangent line formed by the circular wall
                // What I though of is to find the projection of the velocity vector to the radius line (center_dist_vec) which is a the normal portion of the velocity vector
                // Then we could subtract the normal portion to get the tangent portion of the velocity vector then do it again to get the reflected vector
                // verlet.get_velocity(dt) - 2.0 * verlet.get_velocity(dt).dot(center_dist_vec) / center_dist_vec.dot(center_dist_vec) * center_dist_vec
                // Easiest way to understand in just an x and y axis is to imagien the velocity vector is (1, -1) and the center_dist_vec is the y where I add (0, 1) to the velocity twice to get (1, 1) which is reflected across the y axis here and not the tangent axis
                let reflect_vector = - 2.0 * verlet.get_velocity(dt).dot(center_dist_vec) / center_dist_vec.dot(center_dist_vec) * center_dist_vec;
                verlet.set_velocity( verlet.get_velocity(dt) + reflect_vector, dt);
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
}