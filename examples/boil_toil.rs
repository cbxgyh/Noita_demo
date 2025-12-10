// Example: Boil and Toil
// Based on "Boil and Toil" blog post
// https://www.slowrush.dev/news/boil-and-toil

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Steam and poison systems
// Demonstrates fire boiling water into steam, and acid mixing with water to create poison

#[derive(Clone, Debug)]
enum Substance {
    Empty,
    Stone,
    Sand,
    Water,
    Acid,
    Fire,
    Steam,
    Poison,
}

impl Substance {
    fn color(&self) -> Color {
        match self {
            Substance::Empty => Color::rgba(0.0, 0.0, 0.0, 0.0),
            Substance::Stone => Color::rgb(0.4, 0.4, 0.4),
            Substance::Sand => Color::rgb(0.8, 0.7, 0.5),
            Substance::Water => Color::rgba(0.2, 0.4, 0.8, 0.8),
            Substance::Acid => Color::rgb(0.0, 0.8, 0.0),
            Substance::Fire => Color::rgb(1.0, 0.3, 0.0),
            Substance::Steam => Color::rgba(0.9, 0.9, 1.0, 0.6),
            Substance::Poison => Color::rgb(0.5, 0.0, 0.5),
        }
    }

    fn density(&self) -> f32 {
        match self {
            Substance::Empty => 0.0,
            Substance::Stone => 2.5,
            Substance::Sand => 1.6,
            Substance::Water => 1.0,
            Substance::Acid => 1.2,
            Substance::Fire => 0.1,
            Substance::Steam => 0.01, // Steam is very light
            Substance::Poison => 1.1,
        }
    }

    fn is_gas(&self) -> bool {
        matches!(self, Substance::Steam)
    }

    fn is_fluid(&self) -> bool {
        matches!(self, Substance::Water | Substance::Acid | Substance::Poison)
    }

    fn temperature(&self) -> f32 {
        match self {
            Substance::Empty => 20.0,
            Substance::Stone => 20.0,
            Substance::Sand => 20.0,
            Substance::Water => 20.0,
            Substance::Acid => 25.0,
            Substance::Fire => 800.0,
            Substance::Steam => 150.0, // Hot steam
            Substance::Poison => 22.0,
        }
    }

    fn boiling_point(&self) -> f32 {
        match self {
            Substance::Water => 100.0,
            Substance::Acid => 120.0,
            _ => 1000.0, // Won't boil
        }
    }
}

#[derive(Clone, Debug)]
struct ChemicalAtom {
    substance: Substance,
    position: Vec2,
    velocity: Vec2,
    temperature: f32,
    lifetime: Option<f32>,
    mass: f32,
}

impl ChemicalAtom {
    fn new(substance: Substance, position: Vec2) -> Self {
        Self {
            substance: substance.clone(),
            position,
            velocity: Vec2::ZERO,
            temperature: substance.temperature(),
            lifetime: None,
            mass: substance.density(),
        }
    }

