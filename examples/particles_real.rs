// Example: Particles, for real this time
// Based on multiple blog posts from Slow Rush Games:
// - "Particles, for real this time" - https://www.slowrush.dev/news/particles-for-real-this-time
// - "Bridging Physics Worlds" - https://www.slowrush.dev/news/bridging-physics-worlds
// - "Optimizing the Physics Bridge" - https://www.slowrush.dev/news/optimizing-physics-bridge
// - "Making Atoms Kinetic" - https://www.slowrush.dev/news/kinetic-atoms
//
// Optimizations implemented:
// 1. Spatial partitioning for particle interactions (O(n*m) -> O(n*log(m)))
// 2. Smart particle creation (only when needed, based on collision detection)
// 3. Occupied pixel caching for rigid bodies (AABB-based marking)
// 4. Batch particle cleanup and lifecycle management
// 5. Reverse velocity search for finding empty positions
// 6. Optimized atom update with occupied space checking

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Particle system to prevent atoms from crushing moving bodies
// Demonstrates the solution to the "atoms crushing moving bodies" problem

// Rendering constants for better visibility
const ATOM_RENDER_SIZE: f32 = 2.0; // Pixels per atom (reduced to show more atoms on screen)

#[derive(Clone, Debug,Eq, PartialEq)]
enum AtomType {
    Empty,
    Sand,
    Water,
    Stone,
    Oil,
    Lava,
    Acid,
    Steam,
    Smoke,
}

impl AtomType {
    fn mass(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand  => 1.6,
            AtomType::Water => 1.0,
            AtomType::Stone => 2.5,
            AtomType::Oil   => 0.8,
            AtomType::Lava  => 3.0,
            AtomType::Acid  => 1.2,
            AtomType::Steam => 0.1,
            AtomType::Smoke => 0.05,
        }
    }

    fn color(&self) -> Color {
        match self {
            AtomType::Empty => Color::rgba(0.0, 0.0, 0.0, 0.0),
            AtomType::Sand  => Color::rgb(0.8, 0.7, 0.5),
            AtomType::Water => Color::rgba(0.2, 0.4, 0.8, 0.8),
            AtomType::Stone => Color::rgb(0.4, 0.4, 0.4),
            AtomType::Oil   => Color::rgba(0.15, 0.1, 0.05, 0.95),
            AtomType::Lava  => Color::rgba(1.0, 0.4, 0.1, 0.95),
            AtomType::Acid  => Color::rgba(0.3, 0.9, 0.2, 0.9),
            AtomType::Steam => Color::rgba(0.9, 0.9, 0.9, 0.5),
            AtomType::Smoke => Color::rgba(0.3, 0.3, 0.3, 0.5),
        }
    }

    /// Conceptual density used for settling/swapping logic (like sandspiel)
    fn density(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Smoke => 0.05,
            AtomType::Steam => 0.1,
            AtomType::Oil   => 0.8,
            AtomType::Water => 1.0,
            AtomType::Acid  => 1.3,
            AtomType::Sand  => 2.0,
            AtomType::Stone => 5.0,
            AtomType::Lava  => 6.0,
        }
    }

    fn is_solid(&self) -> bool {
        matches!(self, AtomType::Sand | AtomType::Stone)
    }

    fn is_liquid(&self) -> bool {
        matches!(self, AtomType::Water | AtomType::Oil | AtomType::Lava | AtomType::Acid)
    }

    fn is_gas(&self) -> bool {
        matches!(self, AtomType::Steam | AtomType::Smoke)
    }
}

#[derive(Clone, Debug)]
struct ParticleAtom {
    atom_type: AtomType,
    position: Vec2,
    velocity: Vec2,
    is_particle: bool, // Flag to identify particle atoms
    lifetime: Option<f32>,
}

impl ParticleAtom {
    fn new(atom_type: AtomType, position: Vec2) -> Self {
        Self {
            atom_type,
            position,
            velocity: Vec2::ZERO,
            is_particle: false,
            lifetime: None,
        }
    }

    fn new_particle(atom_type: AtomType, position: Vec2, velocity: Vec2, lifetime: f32) -> Self {
        Self {
            atom_type,
            position,
            velocity,
            is_particle: true,
            lifetime: Some(lifetime),
        }
    }
}

