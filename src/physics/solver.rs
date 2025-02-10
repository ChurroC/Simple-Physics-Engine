use super::verlet::Verlet;
use glam::{vec2, Vec2, Vec4};
use serde::{Serialize, Deserialize};
use bincode;
use image::GenericImageView;

#[derive(Serialize, Deserialize)]
pub struct Solver {
    verlets: Vec<Verlet>,
    gravity: Vec2,
    constraint_radius: f32,
}


impl Solver {
    pub fn new(verlets: &[Verlet], gravity: Vec2, constraint_radius: f32) -> Self {
        Solver {
            verlets: verlets.iter().cloned().collect(),
            gravity,
            constraint_radius,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.apply_gravity();
        self.apply_constraints(dt);
        let collisions = self.find_collisions_loop();
        self.solve_collisions(collisions, dt);
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

    // Pezzas way but even more accurate
    // Since his way of moving position creates a velocity spike
    // When just loses the normal velocity and keep the tangential velocity
    fn apply_constraints_smooth(&mut self, dt: f32) {
        let coefficient_of_restitution = 1.0;
        let constraint_center= vec2(0.0, 0.0);

        for verlet in &mut self.verlets {
            let dist_to_cen = verlet.get_position() - constraint_center; // Or distance to verlet from center
            let dist = dist_to_cen.length();
            
            if dist > self.constraint_radius - verlet.get_radius() {
                let dist_mag: Vec2 = dist_to_cen.normalize();

                let vel = verlet.get_velocity();
                let v_norm = vel.project_onto(dist_mag);

                let correct_position = constraint_center + dist_mag * (self.constraint_radius - verlet.get_radius());
                verlet.set_position(correct_position);
                verlet.set_velocity( (vel - v_norm) * coefficient_of_restitution, dt); // Just push the portion normal to the wall inverse
            }
        }
    }

    // More accurate bounce
    fn apply_constraints(&mut self, dt: f32) {
        let coefficient_of_restitution = 1.0;
        let constraint_center= vec2(0.0, 0.0);

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

        return collisions;
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
        let total_particle_area: f32 = self.verlets
            .iter()
            .map(|v| std::f32::consts::PI * v.get_radius() * v.get_radius())
            .sum();
        
        // Calculate container area
        let container_area = std::f32::consts::PI * self.constraint_radius * self.constraint_radius;
        
        // Consider it full if particles take up more than X% of space
        // Note: Perfect circle packing is ~90.7% efficient
        let density = total_particle_area / container_area;
        density > 0.87 // or whatever threshold makes sense
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

    pub fn picture_color(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>>{
        let img = image::open(file_path).expect("File not found!");
        let verlet = &self.verlets[0];

        // We need to map the image to verlet position??
        // First porportionally how much is the image on the left 
        let x_ratio = verlet.get_position().x / self.constraint_radius;
        let y_ratio = verlet.get_position().y / self.constraint_radius;

        for pixel in img.pixels() {
            print!("{:?}", pixel);
        }
        Ok(())
    }

    pub fn color_from_image(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(file_path)?;
        
        // Convert image to RGB format
        let rgb_img = img.to_rgb8();
        let (img_width, img_height) = rgb_img.dimensions();
        
        println!("Image loaded: {}x{}", img_width, img_height);
        
        // Since we're working with a circular constraint, map positions relative to the constraint
        let constraint_center = vec2(0.0, 0.0);
        
        for verlet in &mut self.verlets {
            let pos = verlet.get_position();
            
            // Map position relative to constraint
            // Convert from -radius to +radius range to 0 to 1 range
            let x_ratio = ((pos.x / self.constraint_radius) + 1.0) * 0.5;
            let y_ratio = ((pos.y / self.constraint_radius) + 1.0) * 0.5;
            
            // Convert to pixel coordinates
            let img_x = (x_ratio * (img_width - 1) as f32) as u32;
            let img_y = ((1.0 - y_ratio) * (img_height - 1) as f32) as u32; // Flip Y to match image coordinates
            
            // Get pixel color at mapped position
            let pixel = rgb_img.get_pixel(img_x, img_y);
            
            // Debug print some values
            println!(
                "Position: ({}, {}), Ratios: ({}, {}), Pixel: ({}, {}) Color: {:?}", 
                pos.x, pos.y, x_ratio, y_ratio, img_x, img_y, pixel
            );
            
            // Create Vec4 with correct scaling (RGB values are 0-255, we need 0-1)
            let color = Vec4::new(
                pixel[0] as f32 / 255.0,
                pixel[1] as f32 / 255.0,
                pixel[2] as f32 / 255.0,
                1.0
            );
            
            verlet.set_color(color * 255.0); // Multiply by 255 since the game might expect 0-255 range
        }
        
        Ok(())
    }
}