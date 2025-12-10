use bevy::prelude::*;
use rand::Rng;

// Atom types as described in the blog series
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomType {
    Empty,
    Sand,
    Water,
    Acid,
    Fire,
    Smoke,
    Steam,
    Poison,
    Stone, // Terrain
}



impl AtomType {
    pub fn color(&self) -> Color {
        match self {
            AtomType::Empty => Color::rgba(0.0, 0.0, 0.0, 0.0),
            AtomType::Sand => Color::rgb(0.8, 0.7, 0.5),
            AtomType::Water => Color::rgba(0.2, 0.4, 0.8, 0.8),
            AtomType::Acid => Color::rgb(0.0, 0.8, 0.0),
            AtomType::Fire => Color::rgb(1.0, 0.3, 0.0),
            AtomType::Smoke => Color::rgba(0.3, 0.3, 0.3, 0.5),
            AtomType::Steam => Color::rgba(0.8, 0.8, 0.9, 0.6),
            AtomType::Poison => Color::rgb(0.5, 0.0, 0.5),
            AtomType::Stone => Color::rgb(0.4, 0.4, 0.4),
        }
    }

    pub fn mass(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 1.6,
            AtomType::Water => 1.0,
            AtomType::Acid => 1.2,
            AtomType::Fire => 0.1,
            AtomType::Smoke => 0.05,
            AtomType::Steam => 0.01,
            AtomType::Poison => 1.1,
            AtomType::Stone => 2.5,
        }
    }

    // Approximate density used for simple force calculations
    pub fn density(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 1.6,
            AtomType::Water => 1.0,
            AtomType::Acid => 1.2,
            AtomType::Fire => 0.1,
            AtomType::Smoke => 0.05,
            AtomType::Steam => 0.01,
            AtomType::Poison => 1.1,
            AtomType::Stone => 2.5,
        }
    }

    pub fn friction(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 0.8,
            AtomType::Water => 0.1,
            AtomType::Acid => 0.2,
            AtomType::Fire => 0.05,
            AtomType::Smoke => 0.01,
            AtomType::Steam => 0.02,
            AtomType::Poison => 0.15,
            AtomType::Stone => 0.9,
        }
    }

    pub fn heat_capacity(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 0.8,
            AtomType::Water => 4.18, // High heat capacity
            AtomType::Acid => 2.0,
            AtomType::Fire => 0.5,
            AtomType::Smoke => 0.3,
            AtomType::Steam => 2.0,
            AtomType::Poison => 1.5,
            AtomType::Stone => 0.8,
        }
    }


    pub fn is_fluid(&self) -> bool {
        matches!(self, AtomType::Water | AtomType::Acid | AtomType::Poison | AtomType::Steam)
    }

    pub fn is_gas(&self) -> bool {
        matches!(self, AtomType::Smoke | AtomType::Steam | AtomType::Fire)
    }

    pub fn can_burn(&self) -> bool {
        matches!(self, AtomType::Sand | AtomType::Stone) // Add flammable materials later
    }
}

// Individual atom component with kinetic properties
#[derive(Component, Clone)]
pub struct Atom {
    pub atom_type: AtomType,
    pub velocity: Vec2,
    pub mass: f32,
    pub lifetime: Option<f32>,
    pub temperature: f32, // For heat-based interactions
}

impl Default for Atom {
    fn default() -> Self {
        Self {
            atom_type: AtomType::Empty,
            velocity: Vec2::ZERO,
            mass: 0.0,
            lifetime: None,
            temperature: 20.0, // Room temperature
        }
    }
}

// World grid for atoms
pub struct AtomWorld {
    pub width: usize,
    pub height: usize,
    pub atoms: Vec<Atom>,
    pub updated: Vec<bool>,
}

