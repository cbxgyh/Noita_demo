// Example: Shocking Developments
// Based on "Shocking Developments" blog post
// https://www.slowrush.dev/news/shocking-developments

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Electricity and shock mechanics
// Demonstrates electrical conductivity, shock propagation, and electrical hazards

#[derive(Clone, Debug, PartialEq)]
enum Element {
    Sand,
    Water,
    Metal,
    Electricity,
    Fire,
    Stone,
}

impl Element {
    fn color(&self) -> Color {
        match self {
            Element::Sand => Color::rgb(0.9, 0.8, 0.6),
            Element::Water => Color::rgba(0.3, 0.5, 0.9, 0.8),
            Element::Metal => Color::rgb(0.7, 0.7, 0.8),
            Element::Electricity => Color::rgb(1.0, 1.0, 0.0),
            Element::Fire => Color::rgb(1.0, 0.3, 0.0),
            Element::Stone => Color::rgb(0.5, 0.5, 0.5),
        }
    }

    fn is_conductive(&self) -> bool {
        matches!(self, Element::Metal | Element::Water | Element::Electricity)
    }

    fn is_flammable(&self) -> bool {
        matches!(self, Element::Sand | Element::Water)
    }

    fn density(&self) -> f32 {
        match self {
            Element::Sand => 1.5,
            Element::Water => 1.0,
            Element::Metal => 3.0,
            Element::Electricity => 0.1,
            Element::Fire => 0.8,
            Element::Stone => 2.5,
        }
    }
}

#[derive(Clone, Debug)]
struct Atom {
    position: Vec2,
    velocity: Vec2,
    element: Element,
    temperature: f32,
    charge: f32, // Electrical charge
    lifetime: f32,
    id: u32,
}

