// Example: Playing Nice with Moving Bodies
// Based on "Playing Nice with Moving Bodies" blog post
// https://www.slowrush.dev/news/playing-nice-with-moving-bodies

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Moving bodies that properly interact with atoms (water, sand, etc.)
// Demonstrates displacement, buoyancy, and realistic physics interactions

#[derive(Clone, Debug)]
enum AtomType {
    Empty,
    Sand,
    Water,
    Stone,
}

impl AtomType {
    fn density(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 1.6,
            AtomType::Water => 1.0,
            AtomType::Stone => 2.5,
        }
    }

    fn viscosity(&self) -> f32 {
        match self {
            AtomType::Empty => 0.0,
            AtomType::Sand => 0.9, // High viscosity, resists flow
            AtomType::Water => 0.1, // Low viscosity, flows easily
            AtomType::Stone => 1.0, // Very high viscosity, almost solid
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

    fn is_fluid(&self) -> bool {
        matches!(self, AtomType::Water)
    }

    fn is_granular(&self) -> bool {
        matches!(self, AtomType::Sand)
    }
}

#[derive(Clone, Debug)]
struct InteractiveAtom {
    atom_type: AtomType,
    position: Vec2,
    velocity: Vec2,
    submerged: bool, // Whether atom is displaced by moving body
    displaced_by: Option<usize>, // Index of body displacing this atom
}

impl InteractiveAtom {
    fn new(atom_type: AtomType, position: Vec2) -> Self {
        Self {
            atom_type,
            position,
            velocity: Vec2::ZERO,
            submerged: false,
            displaced_by: None,
        }
    }
}

#[derive(Clone, Debug)]
struct InteractiveBody {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
    mass: f32,
    buoyancy_force: Vec2,
    drag_force: Vec2,
    submerged_volume: f32,
    is_floating: bool,
}

impl InteractiveBody {
    fn new(position: Vec2, size: Vec2, mass: f32) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            size,
            mass,
            buoyancy_force: Vec2::ZERO,
            drag_force: Vec2::ZERO,
            submerged_volume: 0.0,
            is_floating: false,
        }
    }

    fn update(&mut self, dt: f32, world: &mut InteractiveWorld) {
        // Reset forces
        self.buoyancy_force = Vec2::ZERO;
        self.drag_force = Vec2::ZERO;
        self.submerged_volume = 0.0;

        // Calculate interaction with atoms
        self.calculate_atom_interactions(world);

        // Apply gravity
        let gravity = Vec2::new(0.0, -30.0 * self.mass);

        // Net force = gravity + buoyancy + drag
        let net_force = gravity + self.buoyancy_force + self.drag_force;

        // F = ma => a = F/m
        let acceleration = net_force / self.mass;

        // Update velocity and position
        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;

        // Apply damping
        self.velocity *= 0.99;

        // Update floating state
        self.is_floating = self.submerged_volume > self.size.x * self.size.y * 0.3;
    }

    fn calculate_atom_interactions(&mut self, world: &mut InteractiveWorld) {
        let body_rect = Rect::from_center_size(self.position, self.size);
        let mut atoms_in_body = Vec::new();

        // Find atoms intersecting with body
        for y in 0..world.height {
            for x in 0..world.width {
                if let Some(atom) = world.get_atom(x, y) {
                    if atom.atom_type != AtomType::Empty {
                        let atom_rect = Rect::from_center_size(
                            atom.position,
                            Vec2::new(1.0, 1.0)
                        );

                        if body_rect.intersect(atom_rect).is_some() {
                            atoms_in_body.push((x, y, atom.clone()));
                        }
                    }
                }
            }
        }

        // Calculate buoyancy and drag forces
        for (x, y, atom) in atoms_in_body {
            match atom.atom_type {
                AtomType::Water => {
                    // Buoyancy: F = density * volume * g
                    let volume = 1.0; // Each atom represents 1 unit volume
                    let buoyancy = Vec2::new(0.0, atom.atom_type.density() * volume * 30.0);
                    self.buoyancy_force += buoyancy;
                    self.submerged_volume += volume;

                    // Drag force opposes movement through fluid
                    let relative_velocity = self.velocity - atom.velocity;
                    let drag_magnitude = relative_velocity.length() * relative_velocity.length() * atom.atom_type.viscosity();
                    self.drag_force -= relative_velocity.normalize_or(Vec2::ZERO) * drag_magnitude;

                    // Displace water atoms
                    world.displace_atom(x, y, self.position, self.velocity);
                }
                AtomType::Sand => {
                    // Sand creates resistance but less buoyancy
                    let resistance = self.velocity * -0.5;
                    self.drag_force += resistance;

                    // Sand can be pushed around
                    if self.velocity.length_squared() > 10.0 {
                        world.displace_atom(x, y, self.position, self.velocity * 0.1);
                    }
                }
                AtomType::Stone => {
                    // Stone is solid - body bounces off
                    if let Some(intersection) = body_rect.intersect(
                        Rect::from_center_size(atom.position, Vec2::new(1.0, 1.0))
                    ) {
                        // Simple collision response
                        let normal = (self.position - atom.position).normalize_or(Vec2::Y);
                        self.velocity = self.velocity.reflect(normal) * 0.8;
                        self.position += normal * 0.1; // Push away from stone
                    }
                }
                _ => {}
            }
        }
    }

    fn get_bounds(&self) -> Rect {
        Rect::from_center_size(self.position, self.size)
    }
}

