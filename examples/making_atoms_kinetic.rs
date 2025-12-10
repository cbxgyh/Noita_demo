// Example: Making Atoms Kinetic
// Based on "Making Atoms Kinetic" blog post
// https://www.slowrush.dev/news/making-atoms-kinetic

use bevy::prelude::*;
use rand::Rng;

// Kinetic atoms with inertia and velocity
// Demonstrates how atoms gain momentum and behave more realistically

#[derive(Clone, Debug)]
enum AtomType {
    Empty,
    Sand,
    Water,
    Stone,
}

impl AtomType {
    fn mass(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 1.6,
            AtomType::Water => 1.0,
            AtomType::Stone => 2.5,
        }
    }

    fn friction(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 0.9, // High friction, stops quickly
            AtomType::Water => 0.1, // Low friction, flows easily
            AtomType::Stone => 0.95, // Very high friction
        }
    }

    fn restitution(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 0.1, // Little bounce
            AtomType::Water => 0.0, // No bounce
            AtomType::Stone => 0.3, // Some bounce
        }
    }

    fn color(&self) -> Color {
        match self {
            AtomType::Empty => Color::rgba(0.0, 0.0, 0.0, 0.0),
            AtomType::Sand => Color::rgb(0.8, 0.7, 0.5),
            AtomType::Water => Color::rgba(0.2, 0.4, 0.8, 0.8),
            AtomType::Stone => Color::rgb(0.4, 0.4, 0.4),
        }
    }
}

#[derive(Clone, Debug)]
struct KineticAtom {
    atom_type: AtomType,
    velocity: Vec2,
    position: Vec2,
    age: f32,
    last_position: Vec2, // For collision detection
}

impl KineticAtom {
    fn new(atom_type: AtomType, position: Vec2) -> Self {
        Self {
            atom_type,
            velocity: Vec2::ZERO,
            position,
            age: 0.0,
            last_position: position,
        }
    }

    fn update(&mut self, dt: f32, world_bounds: Vec2) {
        if self.atom_type == AtomType::Empty {
            return;
        }

        self.age += dt;

        // Store last position for collision detection
        self.last_position = self.position;

        // Apply gravity (only affects atoms with mass)
        if self.atom_type.mass() > 0.0 {
            self.velocity.y -= 30.0 * dt; // Gravity
        }

        // Apply friction
        let friction = self.atom_type.friction();
        self.velocity *= 1.0 - friction * dt;

        // Update position
        self.position += self.velocity * dt;

        // World bounds collision
        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.x *= -self.atom_type.restitution();
        } else if self.position.x >= world_bounds.x {
            self.position.x = world_bounds.x - 1.0;
            self.velocity.x *= -self.atom_type.restitution();
        }

        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y *= -self.atom_type.restitution();
        } else if self.position.y >= world_bounds.y {
            self.position.y = world_bounds.y - 1.0;
            self.velocity.y *= -self.atom_type.restitution();
        }

        // Minimum velocity threshold (stop very slow atoms)
        if self.velocity.length_squared() < 0.01 {
            self.velocity = Vec2::ZERO;
        }
    }

    fn apply_force(&mut self, force: Vec2, dt: f32) {
        let mass = self.atom_type.mass();
        if mass > 0.0 {
            self.velocity += force / mass * dt;
        }
    }

    fn collides_with(&self, other: &KineticAtom) -> bool {
        let dx = self.position.x - other.position.x;
        let dy = self.position.y - other.position.y;
        let distance_squared = dx * dx + dy * dy;
        distance_squared < 1.0 // Atoms are 1 unit in size
    }

    fn resolve_collision(&mut self, other: &mut KineticAtom) {
        if self.atom_type == AtomType::Empty || other.atom_type == AtomType::Empty {
            return;
        }

        let normal = (other.position - self.position).normalize_or(Vec2::X);

        // Separate atoms
        let separation = 1.0 - (other.position - self.position).length();
        if separation > 0.0 {
            let separation_vector = normal * separation * 0.5;
            self.position -= separation_vector;
            other.position += separation_vector;
        }

        // Calculate relative velocity
        let relative_velocity = other.velocity - self.velocity;
        let velocity_along_normal = relative_velocity.dot(normal);

        // Don't resolve if velocities are separating
        if velocity_along_normal > 0.0 {
            return;
        }

        // Calculate restitution
        let restitution = (self.atom_type.restitution() + other.atom_type.restitution()) * 0.5;

        // Calculate impulse scalar
        let mass1 = self.atom_type.mass();
        let mass2 = other.atom_type.mass();
        let impulse_scalar = -(1.0 + restitution) * velocity_along_normal / (1.0/mass1 + 1.0/mass2);

        // Apply impulse
        let impulse = normal * impulse_scalar;
        self.velocity -= impulse / mass1;
        other.velocity += impulse / mass2;
    }
}

