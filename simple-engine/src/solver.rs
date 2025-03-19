use super::verlet::Verlet;
use glam::Vec2;

pub struct Solver {
    verlets: Vec<Verlet>,
    gravity: Vec2,
    constraint_radius: f32,
    subdivision: usize,
    cell_size: f32,
    grid_size: usize,
    grid: Vec<Vec<usize>>,
}


impl Solver {
    pub fn new(verlets: &[Verlet], gravity: Vec2, constraint_radius: f32, subdivision: usize, cell_size: f32) -> Self {
        let grid_size = (constraint_radius * 2.0 / cell_size) as usize; 
        Solver {
            verlets: verlets.iter().cloned().collect(),
            gravity,
            constraint_radius,
            subdivision,
            cell_size,
            grid_size,
            grid: vec![vec![]; grid_size * grid_size],
        }
    }

    pub fn update(&mut self, dt: f32) {
        let sub_dt = dt / self.subdivision as f32;
        for _ in 0..self.subdivision {
            for verlet in &mut self.verlets {
                verlet.add_acceleration(self.gravity);
            }

            self.apply_wall_constraints(sub_dt);

            let collisions: Vec<(usize, usize)> = self.find_collisions_space_partitioning();
            self.solve_collisions(collisions, sub_dt);

            for verlet in &mut self.verlets {
                verlet.update_position(sub_dt);
            }
        }
    }
    
    fn apply_wall_constraints(&mut self, dt: f32) {
        let coefficient_of_restitution = 1.0;

        for verlet in &mut self.verlets {
            let dist_to_cen = verlet.get_position();
            let dist = dist_to_cen.length();
            
            if dist > self.constraint_radius - verlet.get_radius() {
                let dist_norm = dist_to_cen.normalize();

                let vel = verlet.get_velocity();
                let v_norm = vel.project_onto(dist_norm);

                let correct_position = dist_norm * (self.constraint_radius - verlet.get_radius());
                verlet.set_position(correct_position);
                verlet.set_velocity( (vel - 2.0 * v_norm) * coefficient_of_restitution, dt); // Just push the portion normal to the wall inverse
            }
        }
    }

    // 1322 balls - 6 rad - 8 subs - 16 ms
    fn find_collisions_space_partitioning(&mut self) -> Vec<(usize, usize)> {
        let mut collisions: Vec<(usize, usize)> = vec![];

        for cell in &mut self.grid {
            cell.clear();
        }

        for (i, verlet) in self.verlets.iter().enumerate() {
            let pos = verlet.get_position();
            
            let cell_x = ((pos.x + self.constraint_radius) / self.cell_size).floor() as usize;
            let cell_y = ((pos.y + self.constraint_radius) / self.cell_size).floor() as usize;
            
            let cell_index = (cell_y * self.grid_size) + cell_x;
            if cell_index < self.grid.len() {
                self.grid[cell_index].push(i);
            }
        }

        let neighbor_offsets: [(i32, i32); 4] = [
            (1, 0),    // right
            (1, 1),    // bottom-right
            (0, 1),    // bottom
            (-1, 1),   // bottom-left
        ];

        // Grids are filled with indices of verlets
        for y in 0..self.grid_size {
            for x in 0..self.grid_size {
                let cell_index = y * self.grid_size + x;
                
                // Check collisions within this cell
                let particles_in_cell = &self.grid[cell_index];
                for i in 0..particles_in_cell.len() {
                    let particle_i = particles_in_cell[i];
                    
                    // Check against other particles in the same cell
                    for j in (i + 1)..particles_in_cell.len() {
                        let particle_j = particles_in_cell[j];
                        collisions.push((particle_i.min(particle_j), particle_i.max(particle_j)));
                    }


                    // Check against particles in neighboring cells
                    for &(dx, dy) in &neighbor_offsets {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        
                        // Check if neighbor is in bounds
                        if nx >= 0 && nx < self.grid_size as i32 && 
                           ny >= 0 && ny < self.grid_size as i32 {
                            let neighbor_index = (ny as usize * self.grid_size) + nx as usize;
                            
                            // Check against all particles in the neighboring cell
                            for &particle_j in &self.grid[neighbor_index] {
                                collisions.push((particle_i.min(particle_j), particle_i.max(particle_j)));
                            }
                        }
                    }
                }
            }
        }
        
        collisions
    }

    fn solve_collisions(&mut self, collisions: Vec<(usize, usize)>, dt: f32) {
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
                let overlap = (min_dist - dist) * 1.1;
                
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
        let total_particle_area: f32 = self.verlets
            .iter()
            .map(|v| std::f32::consts::PI * v.get_radius() * v.get_radius())
            .sum();
        
        // Calculate container area
        let container_area = std::f32::consts::PI * self.constraint_radius * self.constraint_radius;
        
        // Consider it full if particles take up more than X% of space
        // Note: Perfect circle packing is ~90.7% efficient
        let density = total_particle_area / container_area;
        density > 0.9 // or whatever threshold makes sense
    }

    pub fn get_positions(&self) -> Vec<Vec2> {
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