impl AtomWorld {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            atoms: vec![Atom::default(); width * height],
            updated: vec![false; width * height],
        }
    }

    pub fn get_index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            Some((y as usize) * self.width + (x as usize))
        } else {
            None
        }
    }

    pub fn get_atom(&self, x: i32, y: i32) -> Option<&Atom> {
        self.get_index(x, y).map(|idx| &self.atoms[idx])
    }

    pub fn get_atom_mut(&mut self, x: i32, y: i32) -> Option<&mut Atom> {
        self.get_index(x, y).map(|idx| &mut self.atoms[idx])
    }

    pub fn set_atom(&mut self, x: i32, y: i32, atom: Atom) {
        if let Some(idx) = self.get_index(x, y) {
            self.atoms[idx] = atom;
        }
    }

    pub fn is_empty(&self, x: i32, y: i32) -> bool {
        self.get_atom(x, y).map_or(true, |atom| atom.atom_type == AtomType::Empty)
    }

    pub fn swap_atoms(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if let (Some(idx1), Some(idx2)) = (self.get_index(x1, y1), self.get_index(x2, y2)) {
            self.atoms.swap(idx1, idx2);
            self.updated[idx1] = true;
            self.updated[idx2] = true;
        }
    }

    pub fn clear_updated(&mut self) {
        self.updated.iter_mut().for_each(|u| *u = false);
    }
}

// Resource for the atom world
#[derive(Resource)]
pub struct AtomWorldResource(pub AtomWorld);

// Systems for atom physics
pub fn update_atoms(
    mut world: ResMut<AtomWorldResource>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    world.0.clear_updated();

    // Apply gravity and other forces first
    apply_gravity(&mut world.0, dt);

    // Update atoms from bottom to top, right to left (to simulate gravity)
    for y in (0..world.0.height).rev() {
        for x in (0..world.0.width).rev() {
            update_atom(&mut world.0, x as i32, y as i32, dt);
        }
    }

    // Apply velocity-based movement
    apply_velocity_movement(&mut world.0, dt);

    // Heat transfer between atoms
    apply_heat_transfer(&mut world.0, dt);

    // Particle interactions (optimized)
    apply_particle_interactions(&mut world.0, dt);
}

fn apply_gravity(world: &mut AtomWorld, dt: f32) {
    let gravity = Vec2::new(0.0, -30.0); // Gravity force

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            if world.atoms[idx].atom_type != AtomType::Empty {
                // Apply gravity based on mass
                let mass = world.atoms[idx].mass;
                if mass > 0.0 {
                    world.atoms[idx].velocity += gravity * dt;
                }

                // Apply friction
                let friction = world.atoms[idx].atom_type.friction();
                world.atoms[idx].velocity *= 1.0 - friction * dt;
            }
        }
    }
}

fn apply_velocity_movement(world: &mut AtomWorld, dt: f32) {
    let mut movements = Vec::new();

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            if world.atoms[idx].atom_type != AtomType::Empty && world.atoms[idx].velocity.length_squared() > 0.01 {
                let new_x = x as f32 + world.atoms[idx].velocity.x * dt;
                let new_y = y as f32 + world.atoms[idx].velocity.y * dt;

                movements.push((x, y, new_x as i32, new_y as i32));
            }
        }
    }

    // Apply movements, handling collisions
    for (old_x, old_y, new_x, new_y) in movements {
        if let Some(atom) = world.get_atom(old_x as i32, old_y as i32).cloned() {
            // Check if new position is valid
            if world.is_empty(new_x, new_y) {
                world.set_atom(old_x as i32, old_y as i32, Atom::default());
                world.set_atom(new_x, new_y, atom);
            } else {
                // Collision - bounce or stop
                let mut atom = atom;
                atom.velocity *= -0.5; // Simple bounce with energy loss
                world.atoms[old_y * world.width + old_x] = atom;
            }
        }
    }
}