    fn update(&mut self, dt: f32, bounds: Vec2) {
        if self.substance == Substance::Empty {
            return;
        }

        // Apply gravity (gases rise, solids fall)
        let gravity = if self.substance.is_gas() {
            Vec2::new(0.0, 10.0) // Gases rise
        } else {
            Vec2::new(0.0, -30.0) // Solids fall
        };

        self.velocity += gravity * dt;

        // Apply drag
        let drag = if self.substance.is_gas() { 0.95 } else { 0.98 };
        self.velocity *= drag;

        // Update position
        self.position += self.velocity * dt;

        // Bounds checking
        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.x *= -0.5;
        } else if self.position.x >= bounds.x {
            self.position.x = bounds.x - 1.0;
            self.velocity.x *= -0.5;
        }

        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y *= -0.5;
        } else if self.position.y >= bounds.y {
            self.position.y = bounds.y - 1.0;
            self.velocity.y *= -0.5;
        }

        // Update lifetime for temporary substances
        if let Some(ref mut lifetime) = self.lifetime {
            *lifetime -= dt;
            if *lifetime <= 0.0 {
                self.substance = Substance::Empty;
            }
        }

        // Heat transfer and reactions
        self.handle_thermal_effects(dt);
    }

    fn handle_thermal_effects(&mut self, dt: f32) {
        // Steam cools down over time
        if self.substance == Substance::Steam {
            self.temperature = (self.temperature - 50.0 * dt).max(20.0);
            if self.temperature <= 25.0 {
                // Steam condenses back to water
                self.substance = Substance::Water;
                self.temperature = 20.0;
                self.lifetime = None;
            }
        }

        // Fire spreads heat
        if self.substance == Substance::Fire {
            // Fire gradually dies out
            self.lifetime = Some(self.lifetime.unwrap_or(10.0) - dt);
        }
    }

    fn can_react_with(&self, other: &Substance) -> bool {
        match (&self.substance, other) {
            (Substance::Fire, Substance::Water) | (Substance::Water, Substance::Fire) => true,
            (Substance::Acid, Substance::Water) | (Substance::Water, Substance::Acid) => true,
            _ => false,
        }
    }

    fn react_with(&mut self, other: &mut ChemicalAtom) -> Vec<ChemicalAtom> {
        let mut products = Vec::new();

        match (&self.substance, &other.substance) {
            // Fire + Water = Steam
            (Substance::Fire, Substance::Water) | (Substance::Water, Substance::Fire) => {
                // Create steam at the reaction site
                for _ in 0..3 {
                    let mut steam = ChemicalAtom::new(Substance::Steam, self.position);
                    steam.velocity = Vec2::new(
                        (rand::random::<f32>() - 0.5) * 20.0,
                        rand::random::<f32>() * 30.0 + 10.0, // Steam rises
                    );
                    steam.lifetime = Some(5.0);
                    steam.temperature = 150.0;
                    products.push(steam);
                }

                // Remove reactants
                self.substance = Substance::Empty;
                other.substance = Substance::Empty;

                println!("💨 Fire + Water = Steam!");
            }

            // Acid + Water = Poison
            (Substance::Acid, Substance::Water) | (Substance::Water, Substance::Acid) => {
                // Create poison
                for _ in 0..2 {
                    let mut poison = ChemicalAtom::new(Substance::Poison, self.position);
                    poison.velocity = Vec2::new(
                        (rand::random::<f32>() - 0.5) * 15.0,
                        (rand::random::<f32>() - 0.5) * 15.0,
                    );
                    poison.temperature = 22.0;
                    products.push(poison);
                }

                // Remove reactants
                self.substance = Substance::Empty;
                other.substance = Substance::Empty;

                println!("☠️ Acid + Water = Poison!");
            }

            _ => {}
        }

        products
    }
}

struct ChemicalWorld {
    width: usize,
    height: usize,
    atoms: Vec<ChemicalAtom>,
    bounds: Vec2,
    reaction_cooldown: f32,
}

impl ChemicalWorld {
    fn new(width: usize, height: usize) -> Self {
        let mut atoms = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let mut substance = Substance::Empty;

                // Create terrain
                if y < 8 {
                    substance = Substance::Stone;
                }

                atoms.push(ChemicalAtom::new(substance, Vec2::new(x as f32, y as f32)));
            }
        }

        Self {
            width,
            height,
            atoms,
            bounds: Vec2::new(width as f32, height as f32),
            reaction_cooldown: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        // Update all atoms
        for atom in &mut self.atoms {
            atom.update(dt, self.bounds);
        }

        // Process chemical reactions
        self.reaction_cooldown -= dt;
        if self.reaction_cooldown <= 0.0 {
            self.process_reactions(dt);
            self.reaction_cooldown = 0.1; // Process every 100ms
        }

        // Clean up expired atoms
        for atom in &mut self.atoms {
            if atom.substance == Substance::Fire && atom.lifetime.unwrap_or(1.0) <= 0.0 {
                atom.substance = Substance::Empty;
            }
        }
    }

    fn process_reactions(&mut self, dt: f32) {
        let mut reaction_products = Vec::new();

        // Check each atom against neighbors
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if self.atoms[idx].substance == Substance::Empty {
                    continue;
                }

                // Check neighboring atoms for reactions
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 { continue; }

                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                            let nidx = ny as usize * self.width + nx as usize;

                            if self.atoms[idx].can_react_with(&self.atoms[nidx].substance) {
                                let products = self.atoms[idx].react_with(&mut self.atoms[nidx]);
                                reaction_products.extend(products);
                            }
                        }
                    }
                }
            }
        }

        // Add reaction products to world
        for product in reaction_products {
            self.add_atom_at_position(product);
        }
    }

    fn add_atom_at_position(&mut self, atom: ChemicalAtom) {
        let x = atom.position.x.round() as usize;
        let y = atom.position.y.round() as usize;

        if x < self.width && y < self.height {
            let idx = y * self.width + x;

            // Only place if position is empty
            if self.atoms[idx].substance == Substance::Empty {
                self.atoms[idx] = atom;
            }
        }
    }

    fn add_substance_at(&mut self, substance: Substance, position: Vec2, spread: f32, count: usize) {
        for _ in 0..count {
            let offset = Vec2::new(
                (rand::random::<f32>() - 0.5) * spread,
                (rand::random::<f32>() - 0.5) * spread,
            );
            let pos = position + offset;
            let velocity = Vec2::new(
                (rand::random::<f32>() - 0.5) * 10.0,
                rand::random::<f32>() * 5.0,
            );

            let mut atom = ChemicalAtom::new(substance.clone(), pos);
            atom.velocity = velocity;

            if substance == Substance::Fire {
                atom.lifetime = Some(8.0);
            }

            self.add_atom_at_position(atom);
        }
    }

    fn get_atom(&self, x: usize, y: usize) -> Option<&ChemicalAtom> {
        if x < self.width && y < self.height {
            Some(&self.atoms[y * self.width + x])
        } else {
            None
        }
    }
}