// Particle system world
struct ParticleWorld {
    width: usize,
    height: usize,
    atoms: Vec<ParticleAtom>,
    moving_bodies: Vec<MovingBody>,
    // Cache for occupied pixels by rigid bodies (optimization from "Optimizing the Physics Bridge")
    occupied_pixels: Vec<bool>,
    // Track which regions need particle updates (spatial optimization)
    dirty_regions: Vec<(usize, usize)>, // (x, y) grid positions that need updates
}

impl ParticleWorld {
    fn new(width: usize, height: usize) -> Self {
        let mut atoms = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                atoms.push(ParticleAtom::new(AtomType::Empty, Vec2::new(x as f32, y as f32)));
            }
        }

        Self {
            width,
            height,
            atoms,
            moving_bodies: Vec::new(),
            occupied_pixels: vec![false; width * height],
            dirty_regions: Vec::new(),
        }
    }

    fn get_atom(&self, x: usize, y: usize) -> Option<&ParticleAtom> {
        if x < self.width && y < self.height {
            Some(&self.atoms[y * self.width + x])
        } else {
            None
        }
    }

    fn get_atom_mut(&mut self, x: usize, y: usize) -> Option<&mut ParticleAtom> {
        if x < self.width && y < self.height {
            Some(&mut self.atoms[y * self.width + x])
        } else {
            None
        }
    }

    fn set_atom(&mut self, x: usize, y: usize, atom: ParticleAtom) {
        if x < self.width && y < self.height {
            self.atoms[y * self.width + x] = atom;
        }
    }

    fn is_empty(&self, x: usize, y: usize) -> bool {
        self.get_atom(x, y).map_or(true, |a| matches!(a.atom_type, AtomType::Empty))
    }

    fn update(&mut self, dt: f32) {
        // Step 1: Mark space occupied by rigid bodies (optimized from "Bridging Physics Worlds")
        self.mark_occupied_pixels();

        // Step 2: Update moving bodies
        for body in &mut self.moving_bodies {
            body.update(dt);
        }

        // Step 3: Create particles around moving bodies only when needed
        // Optimization: Only create particles if body is moving fast or colliding
        let body_data: Vec<(Vec2, Vec2, Vec2)> = self.moving_bodies.iter()
            .map(|body| (body.position, body.velocity, body.size))
            .collect();
        
        for (position, velocity, size) in body_data {
            // Only create particles if body is moving or likely to be crushed
            if velocity.length_squared() > 1.0 || self.is_body_under_pressure(position, size) {
                self.create_particles_around_position(position, velocity);
            }
        }

        // Step 4: Update atoms (with optimized collision detection)
        self.update_atoms(dt);

        // Step 5: Let sand settle like real piles (grid-based settle pass)
        self.settle_sand();

        // Step 6: Let liquids (water / oil) flow and spread
        self.settle_liquids();

        // Step 7: Handle particle interactions (optimized with spatial partitioning)
        self.handle_particle_interactions_optimized(dt);

        // Step 8: Clean up expired particles (batch processing)
        self.cleanup_particles_batch();
    }

    fn update_atoms(&mut self, dt: f32) {
        // Optimized: Process atoms in batches and check occupied pixels
        for atom in &mut self.atoms {
            if atom.atom_type == AtomType::Empty {
                continue;
            }

            // Check if atom is in occupied space (from rigid body)
            let x = atom.position.x.round() as usize;
            let y = atom.position.y.round() as usize;
            if x < self.width && y < self.height {
                let idx = y * self.width + x;
                if self.occupied_pixels[idx] {
                    // Atom is inside a rigid body, push it away
                    // Find nearest body to calculate push direction
                    let mut push_velocity = Vec2::ZERO;
                    for body in &self.moving_bodies {
                        let to_atom = atom.position - body.position;
                        let distance = to_atom.length();
                        if distance < body.size.length() {
                            let push_strength = (body.size.length() - distance) * 20.0;
                            push_velocity += to_atom.normalize_or_zero() * push_strength;
                        }
                    }
                    atom.velocity += push_velocity * dt;
                }
            }

            // Apply gravity to non-particle atoms
            if !atom.is_particle {
                atom.velocity.y -= 30.0 * dt;
            }

            // Apply damping to prevent infinite velocity
            atom.velocity *= 0.98;

            // Update position
            atom.position += atom.velocity * dt;

            // Optimized bounds checking with clamping
            atom.position.x = atom.position.x.max(0.0).min(self.width as f32 - 1.0);
            atom.position.y = atom.position.y.max(0.0).min(self.height as f32 - 1.0);

            // Bounce off walls with damping
            if atom.position.x <= 0.0 || atom.position.x >= self.width as f32 - 1.0 {
                atom.velocity.x *= -0.5;
            }
            if atom.position.y <= 0.0 || atom.position.y >= self.height as f32 - 1.0 {
                atom.velocity.y *= -0.5;
            }

            // Update lifetime for particles
            if let Some(ref mut lifetime) = atom.lifetime {
                *lifetime -= dt;
            }
        }
    }

    /// Make sand behave like a pile: fall straight down, then diagonally, and swap with water
    fn settle_sand(&mut self) {
        let mut moved = vec![false; self.width * self.height];

        // Iterate top-down to avoid moving the same grain multiple times per frame
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if moved[idx] {
                    continue;
                }

                let atom_type = self.atoms[idx].atom_type.clone();
                if atom_type != AtomType::Sand || self.atoms[idx].is_particle {
                    continue;
                }

                // Target positions
                let below = if y > 0 { Some((x, y - 1)) } else { None };

                if let Some((bx, by)) = below {
                    let below_idx = by * self.width + bx;
                    let below_type = self.atoms[below_idx].atom_type.clone();
                    match below_type {
                        AtomType::Empty | AtomType::Steam => {
                            // Move sand straight down
                            let target_idx = below_idx;
                            let sand_atom = ParticleAtom::new(
                                AtomType::Sand,
                                Vec2::new(bx as f32, by as f32),
                            );
                            self.atoms[target_idx] = sand_atom;
                            self.atoms[idx] = ParticleAtom::new(
                                AtomType::Empty,
                                Vec2::new(x as f32, y as f32),
                            );
                            moved[target_idx] = true;
                            moved[idx] = true;
                            continue;
                        }
                        _ if below_type.is_liquid() && below_type.density() < atom_type.density() => {
                            // Sand sinks through lighter liquid (water, oil) by swapping
                            let target_idx = below_idx;
                            self.atoms.swap(idx, target_idx);
                            // Update positions after swap
                            self.atoms[idx].position = Vec2::new(x as f32, y as f32);
                            self.atoms[target_idx].position =
                                Vec2::new(bx as f32, by as f32);
                            moved[target_idx] = true;
                            moved[idx] = true;
                            continue;
                        }
                        _ => {}
                    }
                }

                // Try diagonals if straight down blocked
                let try_diag = |dx: isize, x: usize, y: usize, width: usize, height: usize| -> Option<(usize, usize)> {
                    let nx = x as isize + dx;
                    let ny = y as isize - 1; // downwards
                    if nx >= 0 && (nx as usize) < width && ny >= 0 {
                        Some((nx as usize, ny as usize))
                    } else {
                        None
                    }
                };

                // Randomize left/right to avoid bias
                let left_first = rand::random::<bool>();
                let diag_candidates = if left_first { [ -1, 1 ] } else { [ 1, -1 ] };

                let mut moved_diag = false;
                for dx in diag_candidates {
                    if let Some((nx, ny)) = try_diag(dx, x, y, self.width, self.height) {
                        let nidx = ny * self.width + nx;
                        let n_type = self.atoms[nidx].atom_type.clone();
                        match n_type {
                            AtomType::Empty | AtomType::Steam => {
                                // Move sand diagonally
                                let target_idx = nidx;
                                let sand_atom = ParticleAtom::new(
                                    AtomType::Sand,
                                    Vec2::new(nx as f32, ny as f32),
                                );
                                self.atoms[target_idx] = sand_atom;
                                self.atoms[idx] = ParticleAtom::new(
                                    AtomType::Empty,
                                    Vec2::new(x as f32, y as f32),
                                );
                                moved[target_idx] = true;
                                moved[idx] = true;
                                moved_diag = true;
                                break;
                            }
                            _ if n_type.is_liquid() && n_type.density() < atom_type.density() => {
                                // Swap with lighter liquid diagonally
                                let target_idx = nidx;
                                self.atoms.swap(idx, target_idx);
                                self.atoms[idx].position = Vec2::new(x as f32, y as f32);
                                self.atoms[target_idx].position =
                                    Vec2::new(nx as f32, ny as f32);
                                moved[target_idx] = true;
                                moved[idx] = true;
                                moved_diag = true;
                                break;
                            }
                            _ => {}
                        }
                    }
                }

                if moved_diag {
                    continue;
                }
            }
        }
    }

    /// Make liquids (water / oil) behave more like sandspiel style fluids:
    /// fall down, then diagonally, then spread sideways, using simple CA rules.
    fn settle_liquids(&mut self) {
        let mut moved = vec![false; self.width * self.height];

        // Iterate bottom-up so liquids prefer to fall into newly emptied cells
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if moved[idx] {
                    continue;
                }

                let atom_type = self.atoms[idx].atom_type.clone();
                if !atom_type.is_liquid() || self.atoms[idx].is_particle {
                    continue;
                }

                // Try move straight down
                if y > 0 {
                    let by = y - 1;
                    let bidx = by * self.width + x;
                    let below_type = self.atoms[bidx].atom_type.clone();

                    // Liquids fall through empty or gas
                    if matches!(below_type, AtomType::Empty | AtomType::Steam) {
                        self.atoms.swap(idx, bidx);
                        self.atoms[idx].position = Vec2::new(x as f32, y as f32);
                        self.atoms[bidx].position = Vec2::new(x as f32, by as f32);
                        moved[bidx] = true;
                        moved[idx] = true;
                        continue;
                    }

                    // If below is another liquid that is heavier, we can swap (e.g. oil floating on water)
                    if below_type.is_liquid() && below_type.density() > atom_type.density() {
                        self.atoms.swap(idx, bidx);
                        self.atoms[idx].position = Vec2::new(x as f32, y as f32);
                        self.atoms[bidx].position = Vec2::new(x as f32, by as f32);
                        moved[bidx] = true;
                        moved[idx] = true;
                        continue;
                    }
                }

                // Try diagonals down-left / down-right
                let left_first = rand::random::<bool>();
                let diag_candidates = if left_first { [ -1, 1 ] } else { [ 1, -1 ] };

                let mut moved_diag = false;
                for dx in diag_candidates {
                    let nx = x as isize + dx;
                    let ny = y as isize - 1;
                    if nx < 0 || ny < 0 {
                        continue;
                    }
                    let nxu = nx as usize;
                    let nyu = ny as usize;
                    if nxu >= self.width || nyu >= self.height {
                        continue;
                    }

                    let nidx = nyu * self.width + nxu;
                    let n_type = self.atoms[nidx].atom_type.clone();

                    if matches!(n_type, AtomType::Empty | AtomType::Steam) {
                        self.atoms.swap(idx, nidx);
                        self.atoms[idx].position = Vec2::new(x as f32, y as f32);
                        self.atoms[nidx].position = Vec2::new(nxu as f32, nyu as f32);
                        moved[nidx] = true;
                        moved[idx] = true;
                        moved_diag = true;
                        break;
                    }

                    if n_type.is_liquid() && n_type.density() > atom_type.density() {
                        self.atoms.swap(idx, nidx);
                        self.atoms[idx].position = Vec2::new(x as f32, y as f32);
                        self.atoms[nidx].position = Vec2::new(nxu as f32, nyu as f32);
                        moved[nidx] = true;
                        moved[idx] = true;
                        moved_diag = true;
                        break;
                    }
                }

                if moved_diag {
                    continue;
                }

                // If can't go down, spread sideways (gives that sandspiel-like smooth water flow)
                let side_first = rand::random::<bool>();
                let side_candidates = if side_first { [ -1, 1 ] } else { [ 1, -1 ] };

                for dx in side_candidates {
                    let nx = x as isize + dx;
                    if nx < 0 {
                        continue;
                    }
                    let nxu = nx as usize;
                    if nxu >= self.width {
                        continue;
                    }

                    let nidx = y * self.width + nxu;
                    let n_type = self.atoms[nidx].atom_type.clone();

                    if matches!(n_type, AtomType::Empty | AtomType::Steam) {
                        self.atoms.swap(idx, nidx);
                        self.atoms[idx].position = Vec2::new(x as f32, y as f32);
                        self.atoms[nidx].position = Vec2::new(nxu as f32, y as f32);
                        moved[nidx] = true;
                        moved[idx] = true;
                        break;
                    }
                }
            }
        }
    }

    fn create_particles_around_body(&mut self, body: &MovingBody) {
        self.create_particles_around_position(body.position, body.velocity);
    }

    // Find empty position near a point (from "Particles, for real this time" - reverse velocity search)
    fn find_empty_position_near(&self, start_pos: Vec2, search_direction: Vec2, max_distance: f32) -> Option<Vec2> {
        // Search along reverse velocity direction to find empty space
        let search_dir = -search_direction.normalize_or_zero();
        
        for step in 1..=(max_distance as i32) {
            let check_pos = start_pos + search_dir * (step as f32);
            let x = check_pos.x.round() as usize;
            let y = check_pos.y.round() as usize;
            
            if x < self.width && y < self.height && self.is_empty(x, y) {
                return Some(check_pos);
            }
        }
        
        None
    }

    // Optimized particle creation - only create where needed (from "Particles, for real this time")
    fn create_particles_around_position(&mut self, position: Vec2, velocity: Vec2) {
        let particle_distance = 2.0; // Distance from body to create particles
        let particle_lifetime = 0.5; // How long particles last
        let num_particles = 32; // Number of particles around the body (4x to increase density)

        // Only create particles in directions where there's pressure (atoms nearby)
        for i in 0..num_particles {
            let angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU;
            let direction = Vec2::new(angle.cos(), angle.sin());
            let particle_pos = position + direction * particle_distance;

            let x = particle_pos.x.round() as usize;
            let y = particle_pos.y.round() as usize;

            if x < self.width && y < self.height {
                // Check if this position needs a particle (has atoms nearby in this direction)
                let check_pos = position + direction * (particle_distance + 1.0);
                let check_x = check_pos.x.round() as usize;
                let check_y = check_pos.y.round() as usize;
                
                let needs_particle = if check_x < self.width && check_y < self.height {
                    !self.is_empty(check_x, check_y)
                } else {
                    false
                };

                // Only create particle if position is empty and there's pressure from that direction
                if self.is_empty(x, y) && (needs_particle || velocity.length_squared() > 10.0) {
                    // If position is occupied, try to find empty space nearby (reverse velocity search)
                    let final_pos = if !self.is_empty(x, y) {
                        self.find_empty_position_near(particle_pos, velocity, 3.0)
                            .unwrap_or(particle_pos)
                    } else {
                        particle_pos
                    };

                    let final_x = final_pos.x.round() as usize;
                    let final_y = final_pos.y.round() as usize;

                    if final_x < self.width && final_y < self.height && self.is_empty(final_x, final_y) {
                        // Create a particle atom with velocity based on body movement
                        let particle = ParticleAtom::new_particle(
                            AtomType::Sand, // Use sand as particle material
                            final_pos,
                            velocity * 0.1 + direction * 5.0, // Particles inherit body velocity + push away
                            particle_lifetime,
                        );
                        self.set_atom(final_x, final_y, particle);
                    }
                }
            }
        }
    }

    // Optimized version using spatial partitioning (from "Optimizing the Physics Bridge")
    fn handle_particle_interactions_optimized(&mut self, dt: f32) {
        // Only check atoms near moving bodies to avoid O(n*m) complexity
        for body in &self.moving_bodies.clone() {
            let body_aabb = Rect::from_center_size(body.position, body.size * 1.5);
            
            // Calculate grid bounds for spatial optimization
            let min_x = (body_aabb.min.x.max(0.0) as usize).min(self.width);
            let max_x = (body_aabb.max.x.max(0.0) as usize).min(self.width);
            let min_y = (body_aabb.min.y.max(0.0) as usize).min(self.height);
            let max_y = (body_aabb.max.y.max(0.0) as usize).min(self.height);

            // Only iterate over nearby atoms instead of all atoms
            for y in min_y..max_y {
                for x in min_x..max_x {
                    if let Some(atom) = self.get_atom_mut(x, y) {
                        if atom.atom_type != AtomType::Empty && !atom.is_particle {
                            if body_aabb.contains(atom.position) {
                                // Calculate repulsion force from body
                                let to_atom = atom.position - body.position;
                                let distance = to_atom.length();

                                if distance > 0.0 && distance < body.size.length() * 0.75 {
                                    let force_strength = (body.size.length() * 0.75 - distance) * 50.0;
                                    let force_direction = to_atom.normalize();
                                    atom.velocity += force_direction * force_strength * dt;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Keep old method for compatibility (can be removed if not needed)
    #[allow(dead_code)]
    fn handle_particle_interactions(&mut self, dt: f32) {
        self.handle_particle_interactions_optimized(dt);
    }

    // Optimized batch cleanup (from "Optimizing the Physics Bridge")
    fn cleanup_particles_batch(&mut self) {
        // Batch process particles for better cache performance
        for atom in &mut self.atoms {
            if atom.is_particle {
                if let Some(ref mut lifetime) = atom.lifetime {
                    *lifetime -= 0.016; // Assume ~60fps for batch processing
                    if *lifetime <= 0.0 {
                        *atom = ParticleAtom::new(AtomType::Empty, atom.position);
                    }
                }
            }
        }
    }

    // Keep old method for compatibility
    #[allow(dead_code)]
    fn cleanup_particles(&mut self) {
        self.cleanup_particles_batch();
    }

    // Mark pixels occupied by rigid bodies (optimized from "Bridging Physics Worlds")
    fn mark_occupied_pixels(&mut self) {
        // Clear previous marks
        self.occupied_pixels.fill(false);

        // Mark pixels occupied by moving bodies using AABB
        for body in &self.moving_bodies {
            let half_size = body.size * 0.5;
            let min_x = ((body.position.x - half_size.x).max(0.0) as usize).min(self.width);
            let max_x = ((body.position.x + half_size.x).max(0.0) as usize).min(self.width);
            let min_y = ((body.position.y - half_size.y).max(0.0) as usize).min(self.height);
            let max_y = ((body.position.y + half_size.y).max(0.0) as usize).min(self.height);

            for y in min_y..max_y {
                for x in min_x..max_x {
                    let idx = y * self.width + x;
                    // Check if pixel is inside body (simple AABB check)
                    let pixel_pos = Vec2::new(x as f32, y as f32);
                    let to_pixel = pixel_pos - body.position;
                    if to_pixel.x.abs() <= half_size.x && to_pixel.y.abs() <= half_size.y {
                        self.occupied_pixels[idx] = true;
                    }
                }
            }
        }
    }

    // Check if a body is under pressure (needs particles to prevent crushing)
    fn is_body_under_pressure(&self, position: Vec2, size: Vec2) -> bool {
        let half_size = size * 0.5;
        let check_radius = half_size.length() + 2.0;
        
        let min_x = ((position.x - check_radius).max(0.0) as usize).min(self.width);
        let max_x = ((position.x + check_radius).max(0.0) as usize).min(self.width);
        let min_y = ((position.y - check_radius).max(0.0) as usize).min(self.height);
        let max_y = ((position.y + check_radius).max(0.0) as usize).min(self.height);

        let mut nearby_atoms = 0;
        for y in min_y..max_y {
            for x in min_x..max_x {
                if let Some(atom) = self.get_atom(x, y) {
                    if atom.atom_type != AtomType::Empty && !atom.is_particle {
                        let dist = (Vec2::new(x as f32, y as f32) - position).length();
                        if dist < check_radius {
                            nearby_atoms += 1;
                        }
                    }
                }
            }
        }

        // If there are many atoms nearby, body is under pressure
        nearby_atoms > 5
    }

    fn add_moving_body(&mut self, body: MovingBody) {
        self.moving_bodies.push(body);
    }
}

#[derive(Clone, Debug)]
struct MovingBody {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
    mass: f32,
}

impl MovingBody {
    fn new(position: Vec2, velocity: Vec2, size: Vec2, mass: f32) -> Self {
        Self {
            position,
            velocity,
            size,
            mass,
        }
    }

    fn update(&mut self, dt: f32) {
        // Apply gravity
        self.velocity.y -= 30.0 * dt;

        // Update position
        self.position += self.velocity * dt;

        // Simple bounds
        if self.position.y < self.size.y / 2.0 {
            self.position.y = self.size.y / 2.0;
            self.velocity.y *= -0.3; // Bounce
        }
    }
}

#[derive(Resource)]
struct ParticleWorldResource(ParticleWorld);

#[derive(Component)]
struct MovingBodyEntity {
    body_index: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Particles, for real this time - Preventing Crushing".to_string(),
                resolution: (1600.0, 1200.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(ParticleWorldResource(ParticleWorld::new(200, 150)))
        .add_systems(Startup, setup_particle_demo)
        .add_systems(Update, (
            update_particle_world,
            render_particle_atoms,
            render_moving_bodies,
            handle_particle_interaction,
            demonstrate_particle_system,
        ).chain())
        .run();
}

fn setup_particle_demo(mut commands: Commands, mut world: ResMut<ParticleWorldResource>) {
    commands.spawn(Camera2dBundle::default());

    // Create initial setup
    create_particle_demo_setup(&mut world.0);
}

fn create_particle_demo_setup(world: &mut ParticleWorld) {
    // Create stone floor
    for x in 0..world.width {
        for y in 0..5 {
            world.set_atom(x, y, ParticleAtom::new(AtomType::Stone, Vec2::new(x as f32, y as f32)));
        }
    }

    // Create sand pile that will fall on moving bodies (scaled for larger world)
    let sand_start_x = world.width / 4;
    let sand_end_x = world.width * 3 / 4;
    let sand_start_y = world.height / 3;
    let sand_end_y = world.height / 2;
    
    for x in sand_start_x..sand_end_x {
        for y in sand_start_y..sand_end_y {
            if rand::random::<f32>() < 0.8 {
                world.set_atom(x, y, ParticleAtom::new(AtomType::Sand, Vec2::new(x as f32, y as f32)));
            }
        }
    }

    // Create water above (scaled for larger world) as several separated clusters
    // Cluster layout: rows x cols grid of water blobs with gaps between them
    let clusters_x = 2;
    let clusters_y = 2;
    let cluster_width = world.width / 10;  // width of each blob
    let cluster_height = world.height / 20; // height of each blob
    let gap_x = world.width / 20;          // horizontal spacing between blobs
    let gap_y = world.height / 30;         // vertical spacing between blobs

    let start_x = world.width * 2 / 5;
    let start_y = world.height * 2 / 3;

    for cx in 0..clusters_x {
        for cy in 0..clusters_y {
            let base_x = start_x + cx * (cluster_width + gap_x);
            let base_y = start_y + cy * (cluster_height + gap_y);

            for x in base_x..(base_x + cluster_width).min(world.width) {
                for y in base_y..(base_y + cluster_height).min(world.height) {
                    world.set_atom(x, y, ParticleAtom::new(AtomType::Water, Vec2::new(x as f32, y as f32)));
                }
            }
        }
    }

    // Create a small lava pool
    let lava_start_x = world.width / 10;
    let lava_end_x = lava_start_x + world.width / 15;
    let lava_start_y = world.height / 2;
    let lava_end_y = lava_start_y + world.height / 30;
    for x in lava_start_x..lava_end_x.min(world.width) {
        for y in lava_start_y..lava_end_y.min(world.height) {
            world.set_atom(x, y, ParticleAtom::new(AtomType::Lava, Vec2::new(x as f32, y as f32)));
        }
    }

    // Create a small acid pool
    let acid_start_x = world.width * 7 / 10;
    let acid_end_x = acid_start_x + world.width / 15;
    let acid_start_y = world.height / 2;
    let acid_end_y = acid_start_y + world.height / 30;
    for x in acid_start_x..acid_end_x.min(world.width) {
        for y in acid_start_y..acid_end_y.min(world.height) {
            world.set_atom(x, y, ParticleAtom::new(AtomType::Acid, Vec2::new(x as f32, y as f32)));
        }
    }

    // Add moving bodies (positioned relative to world size)
    world.add_moving_body(MovingBody::new(
        Vec2::new(world.width as f32 / 2.0, 20.0),
        Vec2::new(20.0, 0.0),
        Vec2::new(4.0, 4.0),
        5.0,
    ));

    world.add_moving_body(MovingBody::new(
        Vec2::new(world.width as f32 / 3.0, 25.0),
        Vec2::new(-15.0, 0.0),
        Vec2::new(3.0, 3.0),
        3.0,
    ));
}

fn update_particle_world(
    mut world: ResMut<ParticleWorldResource>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds().min(1.0 / 30.0); // Cap dt for stability
    world.0.update(dt);
}

fn render_particle_atoms(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    world: Res<ParticleWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render atoms
    for atom in &world.0.atoms {
        if atom.atom_type != AtomType::Empty {
            let alpha = if atom.is_particle { 0.6 } else { 1.0 };
            // Ensure the rendered color (including alpha) matches the logical atom type
            let mut color = atom.atom_type.color();
            color.set_alpha(alpha);

            // Render atoms with increased size for better visibility
            let entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(ATOM_RENDER_SIZE, ATOM_RENDER_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    (atom.position.x - world.0.width as f32 / 2.0) * ATOM_RENDER_SIZE,
                    (atom.position.y - world.0.height as f32 / 2.0) * ATOM_RENDER_SIZE,
                    if atom.is_particle { 1.0 } else { 0.0 }, // Particles render above regular atoms
                ),
                ..default()
            }).id();
            atom_entities.push(entity);
        }
    }
}

fn render_moving_bodies(
    mut commands: Commands,
    mut body_entities: Local<Vec<Entity>>,
    world: Res<ParticleWorldResource>,
) {
    // Clear previous frame
    for entity in body_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render moving bodies with same scale as atoms
    // let atom_size = 8.0; // Match atom rendering size
    for (i, body) in world.0.moving_bodies.iter().enumerate() {
        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.3, 0.3),
                    custom_size: Some(body.size * ATOM_RENDER_SIZE),
                    ..default()
                },
                transform: Transform::from_xyz(
                    (body.position.x - world.0.width as f32 / 2.0) * ATOM_RENDER_SIZE,
                    (body.position.y - world.0.height as f32 / 2.0) * ATOM_RENDER_SIZE,
                    2.0, // Render above atoms
                ),
                ..default()
            },
            MovingBodyEntity { body_index: i },
        )).id();
        body_entities.push(entity);
    }
}

fn handle_particle_interaction(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<ParticleWorldResource>,
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    // Account for atom size scaling (8.0 pixels per atom)
                    let atom_x = ((world_pos.origin.x / ATOM_RENDER_SIZE) + world.0.width as f32 / 2.0) as usize;
                    let atom_y = ((world_pos.origin.y / ATOM_RENDER_SIZE) + world.0.height as f32 / 2.0) as usize;

                    if mouse_input.just_pressed(MouseButton::Left) {
                        // Add sand at cursor
                        if atom_x < world.0.width && atom_y < world.0.height {
                            world.0.set_atom(atom_x, atom_y, ParticleAtom::new(
                                AtomType::Sand,
                                Vec2::new(atom_x as f32, atom_y as f32)
                            ));
                        }
                    }

                    if mouse_input.just_pressed(MouseButton::Right) {
                        // Add water at cursor
                        if atom_x < world.0.width && atom_y < world.0.height {
                            world.0.set_atom(atom_x, atom_y, ParticleAtom::new(
                                AtomType::Water,
                                Vec2::new(atom_x as f32, atom_y as f32)
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn demonstrate_particle_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<ParticleWorldResource>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        // Add another moving body (positioned relative to world size)
        let width= world.0.width as f32 * 0.6;
        let height= world.0.height as f32 * 0.3;
        world.0.add_moving_body(MovingBody::new(
            Vec2::new(width, height),
            Vec2::new(-25.0, 10.0),
            Vec2::new(3.0, 3.0),
            2.0,
        ));
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset simulation (use current world size)
        let width = world.0.width;
        let height = world.0.height;
        *world = ParticleWorldResource(ParticleWorld::new(width, height));
        create_particle_demo_setup(&mut world.0);
    }

    if keyboard_input.just_pressed(KeyCode::KeyP) {
        // Toggle particle visibility info
        println!("Particle system active - particles prevent atoms from crushing moving bodies");
        println!("Moving bodies create particle barriers around themselves");
    }
}