fn apply_particle_interactions(world: &mut AtomWorld, dt: f32) {
    // Optimized particle interactions as described in "Particles, for real this time"
    // Only process atoms that are moving or have recently moved

    let mut interaction_pairs = Vec::new();

    // Find atoms that might interact (moving atoms near other atoms)
    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            if world.atoms[idx].atom_type != AtomType::Empty {
                let atom = &world.atoms[idx];

                // Only check moving atoms or atoms near moving objects
                if atom.velocity.length_squared() > 0.1 || is_near_moving_object(world, x, y) {
                    // Check neighboring atoms for interactions
                    for dx in -2..=2 {
                        for dy in -2..=2 {
                            if dx == 0 && dy == 0 { continue; }

                            if let Some(neighbor) = world.get_atom(x as i32 + dx, y as i32 + dy) {
                                if neighbor.atom_type != AtomType::Empty {
                                    interaction_pairs.push(((x, y), (x as i32 + dx, y as i32 + dy)));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Process interactions (limit to prevent performance issues)
    let max_interactions = 1000; // Configurable limit
    for (pos1, pos2) in interaction_pairs.into_iter().take(max_interactions) {
        if let (Some(atom1), Some(atom2)) = (
            world.get_atom(pos1.0 as i32, pos1.1 as i32),
            world.get_atom(pos2.0 as i32, pos2.1 as i32)
        ) {
            // Apply interaction forces
            let force = calculate_interaction_force(atom1, atom2, pos1, pos2);
            apply_force_to_atoms(world, pos1, pos2, force, dt);
        }
    }
}


fn is_near_moving_object(world: &AtomWorld, x: usize, y: usize) -> bool {
    // Check if this atom is near a moving rigid body or fast-moving atom
    // This is a simplified check - in a real implementation, you'd integrate
    // with the physics system to check for nearby moving objects

    // For now, just check for atoms with high velocity nearby
    for dx in -5..=5 {
        for dy in -5..=5 {
            if let Some(atom) = world.get_atom(x as i32 + dx, y as i32 + dy) {
                if atom.velocity.length_squared() > 10.0 {
                    return true;
                }
            }
        }
    }
    false
}

fn calculate_interaction_force(atom1: &Atom, atom2: &Atom, pos1: (usize, usize), pos2: (i32, i32)) -> Vec2 {
    let pos1_vec = Vec2::new(pos1.0 as f32, pos1.1 as f32);
    let pos2_vec = Vec2::new(pos2.0 as f32, pos2.1 as f32);
    let direction = (pos2_vec - pos1_vec).normalize_or_zero();
    let distance = pos1_vec.distance(pos2_vec);

    if distance < 0.1 { return Vec2::ZERO; }

    let mass1 = atom1.mass;
    let mass2 = atom2.mass;

    // Repulsion force between atoms (prevents atoms from occupying same space)
    let repulsion_strength = 50.0;
    let repulsion = direction * repulsion_strength / (distance * distance + 1.0);

    // Attraction based on atom types (e.g., fire attracts flammable materials)
    let attraction = match (atom1.atom_type, atom2.atom_type) {
        (AtomType::Fire, _) if atom2.atom_type.can_burn() => direction * 10.0,
        (_, AtomType::Fire) if atom1.atom_type.can_burn() => -direction * 10.0,
        _ => Vec2::ZERO,
    };

    repulsion + attraction
}

fn apply_force_to_atoms(world: &mut AtomWorld, pos1: (usize, usize), pos2: (i32, i32), force: Vec2, dt: f32) {
    let idx1 = pos1.1 * world.width + pos1.0;
    let mass1 = world.atoms[idx1].mass;

    if let Some(idx2) = world.get_index(pos2.0, pos2.1) {
        let mass2 = world.atoms[idx2].mass;

        // Apply equal and opposite forces
        if mass1 > 0.0 {
            world.atoms[idx1].velocity += force * dt / mass1;
        }
        if mass2 > 0.0 {
            world.atoms[idx2].velocity -= force * dt / mass2;
        }
    }
}

fn update_atom(world: &mut AtomWorld, x: i32, y: i32, dt: f32) {
    let idx = match world.get_index(x, y) {
        Some(idx) => idx,
        None => return,
    };

    if world.updated[idx] {
        return;
    }

    let atom = world.atoms[idx].clone();
    if atom.atom_type == AtomType::Empty {
        return;
    }

    match atom.atom_type {
        AtomType::Sand => update_sand(world, x, y),
        AtomType::Water => update_water(world, x, y),
        AtomType::Acid => update_acid(world, x, y),
        AtomType::Fire => update_fire(world, x, y),
        AtomType::Smoke | AtomType::Steam => update_gas(world, x, y),
        AtomType::Poison => update_poison(world, x, y),
        _ => {} // Stone and other static atoms don't move
    }

    // Update lifetime for temporary atoms
    if let Some(lifetime) = world.atoms[idx].lifetime.as_mut() {
        *lifetime -= dt;
        if *lifetime <= 0.0 {
            world.atoms[idx] = Atom::default();
        }
    }
}

// Simple heat diffusion between neighboring atoms.
fn apply_heat_transfer(world: &mut AtomWorld, dt: f32) {
    // Collect temperature deltas to avoid in-place interference.
    let mut temp_changes = vec![0.0f32; world.atoms.len()];
    let conductivity = 2.0; // tweakable conductivity coefficient

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            let atom = &world.atoms[idx];
            if atom.atom_type == AtomType::Empty {
                continue;
            }

            // 4-neighborhood diffusion
            let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            for (dx, dy) in neighbors {
                if let Some(nidx) = world.get_index(x as i32 + dx, y as i32 + dy) {
                    let neighbor = &world.atoms[nidx];
                    if neighbor.atom_type == AtomType::Empty {
                        continue;
                    }

                    let temp_diff = atom.temperature - neighbor.temperature;
                    // Heat flows from hot to cold
                    let flow = temp_diff * conductivity * dt;
                    temp_changes[idx] -= flow;
                    temp_changes[nidx] += flow;
                }
            }
        }
    }

    // Apply accumulated temperature changes
    for (atom, delta) in world.atoms.iter_mut().zip(temp_changes.into_iter()) {
        atom.temperature += delta;
    }
}

fn update_sand(world: &mut AtomWorld, x: i32, y: i32) {
    let idx = world.get_index(x, y).unwrap();

    // Sand falls down, but also responds to velocity
    if world.is_empty(x, y + 1) {
        world.swap_atoms(x, y, x, y + 1);
    } else if world.atoms[idx].velocity.x > 0.1 && world.is_empty(x + 1, y) {
        world.swap_atoms(x, y, x + 1, y);
    } else if world.atoms[idx].velocity.x < -0.1 && world.is_empty(x - 1, y) {
        world.swap_atoms(x, y, x - 1, y);
    } else if world.is_empty(x - 1, y + 1) {
        world.swap_atoms(x, y, x - 1, y + 1);
    } else if world.is_empty(x + 1, y + 1) {
        world.swap_atoms(x, y, x + 1, y + 1);
    }
}

fn update_water(world: &mut AtomWorld, x: i32, y: i32) {
    // Water falls down or flows sideways
    if world.is_empty(x, y + 1) {
        world.swap_atoms(x, y, x, y + 1);
    } else {
        let mut rng = rand::thread_rng();
        let dir = if rng.gen_bool(0.5) { -1 } else { 1 };

        if world.is_empty(x + dir, y) {
            world.swap_atoms(x, y, x + dir, y);
        } else if world.is_empty(x - dir, y) {
            world.swap_atoms(x, y, x - dir, y);
        }
    }
}

fn update_acid(world: &mut AtomWorld, x: i32, y: i32) {
    // Acid behaves like water but corrodes things
    update_water(world, x, y);

    // Corrosion logic will be added with reactions
}

fn update_fire(world: &mut AtomWorld, x: i32, y: i32) {
    // Fire rises and spreads
    if world.is_empty(x, y - 1) {
        world.swap_atoms(x, y, x, y - 1);
    } else {
        let mut rng = rand::thread_rng();
        let dir = if rng.gen_bool(0.5) { -1 } else { 1 };

        if world.is_empty(x + dir, y - 1) {
            world.swap_atoms(x, y, x + dir, y - 1);
        } else if world.is_empty(x - dir, y - 1) {
            world.swap_atoms(x, y, x - dir, y - 1);
        }
    }

    // Fire spreads to flammable materials
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 { continue; }
            if let Some(atom) = world.get_atom(x + dx, y + dy) {
                if atom.atom_type.can_burn() && rand::random::<f32>() < 0.01 {
                    // Chance to ignite
                    if let Some(mut_atom) = world.get_atom_mut(x + dx, y + dy) {
                        mut_atom.atom_type = AtomType::Fire;
                        mut_atom.lifetime = Some(5.0); // Fire burns for 5 seconds
                    }
                }
            }
        }
    }
}

fn update_gas(world: &mut AtomWorld, x: i32, y: i32) {
    // Gases rise
    if world.is_empty(x, y - 1) {
        world.swap_atoms(x, y, x, y - 1);
    } else {
        let mut rng = rand::thread_rng();
        let dir = if rng.gen_bool(0.5) { -1 } else { 1 };

        if world.is_empty(x + dir, y - 1) {
            world.swap_atoms(x, y, x + dir, y - 1);
        } else if world.is_empty(x - dir, y - 1) {
            world.swap_atoms(x, y, x - dir, y - 1);
        }
    }
}

fn update_poison(world: &mut AtomWorld, x: i32, y: i32) {
    update_water(world, x, y);
    // Poison logic will be expanded
}

// Chemical reactions between atoms
pub fn process_reactions(mut world: ResMut<AtomWorldResource>) {
    let world = &mut world.0;
    let mut reactions = Vec::new();

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            let atom = &world.atoms[idx];

            if atom.atom_type == AtomType::Empty {
                continue;
            }

            // Check neighboring atoms for reactions
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 { continue; }

                    if let Some(neighbor) = world.get_atom(x as i32 + dx, y as i32 + dy) {
                        let reaction = check_reaction(atom.atom_type, neighbor.atom_type);
                        if reaction.is_some() {
                            reactions.push(((x as i32, y as i32), (x as i32 + dx, y as i32 + dy), reaction.unwrap()));
                        }
                    }
                }
            }
        }
    }

    // Apply reactions
    for ((x1, y1), (x2, y2), products) in reactions {
        for product in products {
            // Find empty space for new atoms
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let nx = x1 + dx;
                    let ny = y1 + dy;
                    if world.is_empty(nx, ny) {
                        world.set_atom(nx, ny, Atom {
                            atom_type: product.0,
                            velocity: Vec2::ZERO,
                            mass: product.0.mass(),
                            lifetime: product.1,
                            temperature: 20.0,
                        });
                        break;
                    }
                }
            }
        }

        // Remove reactants
        world.set_atom(x1, y1, Atom::default());
        world.set_atom(x2, y2, Atom::default());
    }
}

fn check_reaction(atom1: AtomType, atom2: AtomType) -> Option<Vec<(AtomType, Option<f32>)>> {
    match (atom1, atom2) {
        (AtomType::Fire, AtomType::Water) | (AtomType::Water, AtomType::Fire) => {
            // Fire + Water = Steam
            Some(vec![(AtomType::Steam, Some(3.0))])
        }
        (AtomType::Acid, AtomType::Water) | (AtomType::Water, AtomType::Acid) => {
            // Acid + Water = Poison
            Some(vec![(AtomType::Poison, None)])
        }
        (AtomType::Fire, AtomType::Sand) | (AtomType::Sand, AtomType::Fire) => {
            // Fire + Sand = Smoke (when sand is flammable)
            Some(vec![(AtomType::Smoke, Some(2.0))])
        }
        _ => None,
    }
}