impl Atom {
    fn new(position: Vec2, element: Element, id: u32) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            element,
            temperature: 20.0,
            charge: if element == Element::Electricity { 1.0 } else { 0.0 },
            lifetime: match element {
                Element::Electricity => 2.0,
                Element::Fire => 3.0,
                _ => -1.0, // Permanent
            },
            id,
        }
    }

    fn update(&mut self, dt: f32, grid: &AtomGrid) {
        // Apply gravity
        if self.element != Element::Electricity {
            self.velocity.y -= 300.0 * dt * self.element.density();
        }

        // Update position
        self.position += self.velocity * dt;

        // Handle lifetime
        if self.lifetime > 0.0 {
            self.lifetime -= dt;
        }

        // Electrical conductivity and shock propagation
        if self.element.is_conductive() && self.charge > 0.0 {
            // Find nearby conductive atoms and spread charge
            let nearby_atoms = grid.get_nearby_atoms(self.position, 1.5);
            for (other_id, other_pos) in nearby_atoms {
                if *other_id != self.id {
                    let distance = self.position.distance(*other_pos);
                    if distance < 1.5 {
                        // Spread charge to nearby conductive atoms
                        if let Some(other_atom) = grid.get_atom_mut(*other_id) {
                            if other_atom.element.is_conductive() {
                                let charge_transfer = self.charge * 0.1 * dt;
                                self.charge -= charge_transfer;
                                other_atom.charge += charge_transfer;

                                // Create spark visual effect
                                if rand::random::<f32>() < 0.1 {
                                    grid.add_atom(Atom::new(
                                        self.position.lerp(*other_pos, 0.5),
                                        Element::Electricity,
                                        grid.next_id(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Temperature effects
        self.temperature *= 0.99; // Cool down over time

        // Electrical heating
        if self.charge > 0.5 {
            self.temperature += 50.0 * dt;
        }

        // Combustion
        if self.temperature > 100.0 && self.element.is_flammable() {
            if rand::random::<f32>() < 0.01 {
                grid.add_atom(Atom::new(self.position, Element::Fire, grid.next_id()));
            }
        }

        // Fire spread
        if self.element == Element::Fire {
            let nearby_atoms = grid.get_nearby_atoms(self.position, 1.0);
            for (other_id, _) in nearby_atoms {
                if let Some(other_atom) = grid.get_atom_mut(*other_id) {
                    if other_atom.element.is_flammable() && rand::random::<f32>() < 0.05 {
                        other_atom.temperature += 20.0;
                    }
                }
            }
        }
    }

    fn is_alive(&self) -> bool {
        self.lifetime <= 0.0 || self.lifetime > -1.0
    }
}

#[derive(Clone, Debug)]
struct AtomGrid {
    atoms: Vec<Atom>,
    width: usize,
    height: usize,
    cell_size: f32,
    next_atom_id: u32,
}

impl AtomGrid {
    fn new(width: usize, height: usize, cell_size: f32) -> Self {
        Self {
            atoms: Vec::new(),
            width,
            height,
            cell_size,
            next_atom_id: 0,
        }
    }

    fn next_id(&mut self) -> u32 {
        self.next_atom_id += 1;
        self.next_atom_id - 1
    }

    fn add_atom(&mut self, atom: Atom) {
        self.atoms.push(atom);
    }

    fn get_nearby_atoms(&self, position: Vec2, radius: f32) -> Vec<(u32, Vec2)> {
        self.atoms.iter()
            .filter(|atom| atom.position.distance(position) <= radius)
            .map(|atom| (atom.id, atom.position))
            .collect()
    }

    fn get_atom_mut(&mut self, id: u32) -> Option<&mut Atom> {
        self.atoms.iter_mut().find(|atom| atom.id == id)
    }

    fn update(&mut self, dt: f32) {
        // Update all atoms
        for i in 0..self.atoms.len() {
            if let Some(atom) = self.atoms.get_mut(i) {
                atom.update(dt, self);
            }
        }

        // Remove dead atoms
        self.atoms.retain(|atom| atom.is_alive());
    }

    fn create_electrical_source(&mut self, position: Vec2, power: f32) {
        for _ in 0..10 {
            let offset = Vec2::new(
                (rand::random::<f32>() - 0.5) * 10.0,
                (rand::random::<f32>() - 0.5) * 10.0,
            );
            self.add_atom(Atom::new(position + offset, Element::Electricity, self.next_id()));
        }
    }

    fn create_metal_structure(&mut self, start: Vec2, end: Vec2) {
        let distance = start.distance(end);
        let steps = (distance / 2.0) as usize;

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let position = start.lerp(end, t);
            self.add_atom(Atom::new(position, Element::Metal, self.next_id()));
        }
    }
}

#[derive(Clone, Debug)]
struct ShockSystem {
    active_shocks: Vec<ElectricalShock>,
}

#[derive(Clone, Debug)]
struct ElectricalShock {
    start_position: Vec2,
    end_position: Vec2,
    power: f32,
    lifetime: f32,
    max_lifetime: f32,
}

impl ElectricalShock {
    fn new(start: Vec2, end: Vec2, power: f32) -> Self {
        Self {
            start_position: start,
            end_position: end,
            power,
            lifetime: 0.5,
            max_lifetime: 0.5,
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        self.lifetime -= dt;
        self.lifetime > 0.0
    }

    fn intensity(&self) -> f32 {
        self.lifetime / self.max_lifetime
    }
}

impl ShockSystem {
    fn new() -> Self {
        Self {
            active_shocks: Vec::new(),
        }
    }

    fn add_shock(&mut self, shock: ElectricalShock) {
        self.active_shocks.push(shock);
    }

    fn update(&mut self, dt: f32) {
        self.active_shocks.retain_mut(|shock| shock.update(dt));
    }

    fn create_chain_shock(&mut self, grid: &AtomGrid, start_pos: Vec2, max_distance: f32) {
        let mut current_pos = start_pos;
        let mut visited_positions = Vec::new();

        for _ in 0..20 { // Max chain length
            let nearby_atoms = grid.get_nearby_atoms(current_pos, max_distance);
            let mut best_target = None;
            let mut best_distance = max_distance;

            for (_, pos) in nearby_atoms {
                if !visited_positions.contains(pos) {
                    let distance = current_pos.distance(*pos);
                    if distance < best_distance && distance > 1.0 {
                        best_distance = distance;
                        best_target = Some(*pos);
                    }
                }
            }

            if let Some(target) = best_target {
                self.add_shock(ElectricalShock::new(current_pos, target, 1.0));
                visited_positions.push(target);
                current_pos = target;
            } else {
                break;
            }
        }
    }
}

#[derive(Resource)]
struct ShockDemo {
    grid: AtomGrid,
    shock_system: ShockSystem,
    mouse_position: Vec2,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shocking Developments - Electricity & Conductivity".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(ShockDemo {
            grid: AtomGrid::new(80, 60, 10.0),
            shock_system: ShockSystem::new(),
            mouse_position: Vec2::ZERO,
        })
        .add_systems(Startup, setup_shock_demo)
        .add_systems(Update, (
            handle_shock_input,
            update_shock_simulation,
            render_shock_demo,
            display_shock_info,
        ).chain())
        .run();
}

fn setup_shock_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_shock_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<ShockDemo>,
) {
    // Update mouse position
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    demo.mouse_position = world_pos.origin.truncate();
                }
            }
        }
    }

    // Add atoms based on input
    if mouse_input.pressed(MouseButton::Left) {
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            demo.grid.add_atom(Atom::new(demo.mouse_position, Element::Electricity, demo.grid.next_id()));
        } else if keyboard_input.pressed(KeyCode::KeyM) {
            demo.grid.add_atom(Atom::new(demo.mouse_position, Element::Metal, demo.grid.next_id()));
        } else if keyboard_input.pressed(KeyCode::KeyW) {
            demo.grid.add_atom(Atom::new(demo.mouse_position, Element::Water, demo.grid.next_id()));
        } else if keyboard_input.pressed(KeyCode::KeyF) {
            demo.grid.add_atom(Atom::new(demo.mouse_position, Element::Fire, demo.grid.next_id()));
        } else {
            demo.grid.add_atom(Atom::new(demo.mouse_position, Element::Sand, demo.grid.next_id()));
        }
    }

    // Create electrical source
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        demo.grid.create_electrical_source(demo.mouse_position, 1.0);
    }

    // Create metal structures
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        let start = demo.mouse_position;
        let end = demo.mouse_position + Vec2::new(100.0, 0.0);
        demo.grid.create_metal_structure(start, end);
    }

    // Trigger shock chain
    if keyboard_input.just_pressed(KeyCode::Space) {
        demo.shock_system.create_chain_shock(&demo.grid, demo.mouse_position, 50.0);
    }

    // Clear everything
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        demo.grid.atoms.clear();
        demo.shock_system.active_shocks.clear();
    }
}

