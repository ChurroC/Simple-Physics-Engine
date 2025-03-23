use super::verlet::Verlet;
use glam::{Vec2, Vec4};
use physics_engine::ThreadPool;
use serde::{Serialize, Deserialize};
use bincode;
use std::{sync::Arc, thread};
use rayon::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Solver {
    verlets: Vec<Verlet>,
    gravity: Vec2,
    constraint_radius: f32,
    events: Vec<(f32, bool, usize)>,  // Store persistent events list
    color_frames: Vec<Vec4>,
    current_frame: usize,
    subdivision: usize,
    cell_size: f32,
    grid_size: usize,
    grid: Vec<Vec<usize>>,
    #[serde(skip)]
    pool: ThreadPool,
    region_split: (usize, usize)
}


impl Solver {
    pub fn new(verlets: &[Verlet], gravity: Vec2, constraint_radius: f32, subdivision: usize, cell_size: f32, region_split: (usize, usize)) -> Self {
        let mut events = Vec::new();
        
        for (i, verlet) in verlets.iter().enumerate() {
            let pos = verlet.get_position().x;
            let radius = verlet.get_radius();
            events.push((pos - radius, false, i));
            events.push((pos + radius, true, i));
        }

        //  TimSort - O(n * log(n)) - Gonna use time sort initially since we don't know how in order these balls are
        events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let grid_size = (constraint_radius * 2.0 / cell_size) as usize;

        Solver {
            verlets: verlets.iter().cloned().collect(),
            gravity,
            constraint_radius,
            events,
            color_frames: Vec::new(),
            current_frame: 0,
            subdivision,
            cell_size: cell_size,
            grid_size: grid_size,
            grid: vec![vec![]; grid_size * grid_size],
            pool: ThreadPool::new(region_split.0 * region_split.1 + 2),
            region_split
        }
    }