struct KineticAtomWorld {
    width: usize,
    height: usize,
    atoms: Vec<KineticAtom>,
    world_bounds: Vec2,
}

impl KineticAtomWorld {
    fn new(width: usize, height: usize) -> Self {
        let mut atoms = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                atoms.push(KineticAtom::new(AtomType::Empty, Vec2::new(x as f32, y as f32)));
            }
        }

        Self {
            width,
            height,
            atoms,
            world_bounds: Vec2::new(width as f32, height as f32),
        }
    }

    fn get_atom(&self, x: usize, y: usize) -> Option<&KineticAtom> {
        if x < self.width && y < self.height {
            Some(&self.atoms[y * self.width + x])
        } else {
            None
        }
    }

    fn get_atom_mut(&mut self, x: usize, y: usize) -> Option<&mut KineticAtom> {
        if x < self.width && y < self.height {
            Some(&mut self.atoms[y * self.width + x])
        } else {
            None
        }
    }

    fn set_atom(&mut self, x: usize, y: usize, atom: KineticAtom) {
        if x < self.width && y < self.height {
            self.atoms[y * self.width + x] = atom;
        }
    }

    fn is_empty(&self, x: usize, y: usize) -> bool {
        self.get_atom(x, y).map_or(true, |a| matches!(a.atom_type, AtomType::Empty))
    }

    fn update(&mut self, dt: f32) {
        // Update all atoms
        for atom in &mut self.atoms {
            atom.update(dt, self.world_bounds);
        }

        // Resolve collisions
        self.resolve_collisions();
    }

    fn resolve_collisions(&mut self) {
        // Simple collision detection - check each atom against its neighbors
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(atom) = self.get_atom_mut(x, y) {
                    if atom.atom_type == AtomType::Empty {
                        continue;
                    }

                    // Check neighbors
                    let neighbors = [
                        (x.wrapping_sub(1), y),
                        (x + 1, y),
                        (x, y.wrapping_sub(1)),
                        (x, y + 1),
                        (x.wrapping_sub(1), y.wrapping_sub(1)),
                        (x + 1, y.wrapping_sub(1)),
                        (x.wrapping_sub(1), y + 1),
                        (x + 1, y + 1),
                    ];

                    for (nx, ny) in neighbors {
                        if nx < self.width && ny < self.height {
                            if let Some(other_atom) = self.get_atom_mut(nx, ny) {
                                if other_atom.atom_type != AtomType::Empty && atom.collides_with(other_atom) {
                                    let mut atom_copy = atom.clone();
                                    let mut other_copy = other_atom.clone();

                                    atom_copy.resolve_collision(&mut other_copy);

                                    *atom = atom_copy;
                                    *other_atom = other_copy;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn add_force_at_position(&mut self, position: Vec2, force: Vec2, radius: f32) {
        let radius_squared = radius * radius;

        for atom in &mut self.atoms {
            if atom.atom_type != AtomType::Empty {
                let distance_squared = (atom.position - position).length_squared();
                if distance_squared <= radius_squared && distance_squared > 0.0 {
                    let distance = distance_squared.sqrt();
                    let falloff = 1.0 - (distance / radius);
                    atom.apply_force(force * falloff, 0.016); // Assume 60fps dt
                }
            }
        }
    }

    fn create_fountain(&mut self, position: Vec2, initial_velocity: Vec2, spread: f32, count: usize) {
        let mut rng = rand::thread_rng();

        for _ in 0..count {
            let angle = rng.gen_range(-spread..spread);
            let speed = rng.gen_range(50.0..100.0);
            let velocity = Vec2::new(
                angle.cos() * speed,
                angle.sin() * speed + initial_velocity.y,
            );

            let pos_x = (position.x + rng.gen_range(-5.0..5.0)) as usize;
            let pos_y = position.y as usize;

            if pos_x < self.width && pos_y < self.height {
                let mut atom = KineticAtom::new(AtomType::Water, Vec2::new(pos_x as f32, pos_y as f32));
                atom.velocity = velocity;
                self.set_atom(pos_x, pos_y, atom);
            }
        }
    }
}

#[derive(Resource)]
struct KineticWorldResource(KineticAtomWorld);

#[derive(Component)]
struct AtomSprite;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Making Atoms Kinetic - Inertia and Momentum".to_string(),
                resolution: (1000.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(KineticWorldResource(KineticAtomWorld::new(100, 80)))
        .add_systems(Startup, setup_kinetic_demo)
        .add_systems(Update, (
            update_kinetic_world,
            render_kinetic_atoms,
            handle_user_interaction,
            demonstrate_kinetics,
        ).chain())
        .run();
}

fn setup_kinetic_demo(mut commands: Commands, mut world: ResMut<KineticWorldResource>) {
    commands.spawn(Camera2dBundle::default());

    // Create initial setup
    create_demo_setup(&mut world.0);
}

fn create_demo_setup(world: &mut KineticAtomWorld) {
    // Create a stone floor
    for x in 0..world.width {
        for y in 0..5 {
            world.set_atom(x, y, KineticAtom::new(AtomType::Stone, Vec2::new(x as f32, y as f32)));
        }
    }

    // Create a sand pile in the center
    for x in 40..60 {
        for y in 10..25 {
            if rand::random::<f32>() < 0.8 {
                world.set_atom(x, y, KineticAtom::new(AtomType::Sand, Vec2::new(x as f32, y as f32)));
            }
        }
    }

    // Create a water reservoir
    for x in 20..35 {
        for y in 15..25 {
            if rand::random::<f32>() < 0.9 {
                world.set_atom(x, y, KineticAtom::new(AtomType::Water, Vec2::new(x as f32, y as f32)));
            }
        }
    }
}

fn update_kinetic_world(
    mut world: ResMut<KineticWorldResource>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    world.0.update(dt);
}

fn render_kinetic_atoms(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    world: Res<KineticWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render current atoms
    for atom in &world.0.atoms {
        if atom.atom_type != AtomType::Empty {
            let entity = commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: atom.atom_type.color(),
                        custom_size: Some(Vec2::new(1.0, 1.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        atom.position.x - world.0.width as f32 / 2.0,
                        atom.position.y - world.0.height as f32 / 2.0,
                        0.0,
                    ),
                    ..default()
                },
                AtomSprite,
            )).id();
            atom_entities.push(entity);
        }
    }
}

fn handle_user_interaction(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<KineticWorldResource>,
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    let atom_x = (world_pos.origin.x + world.0.width as f32 / 2.0) as usize;
                    let atom_y = (world_pos.origin.y + world.0.height as f32 / 2.0) as usize;

                    if mouse_input.just_pressed(MouseButton::Left) {
                        // Add force at cursor position
                        let force_position = Vec2::new(atom_x as f32, atom_y as f32);
                        world.0.add_force_at_position(force_position, Vec2::new(0.0, 100.0), 10.0);
                    }

                    if mouse_input.just_pressed(MouseButton::Right) {
                        // Create fountain at cursor
                        let fountain_pos = Vec2::new(atom_x as f32, atom_y as f32);
                        world.0.create_fountain(fountain_pos, Vec2::new(0.0, 50.0), std::f32::consts::PI / 4.0, 20);
                    }
                }
            }
        }
    }
}

fn demonstrate_kinetics(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<KineticWorldResource>,
) {
    // Demonstrate different kinetic behaviors
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        // Spawn sand with initial velocity
        for i in 0..10 {
            let x = 50 + i;
            let y = 60;
            if x < world.0.width && y < world.0.height {
                let mut atom = KineticAtom::new(AtomType::Sand, Vec2::new(x as f32, y as f32));
                atom.velocity = Vec2::new(rand::random::<f32>() * 40.0 - 20.0, rand::random::<f32>() * 20.0);
                world.0.set_atom(x, y, atom);
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyW) {
        // Spawn water with momentum
        for i in 0..15 {
            let x = 25 + i;
            let y = 50;
            if x < world.0.width && y < world.0.height {
                let mut atom = KineticAtom::new(AtomType::Water, Vec2::new(x as f32, y as f32));
                atom.velocity = Vec2::new(rand::random::<f32>() * 60.0 - 30.0, rand::random::<f32>() * 30.0);
                world.0.set_atom(x, y, atom);
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset simulation
        *world = KineticWorldResource(KineticAtomWorld::new(100, 80));
        create_demo_setup(&mut world.0);
    }
}
