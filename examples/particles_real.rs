// Example: Particles, for real this time
// Based on "Particles, for real this time" blog post
// https://www.slowrush.dev/news/particles-for-real-this-time

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Particle system to prevent atoms from crushing moving bodies
// Demonstrates the solution to the "atoms crushing moving bodies" problem

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
        // Update moving bodies
        for body in &mut self.moving_bodies {
            body.update(dt);

            // Create particles around moving bodies to prevent crushing
            self.create_particles_around_body(body);
        }

        // Update atoms
        self.update_atoms(dt);

        // Handle particle interactions
        self.handle_particle_interactions(dt);

        // Clean up expired particles
        self.cleanup_particles();
    }

    fn update_atoms(&mut self, dt: f32) {
        for atom in &mut self.atoms {
            if atom.atom_type == AtomType::Empty {
                continue;
            }

            // Apply gravity to non-particle atoms
            if !atom.is_particle {
                atom.velocity.y -= 30.0 * dt;
            }

            // Update position
            atom.position += atom.velocity * dt;

            // Simple bounds checking
            if atom.position.x < 0.0 {
                atom.position.x = 0.0;
                atom.velocity.x *= -0.5;
            } else if atom.position.x >= self.width as f32 {
                atom.position.x = self.width as f32 - 1.0;
                atom.velocity.x *= -0.5;
            }

            if atom.position.y < 0.0 {
                atom.position.y = 0.0;
                atom.velocity.y *= -0.5;
            } else if atom.position.y >= self.height as f32 {
                atom.position.y = self.height as f32 - 1.0;
                atom.velocity.y *= -0.5;
            }

            // Update lifetime for particles
            if let Some(ref mut lifetime) = atom.lifetime {
                *lifetime -= dt;
            }
        }
    }

    fn create_particles_around_body(&mut self, body: &MovingBody) {
        let particle_distance = 2.0; // Distance from body to create particles
        let particle_lifetime = 0.5; // How long particles last
        let num_particles = 8; // Number of particles around the body

        for i in 0..num_particles {
            let angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU;
            let particle_pos = body.position + Vec2::new(angle.cos(), angle.sin()) * particle_distance;

            let x = particle_pos.x.round() as usize;
            let y = particle_pos.y.round() as usize;

            if x < self.width && y < self.height && self.is_empty(x, y) {
                // Create a particle atom
                let particle = ParticleAtom::new_particle(
                    AtomType::Sand, // Use sand as particle material
                    particle_pos,
                    body.velocity * 0.1, // Particles inherit some body velocity
                    particle_lifetime,
                );
                self.set_atom(x, y, particle);
            }
        }
    }

    fn handle_particle_interactions(&mut self, dt: f32) {
        // Particles push regular atoms away from moving bodies
        for body in &self.moving_bodies {
            let body_aabb = Rect::from_center_size(body.position, Vec2::new(4.0, 4.0));

            for atom in &mut self.atoms {
                if atom.atom_type != AtomType::Empty && !atom.is_particle {
                    if body_aabb.contains(atom.position) {
                        // Calculate repulsion force from body
                        let to_atom = atom.position - body.position;
                        let distance = to_atom.length();

                        if distance > 0.0 && distance < 3.0 {
                            let force_strength = (3.0 - distance) * 50.0;
                            let force_direction = to_atom.normalize();
                            atom.velocity += force_direction * force_strength * dt;
                        }
                    }
                }
            }
        }
    }

    fn cleanup_particles(&mut self) {
        for atom in &mut self.atoms {
            if atom.is_particle {
                if let Some(lifetime) = atom.lifetime {
                    if lifetime <= 0.0 {
                        *atom = ParticleAtom::new(AtomType::Empty, atom.position);
                    }
                }
            }
        }
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
                resolution: (1000.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(ParticleWorldResource(ParticleWorld::new(100, 80)))
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
        for y in 0..3 {
            world.set_atom(x, y, ParticleAtom::new(AtomType::Stone, Vec2::new(x as f32, y as f32)));
        }
    }

    // Create sand pile that will fall on moving bodies
    for x in 30..70 {
        for y in 20..40 {
            if rand::random::<f32>() < 0.8 {
                world.set_atom(x, y, ParticleAtom::new(AtomType::Sand, Vec2::new(x as f32, y as f32)));
            }
        }
    }

    // Create water above
    for x in 40..60 {
        for y in 50..60 {
            if rand::random::<f32>() < 0.7 {
                world.set_atom(x, y, ParticleAtom::new(AtomType::Water, Vec2::new(x as f32, y as f32)));
            }
        }
    }

    // Add moving bodies
    world.add_moving_body(MovingBody::new(
        Vec2::new(50.0, 10.0),
        Vec2::new(20.0, 0.0),
        Vec2::new(4.0, 4.0),
        5.0,
    ));

    world.add_moving_body(MovingBody::new(
        Vec2::new(30.0, 15.0),
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
            let color = atom.atom_type.color().with_a(alpha);

            let entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    atom.position.x - world.0.width as f32 / 2.0,
                    atom.position.y - world.0.height as f32 / 2.0,
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

    // Render moving bodies
    for (i, body) in world.0.moving_bodies.iter().enumerate() {
        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.3, 0.3),
                    custom_size: Some(body.size),
                    ..default()
                },
                transform: Transform::from_xyz(
                    body.position.x - world.0.width as f32 / 2.0,
                    body.position.y - world.0.height as f32 / 2.0,
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
                if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    let atom_x = (world_pos.origin.x + world.0.width as f32 / 2.0) as usize;
                    let atom_y = (world_pos.origin.y + world.0.height as f32 / 2.0) as usize;

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
        // Add another moving body
        world.0.add_moving_body(MovingBody::new(
            Vec2::new(80.0, 30.0),
            Vec2::new(-25.0, 10.0),
            Vec2::new(3.0, 3.0),
            2.0,
        ));
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset simulation
        *world = ParticleWorldResource(ParticleWorld::new(100, 80));
        create_particle_demo_setup(&mut world.0);
    }

    if keyboard_input.just_pressed(KeyCode::KeyP) {
        // Toggle particle visibility info
        println!("Particle system active - particles prevent atoms from crushing moving bodies");
        println!("Moving bodies create particle barriers around themselves");
    }
}