    pub fn update(&mut self, dt: f32) {
        let sub_dt = dt / self.subdivision as f32;
        for _ in 0..self.subdivision {
            self.apply_gravity();
            self.apply_wall_constraints(sub_dt);
            let collisions: Vec<(usize, usize)> = self.find_collisions_space_partitioning_parallel();
            self.solve_collisions(collisions, sub_dt);
            self.update_positions(sub_dt);
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

    // Pezzas way but even more accurate
    // Since his way of moving position creates a velocity spike
    // We just lose the normal velocity and keep the tangential velocity
    fn apply_wall_constraints_smooth(&mut self, dt: f32) {
        let coefficient_of_restitution = 1.0;

        for verlet in &mut self.verlets {
            let dist_to_cen = verlet.get_position(); // Or distance to verlet from center
            let dist = dist_to_cen.length();
            
            if dist > self.constraint_radius - verlet.get_radius() {
                let dist_norm: Vec2 = dist_to_cen.normalize();

                let vel = verlet.get_velocity();
                let v_norm = vel.project_onto(dist_norm);

                let correct_position = dist_norm * (self.constraint_radius - verlet.get_radius());
                verlet.set_position(correct_position);
                verlet.set_velocity( (vel - v_norm) * coefficient_of_restitution, dt); // Just push the portion normal to the wall inverse
            }
        }
    }

    // More accurate bounce
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
    
    // O(n^2)
    // 371 balls - 6 rad - 8 subs - 16 ms - -100 grav - -.250 grav
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

    // O(n log(n))
    // 884 balls - 6 rad - 8 subs - 16 ms
    fn find_collisions_sort_sweep(&mut self) -> Vec<(usize, usize)> {
        let mut collisions: Vec<(usize, usize)> = Vec::new();
        let len = self.verlets.len();
    
        // Step 1: Update event positions without recreating the list
        for event in &mut self.events {
            let id = event.2;
            let verlet = &self.verlets[id];
            let pos = verlet.get_position().x;
            let radius = verlet.get_radius();
    
            if event.1 {
                event.0 = pos + radius; // Right boundary
            } else {
                event.0 = pos - radius; // Left boundary
            }
        }

        let start_index = self.events.len() / 2;
        for i in start_index..len {
            let id = i; // Compute the new index in `verlets`
            let pos = self.verlets[i].get_position().x;
            let radius = self.verlets[i].get_radius();

            self.events.push((pos - radius, false, id));
            self.events.push((pos + radius, true, id));
        }
    
        // Step 2: Use insertion sort since events are nearly sorted
        for i in 1..self.events.len() {
            for j in (1..i).rev() {
                if self.events[j].0 < self.events[j + 1].0 {break;}
                self.events.swap(j, j + 1);
            }
        }
    
        // Step 3: Sweep Line Collision Detection
        let mut active: Vec<usize> = Vec::new();
        let mut active_positions: Vec<i32> = vec![-1_i32; len];
    
        for &(_, is_end, id) in &self.events {
            if !is_end {
                for &active_id in &active {
                    collisions.push((active_id.min(id), active_id.max(id)));
                }
                active_positions[id] = active.len() as i32;
                active.push(id);
            } else {
                let pos = active_positions[id];
                if pos >= 0 {
                    active.swap_remove(pos as usize);
                    if (pos as usize) < active.len() {
                        active_positions[active[pos as usize]] = pos;
                    }
                    active_positions[id] = -1;
                }
            }
        }
    
        collisions
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

    fn find_collisions_space_partitioning_parallel(&mut self) -> Vec<(usize, usize)> {
        let mut new_grid = vec![Vec::new(); self.grid.len()];
    
        // Populate using iterators
        self.verlets.iter().enumerate()
            .for_each(|(i, verlet)| {
                let pos = verlet.get_position();
                
                let cell_x = ((pos.x + self.constraint_radius) / self.cell_size).floor() as usize;
                let cell_y = ((pos.y + self.constraint_radius) / self.cell_size).floor() as usize;
                
                let cell_index = (cell_y * self.grid_size) + cell_x;
                if cell_index < new_grid.len() {
                    new_grid[cell_index].push(i);
                }
            });
        
        // Wrap grid in Arc for thread-safe sharing without cloning the actual data
        let grid = Arc::new(new_grid);
        let grid_size = self.grid_size;
    
        // Define neighbor offsets for collision checks
        let neighbor_offsets: [(i32, i32); 4] = [
            (1, 0),    // right
            (1, 1),    // bottom-right
            (0, 1),    // bottom
            (-1, 1),   // bottom-left
        ];

        let mut handles = vec![];

        let x_regions = self.region_split.0;
        let y_regions= self.region_split.1;
    
        for y_region in 0..y_regions {
            let start_y = (y_region * grid_size) / y_regions;
            let end_y = ((y_region + 1) * grid_size) / y_regions;
    
            for x_region in 0..x_regions {
                // Clone the Arc (cheap), not the grid itself
                let grid_ref = Arc::clone(&grid);
                
                // Calculate this thread's region
                let start_x = (x_region * grid_size) / x_regions;
                let end_x = ((x_region + 1) * grid_size) / x_regions;
    
                // Process assigned region
                let handle = self.pool.execute(move || {
                    let mut collisions = vec![];
                    
                    for y in start_y..end_y {
                        for x in start_x..end_x {
                            let cell_index = y * grid_size + x;
                            
                            // Check collisions within this cell
                            let particles_in_cell = &grid_ref[cell_index];
                            let particles_in_cell_count = particles_in_cell.len();

                            if particles_in_cell_count > 0 {
                                for i in 0..particles_in_cell_count {
                                    let particle_i = particles_in_cell[i];
                                    
                                    // Check against other particles in the same cell
                                    for j in (i + 1)..particles_in_cell_count {
                                        let particle_j = particles_in_cell[j];
                                        collisions.push((particle_i.min(particle_j), particle_i.max(particle_j)));
                                    }
                
                
                                    // Check against particles in neighboring cells
                                    for &(dx, dy) in &neighbor_offsets {
                                        let nx = x as i32 + dx;
                                        let ny = y as i32 + dy;
                                        
                                        // Check if neighbor is in bounds
                                        if nx >= 0 && nx < grid_size as i32 && 
                                           ny >= 0 && ny < grid_size as i32 {
                                            let neighbor_index = (ny as usize * grid_size) + nx as usize;
                                            
                                            // Check against all particles in the neighboring cell
                                            for &particle_j in &grid_ref[neighbor_index] {
                                                collisions.push((particle_i.min(particle_j), particle_i.max(particle_j)));
                                            }
                                        }
                                    }
                                }
                            }
                            
                        }
                    }
                    
                    collisions
                });
                
                handles.push(handle);
            }
        }
        
        // Collect results from all threads
        let mut all_collisions = Vec::new();
        for handle in handles {
            let result = handle.recv().unwrap();
            all_collisions.extend(result);
        }
        
        all_collisions
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
    pub fn add_position(&mut self, mut verlet: Verlet) {
        if !self.color_frames.is_empty() && self.current_frame < self.color_frames.len() {
            verlet.set_color(self.color_frames[self.current_frame]);
            self.current_frame += 1;
        }
        self.verlets.push(verlet);
    }
    pub fn add_positions(&mut self, verlets: &mut [Verlet]) {
        for verlet in verlets.iter_mut() {
            if !self.color_frames.is_empty() && self.current_frame < self.color_frames.len() {
                verlet.set_color(self.color_frames[self.current_frame]);
                self.current_frame += 1;
            }
        }
        self.verlets.extend(verlets.iter().cloned());
    }

    pub fn get_verlets(&self) -> &Vec<Verlet> {
        &self.verlets
    }

    pub fn save_state(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = bincode::serialize(&self)?;
        std::fs::write(filename, encoded)?;
        Ok(())
    }

    pub fn load_state(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read(filename)?;
        let solver = bincode::deserialize(&data)?;
        Ok(solver)
    }
    
    
    pub fn apply_rainbow_gradient(&mut self) {
        // Sort verlets by y position (from bottom to top)
        let mut sorted_indices: Vec<usize> = (0..self.verlets.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            self.verlets[b].get_position().y
                .partial_cmp(&self.verlets[a].get_position().y)
                .unwrap()
        });

        // Define rainbow colors (from bottom to top)
        let colors = [
            Vec4::new(255.0, 0.0, 0.0, 1.0),    // Red
            Vec4::new(255.0, 127.0, 0.0, 1.0),  // Orange
            Vec4::new(255.0, 255.0, 0.0, 1.0),  // Yellow
            Vec4::new(0.0, 255.0, 0.0, 1.0),    // Green
            Vec4::new(0.0, 0.0, 255.0, 1.0),    // Blue
            Vec4::new(75.0, 0.0, 130.0, 1.0),   // Indigo
            Vec4::new(148.0, 0.0, 211.0, 1.0),  // Violet
        ];

        // Update colors based on position
        let total_verlets = sorted_indices.len() as f32;
        for (i, &idx) in sorted_indices.iter().enumerate() {
            let progress = i as f32 / total_verlets;
            let color_index = (progress * (colors.len() - 1) as f32) as usize;
            let next_color_index = (color_index + 1).min(colors.len() - 1);
            let color_progress = (progress * (colors.len() - 1) as f32) - color_index as f32;

            // Interpolate between colors
            let start_color = colors[color_index];
            let end_color = colors[next_color_index];
            let interpolated_color = Vec4::new(
                start_color.x + (end_color.x - start_color.x) * color_progress,
                start_color.y + (end_color.y - start_color.y) * color_progress,
                start_color.z + (end_color.z - start_color.z) * color_progress,
                1.0
            );

            self.verlets[idx].set_color(interpolated_color);
        }
    }


    pub fn color_from_image(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(file_path)?;
        let rgb_img = img.to_rgb8();
        let (img_width, img_height) = rgb_img.dimensions();
        
        // Create Gaussian kernel
        let kernel_size = 7; // Must be odd
        let sigma = 10.0;
        let kernel = self.create_gaussian_kernel(kernel_size, sigma);
        
        for verlet in &mut self.verlets {
            let pos: Vec2 = verlet.get_position();
            
            // Map position to image coordinates
            let x_ratio = 1.0 - ((pos.x / self.constraint_radius) + 1.0) * 0.5;
            let y_ratio = 1.0 - ((pos.y / self.constraint_radius) + 1.0) * 0.5;
            
            let img_x = (x_ratio * (img_width - 1) as f32) as i32;
            let img_y = (y_ratio * (img_height - 1) as f32) as i32;
            
            // Apply Gaussian blur at this position
            let mut r_sum = 0.0;
            let mut g_sum = 0.0;
            let mut b_sum = 0.0;
            let mut weight_sum = 0.0;
            
            let half_kernel = (kernel_size / 2) as i32;
            
            for y_offset in -half_kernel..half_kernel {
                for x_offset in -half_kernel..half_kernel {
                    let sample_x = img_x + x_offset;
                    let sample_y = img_y + y_offset;
                    
                    // Skip samples outside image bounds
                    if sample_x < 0 || sample_x >= img_width as i32 || 
                       sample_y < 0 || sample_y >= img_height as i32 {
                        continue;
                    }
                    
                    let kernel_x = (x_offset + half_kernel) as usize; // so for -3 to 3, it will be 0 to 6 indices
                    let kernel_y = (y_offset + half_kernel) as usize;
                    let weight = kernel[kernel_y][kernel_x];
                    
                    let pixel = rgb_img.get_pixel(sample_x as u32, sample_y as u32);
                    
                    r_sum += pixel[0] as f32 * weight;
                    g_sum += pixel[1] as f32 * weight;
                    b_sum += pixel[2] as f32 * weight;
                    weight_sum += weight;
                }
            }
            
            // Normalize by total weight
            let r = (r_sum / weight_sum).clamp(0.0, 255.0);
            let g = (g_sum / weight_sum).clamp(0.0, 255.0);
            let b = (b_sum / weight_sum).clamp(0.0, 255.0);
            
            // Set the blurred color
            let color = Vec4::new(
                r,
                g,
                b,
                255.0 // Full alpha
            );
            
            verlet.set_color(color);
        }
        
        Ok(())
    }
    
    fn create_gaussian_kernel(&self, size: usize, sigma: f32) -> Vec<Vec<f32>> {
        let mut kernel = vec![vec![0.0; size]; size];
        let center = (size as f32 - 1.0) / 2.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let exponent = -(dx * dx + dy * dy) / (2.0 * sigma * sigma);
                kernel[y][x] = 1.0 / (2.0 * std::f32::consts::PI * sigma * sigma) * exponent.exp();
            }
        }
        
        // Normalize kernel
        let sum: f32 = kernel.iter().flatten().sum();
        for row in kernel.iter_mut() {
            for value in row.iter_mut() {
                *value /= sum;
            }
        }
        
        kernel
    }

    pub fn load_colors(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = std::fs::read(filename)?;
        self.color_frames = bincode::deserialize(&data)?;
        self.current_frame = 0;
        for verlet in &mut self.verlets {
            if self.current_frame < self.color_frames.len() {
                verlet.set_color(self.color_frames[self.current_frame]);
                self.current_frame += 1;
            }
        }
        Ok(())
    }

    pub fn save_colors(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut colors: Vec<Vec4> = Vec::new();
        for verlet in &self.verlets {
            colors.push(verlet.get_color());
        }
        
        let encoded = bincode::serialize(&colors)?;
        std::fs::write(filename, encoded)?;
        Ok(())
    }
}