struct InteractiveWorld {
    width: usize,
    height: usize,
    atoms: Vec<InteractiveAtom>,
    bodies: Vec<InteractiveBody>,
    displacement_particles: Vec<DisplacementParticle>,
}

#[derive(Clone, Debug)]
struct DisplacementParticle {
    position: Vec2,
    velocity: Vec2,
    lifetime: f32,
    atom_type: AtomType,
}

impl DisplacementParticle {
    fn new(position: Vec2, velocity: Vec2, atom_type: AtomType) -> Self {
        Self {
            position,
            velocity,
            lifetime: 0.5, // 0.5 seconds
            atom_type,
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        self.position += self.velocity * dt;
        self.velocity.y -= 30.0 * dt; // Gravity
        self.lifetime -= dt;
        self.lifetime > 0.0
    }
}

impl InteractiveWorld {
    fn new(width: usize, height: usize) -> Self {
        let mut atoms = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                atoms.push(InteractiveAtom::new(AtomType::Empty, Vec2::new(x as f32, y as f32)));
            }
        }

        Self {
            width,
            height,
            atoms,
            bodies: Vec::new(),
            displacement_particles: Vec::new(),
        }
    }

    fn get_atom(&self, x: usize, y: usize) -> Option<&InteractiveAtom> {
        if x < self.width && y < self.height {
            Some(&self.atoms[y * self.width + x])
        } else {
            None
        }
    }

    fn get_atom_mut(&mut self, x: usize, y: usize) -> Option<&mut InteractiveAtom> {
        if x < self.width && y < self.height {
            Some(&mut self.atoms[y * self.width + x])
        } else {
            None
        }
    }

    fn set_atom(&mut self, x: usize, y: usize, atom: InteractiveAtom) {
        if x < self.width && y < self.height {
            self.atoms[y * self.width + x] = atom;
        }
    }

    fn displace_atom(&mut self, x: usize, y: usize, body_pos: Vec2, body_velocity: Vec2) {
        if let Some(atom) = self.get_atom_mut(x, y) {
            if atom.atom_type != AtomType::Empty {
                // Calculate displacement direction away from body
                let to_body = body_pos - atom.position;
                let distance = to_body.length();

                if distance > 0.0 {
                    let displacement_dir = -to_body.normalize();
                    let displacement_force = displacement_dir * (1.0 / (distance + 0.1)) * 20.0;

                    atom.velocity += displacement_force + body_velocity * 0.3;

                    // Mark as displaced
                    atom.submerged = true;

                    // Create displacement particle
                    let particle = DisplacementParticle::new(
                        atom.position,
                        atom.velocity * 0.5,
                        atom.atom_type.clone(),
                    );
                    self.displacement_particles.push(particle);
                }
            }
        }
    }

    fn update(&mut self, dt: f32) {
        // Update bodies
        for body in &mut self.bodies {
            body.update(dt, self);
        }

        // Update atoms
        self.update_atoms(dt);

        // Update displacement particles
        self.displacement_particles.retain_mut(|p| p.update(dt));

        // Clean up submerged atoms
        self.cleanup_displaced_atoms();
    }

    fn update_atoms(&mut self, dt: f32) {
        for atom in &mut self.atoms {
            if atom.atom_type == AtomType::Empty {
                continue;
            }

            // Apply gravity to non-submerged atoms
            if !atom.submerged {
                atom.velocity.y -= 30.0 * dt;
            }

            // Update position
            atom.position += atom.velocity * dt;

            // Apply friction/damping
            atom.velocity *= 0.98;

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
        }
    }

    fn cleanup_displaced_atoms(&mut self) {
        for atom in &mut self.atoms {
            if atom.submerged {
                // Check if atom is still being displaced
                let still_displaced = self.bodies.iter().any(|body| {
                    let body_bounds = body.get_bounds();
                    body_bounds.contains(atom.position)
                });

                if !still_displaced {
                    atom.submerged = false;
                    atom.displaced_by = None;
                }
            }
        }
    }

    fn add_body(&mut self, body: InteractiveBody) {
        self.bodies.push(body);
    }
}