#[derive(Resource)]
struct ChemicalWorldResource(ChemicalWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Boil and Toil - Steam and Poison Chemistry".to_string(),
                resolution: (1000, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(ChemicalWorldResource(ChemicalWorld::new(150, 100)))
        .add_systems(Startup, setup_chemical_demo)
        .add_systems(Update, (
            update_chemical_world,
            render_chemical_world,
            handle_chemical_input,
            demonstrate_chemistry,
        ).chain())
        .run();
}

fn setup_chemical_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_chemical_world(mut world: ResMut<ChemicalWorldResource>, time: Res<Time>) {
    let dt = time.delta_seconds().min(1.0 / 30.0);
    world.0.update(dt);
}

fn render_chemical_world(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    world: Res<ChemicalWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render atoms
    for atom in &world.0.atoms {
        if atom.substance != Substance::Empty {
            let mut color = atom.substance.color();

            // Show temperature with brightness
            let temp_factor = ((atom.temperature - 20.0) / 200.0).max(0.0).min(1.0);
            color = color.with_r((color.r() + temp_factor * 0.3).min(1.0))
                        .with_g((color.g() + temp_factor * 0.3).min(1.0))
                        .with_b((color.b() + temp_factor * 0.3).min(1.0));

            // Steam has different opacity based on temperature
            if atom.substance == Substance::Steam {
                let alpha = (atom.temperature / 150.0).max(0.3);
                color = color.with_a(alpha);
            }

            let entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    atom.position.x - world.0.width as f32 / 2.0,
                    atom.position.y - world.0.height as f32 / 2.0,
                    if atom.substance.is_gas() { 1.0 } else { 0.0 },
                ),
                ..default()
            }).id();
            atom_entities.push(entity);
        }
    }
}

fn handle_chemical_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<ChemicalWorldResource>,
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    let atom_x = (world_pos.origin.x + world.0.width as f32 / 2.0) as f32;
                    let atom_y = (world_pos.origin.y + world.0.height as f32 / 2.0) as f32;
                    let position = Vec2::new(atom_x, atom_y);

                    if mouse_input.just_pressed(MouseButton::Left) {
                        world.0.add_substance_at(Substance::Water, position, 3.0, 5);
                    }

                    if mouse_input.just_pressed(MouseButton::Right) {
                        world.0.add_substance_at(Substance::Fire, position, 2.0, 3);
                    }

                    if mouse_input.just_pressed(MouseButton::Middle) {
                        world.0.add_substance_at(Substance::Acid, position, 2.0, 3);
                    }
                }
            }
        }
    }

    // Keyboard shortcuts
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        world.0.add_substance_at(Substance::Fire, Vec2::new(50.0, 50.0), 3.0, 3);
    }

    if keyboard_input.just_pressed(KeyCode::KeyW) {
        world.0.add_substance_at(Substance::Water, Vec2::new(75.0, 50.0), 5.0, 8);
    }

    if keyboard_input.just_pressed(KeyCode::KeyA) {
        world.0.add_substance_at(Substance::Acid, Vec2::new(100.0, 50.0), 3.0, 4);
    }

    if keyboard_input.just_pressed(KeyCode::KeyS) {
        world.0.add_substance_at(Substance::Sand, Vec2::new(125.0, 50.0), 8.0, 15);
    }
}

fn demonstrate_chemistry(keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Boil and Toil Chemistry Demo ===");
        println!("Chemical Reactions:");
        println!("💨 Fire + Water = Steam (fire extinguished, steam rises)");
        println!("☠️ Acid + Water = Poison (dangerous reaction)");
        println!("");
        println!("Substance Properties:");
        println!("- Steam: Rises, cools, condenses back to water");
        println!("- Poison: Dense fluid, harmful");
        println!("- Temperature affects appearance brightness");
        println!("- Gases are lighter and rise");
        println!("");
        println!("Controls:");
        println!("Left click: Add Water");
        println!("Right click: Add Fire");
        println!("Middle click: Add Acid");
        println!("F: Fire | W: Water | A: Acid | S: Sand");
        println!("H: Show this help");
        println!("=====================================\n");
    }
}