fn update_shock_simulation(time: Res<Time>, mut demo: ResMut<ShockDemo>) {
    let dt = time.delta_seconds().min(1.0 / 30.0); // Cap delta time

    demo.grid.update(dt);
    demo.shock_system.update(dt);
}

fn render_shock_demo(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    mut shock_entities: Local<Vec<Entity>>,
    demo: Res<ShockDemo>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in shock_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render atoms
    for atom in &demo.grid.atoms {
        let color = atom.element.color();

        // Intensify color based on charge
        let charge_intensity = if atom.charge > 0.0 {
            (atom.charge * 0.5).min(0.5)
        } else {
            0.0
        };

        let final_color = Color::rgb(
            color.r + charge_intensity,
            color.g + charge_intensity,
            color.b + charge_intensity,
        );

        // Size based on charge
        let size = 4.0 + atom.charge * 2.0;

        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: final_color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            transform: Transform::from_xyz(atom.position.x, atom.position.y, 0.0),
            ..default()
        }).id();
        atom_entities.push(entity);
    }

    // Render shock effects
    for shock in &demo.shock_system.active_shocks {
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 0.0, shock.intensity()),
                custom_size: Some(Vec2::new(2.0, shock.start_position.distance(shock.end_position))),
                ..default()
            },
            transform: Transform {
                translation: shock.start_position.extend(1.0),
                rotation: Quat::from_rotation_z(
                    (shock.end_position - shock.start_position).angle_between(Vec2::Y)
                ),
                ..default()
            },
            ..default()
        }).id();
        shock_entities.push(entity);
    }
}

fn display_shock_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<ShockDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Shocking Developments Demo ===");
        println!("Atom Count: {}", demo.grid.atoms.len());
        println!("Active Shocks: {}", demo.shock_system.active_shocks.len());

        let element_counts: std::collections::HashMap<String, usize> =
            demo.grid.atoms.iter()
                .fold(std::collections::HashMap::new(), |mut map, atom| {
                    let key = format!("{:?}", atom.element);
                    *map.entry(key).or_insert(0) += 1;
                    map
                });

        println!("Element Distribution:");
        for (element, count) in element_counts {
            println!("  {}: {}", element, count);
        }

        println!("\nControls:");
        println!("  Left Click: Add atoms (Sand by default)");
        println!("  Left Click + Shift: Add Electricity");
        println!("  Left Click + M: Add Metal");
        println!("  Left Click + W: Add Water");
        println!("  Left Click + F: Add Fire");
        println!("  E: Create electrical source");
        println!("  R: Create metal rod");
        println!("  Space: Trigger shock chain");
        println!("  C: Clear all atoms");
        println!("  H: Show this info");
        println!("======================\n");
    }
}