#[derive(Resource)]
struct InteractiveWorldResource(InteractiveWorld);

#[derive(Component)]
struct BodyEntity {
    body_index: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Playing Nice with Moving Bodies - Realistic Interactions".to_string(),
                resolution: (1200.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(InteractiveWorldResource(InteractiveWorld::new(120, 90)))
        .add_systems(Startup, setup_interactive_demo)
        .add_systems(Update, (
            update_interactive_world,
            render_interactive_atoms,
            render_interactive_bodies,
            render_displacement_particles,
            handle_interactive_input,
            demonstrate_interactions,
        ).chain())
        .run();
}

fn setup_interactive_demo(mut commands: Commands, mut world: ResMut<InteractiveWorldResource>) {
    commands.spawn(Camera2dBundle::default());

    // Create the interactive demo setup
    create_interactive_setup(&mut world.0);
}

fn create_interactive_setup(world: &mut InteractiveWorld) {
    // Create stone floor
    for x in 0..world.width {
        for y in 0..5 {
            world.set_atom(x, y, InteractiveAtom::new(AtomType::Stone, Vec2::new(x as f32, y as f32)));
        }
    }

    // Create water pool
    for x in 20..60 {
        for y in 10..30 {
            if rand::random::<f32>() < 0.9 {
                world.set_atom(x, y, InteractiveAtom::new(AtomType::Water, Vec2::new(x as f32, y as f32)));
            }
        }
    }

    // Create sand area
    for x in 70..100 {
        for y in 15..35 {
            if rand::random::<f32>() < 0.8 {
                world.set_atom(x, y, InteractiveAtom::new(AtomType::Sand, Vec2::new(x as f32, y as f32)));
            }
        }
    }

    // Add interactive bodies
    world.add_body(InteractiveBody::new(
        Vec2::new(30.0, 50.0),
        Vec2::new(6.0, 6.0),
        8.0,
    ));

    world.add_body(InteractiveBody::new(
        Vec2::new(80.0, 55.0),
        Vec2::new(4.0, 4.0),
        4.0,
    ));
}

fn update_interactive_world(
    mut world: ResMut<InteractiveWorldResource>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds().min(1.0 / 30.0); // Cap dt for stability
    world.0.update(dt);
}

fn render_interactive_atoms(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    world: Res<InteractiveWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render atoms
    for atom in &world.0.atoms {
        if atom.atom_type != AtomType::Empty {
            let alpha = if atom.submerged { 0.7 } else { 1.0 };
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
                    0.0,
                ),
                ..default()
            }).id();
            atom_entities.push(entity);
        }
    }
}

fn render_interactive_bodies(
    mut commands: Commands,
    mut body_entities: Local<Vec<Entity>>,
    world: Res<InteractiveWorldResource>,
) {
    // Clear previous frame
    for entity in body_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render bodies
    for (i, body) in world.0.bodies.iter().enumerate() {
        let color = if body.is_floating {
            Color::rgb(0.3, 0.7, 0.9) // Blue when floating
        } else {
            Color::rgb(0.8, 0.4, 0.4) // Red when not floating
        };

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(body.size),
                    ..default()
                },
                transform: Transform::from_xyz(
                    body.position.x - world.0.width as f32 / 2.0,
                    body.position.y - world.0.height as f32 / 2.0,
                    1.0,
                ),
                ..default()
            },
            BodyEntity { body_index: i },
        )).id();
        body_entities.push(entity);
    }
}

fn render_displacement_particles(
    mut commands: Commands,
    mut particle_entities: Local<Vec<Entity>>,
    world: Res<InteractiveWorldResource>,
) {
    // Clear previous frame
    for entity in particle_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render displacement particles
    for particle in &world.0.displacement_particles {
        let alpha = (particle.lifetime / 0.5).max(0.0);
        let color = particle.atom_type.color().with_a(alpha * 0.6);

        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(0.8, 0.8)),
                ..default()
            },
            transform: Transform::from_xyz(
                particle.position.x - world.0.width as f32 / 2.0,
                particle.position.y - world.0.height as f32 / 2.0,
                2.0,
            ),
            ..default()
        }).id();
        particle_entities.push(entity);
    }
}

fn handle_interactive_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<InteractiveWorldResource>,
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    let atom_x = (world_pos.origin.x + world.0.width as f32 / 2.0) as usize;
                    let atom_y = (world_pos.origin.y + world.0.height as f32 / 2.0) as usize;

                    if mouse_input.just_pressed(MouseButton::Left) {
                        // Add water at cursor
                        if atom_x < world.0.width && atom_y < world.0.height {
                            world.0.set_atom(atom_x, atom_y, InteractiveAtom::new(
                                AtomType::Water,
                                Vec2::new(atom_x as f32, atom_y as f32)
                            ));
                        }
                    }

                    if mouse_input.just_pressed(MouseButton::Right) {
                        // Add sand at cursor
                        if atom_x < world.0.width && atom_y < world.0.height {
                            world.0.set_atom(atom_x, atom_y, InteractiveAtom::new(
                                AtomType::Sand,
                                Vec2::new(atom_x as f32, atom_y as f32)
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn demonstrate_interactions(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<InteractiveWorldResource>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        // Add another body
        world.0.add_body(InteractiveBody::new(
            Vec2::new(50.0, 70.0),
            Vec2::new(5.0, 5.0),
            6.0,
        ));
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset simulation
        *world = InteractiveWorldResource(InteractiveWorld::new(120, 90));
        create_interactive_setup(&mut world.0);
    }

    if keyboard_input.just_pressed(KeyCode::KeyI) {
        // Display interaction info
        println!("Interactive Body Physics Demo:");
        println!("- Bodies experience buoyancy in water");
        println!("- Drag forces resist movement through fluids");
        println!("- Atoms are displaced by moving bodies");
        println!("- Blue bodies are floating, red are sinking");
        println!("- Displacement particles show atom movement");
    }
}
