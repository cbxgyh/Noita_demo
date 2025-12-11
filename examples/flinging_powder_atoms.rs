// Example: Flinging Powder Atoms
// Based on "Flinging Powder Atoms" blog post
// https://www.slowrush.dev/news/flinging-powder-atoms

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Powder atom physics and flinging mechanics
// Demonstrates how to throw and manipulate granular materials

#[derive(Clone, Debug)]
enum PowderType {
    Sand,
    Gunpowder,
    ColoredPowder,
}

impl PowderType {
    fn color(&self) -> Color {
        match self {
            PowderType::Sand => Color::rgb(0.8, 0.7, 0.5),
            PowderType::Gunpowder => Color::rgb(0.3, 0.3, 0.3),
            PowderType::ColoredPowder => Color::hsl(rand::random::<f32>() * 360.0, 1.0, 0.6),
        }
    }

    fn density(&self) -> f32 {
        match self {
            PowderType::Sand => 1.6,
            PowderType::Gunpowder => 1.0,
            PowderType::ColoredPowder => 1.2,
        }
    }

    fn flammability(&self) -> f32 {
        match self {
            PowderType::Sand => 0.0,
            PowderType::Gunpowder => 1.0, // Highly flammable
            PowderType::ColoredPowder => 0.2,
        }
    }

    fn explosion_chance(&self) -> f32 {
        match self {
            PowderType::Sand => 0.0,
            PowderType::Gunpowder => 0.8, // High chance to explode when ignited
            PowderType::ColoredPowder => 0.1,
        }
    }
}

#[derive(Clone, Debug)]
struct PowderAtom {
    powder_type: PowderType,
    position: Vec2,
    velocity: Vec2,
    size: f32,
    temperature: f32,
    lifetime: Option<f32>,
    is_ignited: bool,
}

impl PowderAtom {
    fn new(powder_type: PowderType, position: Vec2) -> Self {
        Self {
            powder_type: powder_type.clone(),
            position,
            velocity: Vec2::ZERO,
            size: 2.0,
            temperature: 20.0,
            lifetime: None,
            is_ignited: false,
        }
    }

    fn update(&mut self, dt: f32, world_bounds: Vec2) -> bool {
        // Apply gravity
        self.velocity.y -= 30.0 * dt;

        // Apply air resistance
        self.velocity *= 0.99;

        // Update position
        self.position += self.velocity * dt;

        // Bounds checking
        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.x *= -0.5;
        } else if self.position.x >= world_bounds.x {
            self.position.x = world_bounds.x - 1.0;
            self.velocity.x *= -0.5;
        }

        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y *= -0.5;
        } else if self.position.y >= world_bounds.y {
            self.position.y = world_bounds.y - 1.0;
            self.velocity.y *= -0.5;
        }

        // Handle ignition
        if self.is_ignited {
            self.temperature += 100.0 * dt;
            if self.temperature > 200.0 {
                // Burn out
                self.is_ignited = false;
                self.temperature = 20.0;
            }
        }

        // Update lifetime
        if let Some(ref mut lifetime) = self.lifetime {
            *lifetime -= dt;
            if *lifetime <= 0.0 {
                return false;
            }
        }

        true
    }

    fn apply_force(&mut self, force: Vec2) {
        self.velocity += force / self.powder_type.density();
    }

    fn ignite(&mut self) {
        if self.powder_type.flammability() > 0.0 {
            self.is_ignited = true;
            self.temperature = 150.0;
        }
    }

    fn can_ignite_neighbor(&self, other: &PowderAtom) -> bool {
        self.is_ignited && other.powder_type.flammability() > 0.0 &&
        self.position.distance(other.position) < 5.0
    }
}

#[derive(Clone, Debug)]
struct PowderGrenade {
    position: Vec2,
    velocity: Vec2,
    powder_type: PowderType,
    powder_count: usize,
    fuse_time: f32,
    exploded: bool,
}

impl PowderGrenade {
    fn new(position: Vec2, velocity: Vec2, powder_type: PowderType, powder_count: usize) -> Self {
        Self {
            position,
            velocity,
            powder_type,
            powder_count,
            fuse_time: 2.0, // 2 second fuse
            exploded: false,
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        if self.exploded {
            return false;
        }

        // Apply gravity
        self.velocity.y -= 30.0 * dt;

        // Update position
        self.position += self.velocity * dt;

        // Update fuse
        self.fuse_time -= dt;

        // Check for ground collision (simple)
        if self.position.y <= 10.0 {
            self.position.y = 10.0;
            self.velocity.y *= -0.3; // Bounce
            self.velocity.x *= 0.8; // Friction
        }

        self.fuse_time > 0.0
    }

    fn should_explode(&self) -> bool {
        self.fuse_time <= 0.0
    }

    fn explode(&mut self) -> Vec<PowderAtom> {
        self.exploded = true;

        let mut powder_atoms = Vec::new();

        for _ in 0..self.powder_count {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            let speed = 50.0 + rand::random::<f32>() * 100.0;
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            let offset = Vec2::new(
                (rand::random::<f32>() - 0.5) * 20.0,
                (rand::random::<f32>() - 0.5) * 20.0,
            );

            let mut atom = PowderAtom::new(self.powder_type.clone(), self.position + offset);
            atom.velocity = velocity;
            atom.lifetime = Some(10.0); // Powder lasts 10 seconds

            powder_atoms.push(atom);
        }

        powder_atoms
    }
}

#[derive(Clone, Debug)]
struct Player {
    position: Vec2,
    selected_powder: PowderType,
    grenades: Vec<PowderGrenade>,
}

impl Player {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            selected_powder: PowderType::Sand,
            grenades: Vec::new(),
        }
    }

    fn throw_grenade(&mut self, target: Vec2) {
        let direction = (target - self.position).normalize();
        let speed = 150.0;
        let velocity = direction * speed;

        let grenade = PowderGrenade::new(
            self.position,
            velocity,
            self.selected_powder.clone(),
            50, // 50 powder atoms per grenade
        );

        self.grenades.push(grenade);
    }
}

struct PowderWorld {
    atoms: Vec<PowderAtom>,
    player: Player,
    bounds: Vec2,
}

impl PowderWorld {
    fn new(width: f32, height: f32) -> Self {
        let player = Player::new(Vec2::new(width / 2.0, height - 50.0));

        Self {
            atoms: Vec::new(),
            player,
            bounds: Vec2::new(width, height),
        }
    }

    fn update(&mut self, dt: f32) {
        // Update powder atoms
        self.atoms.retain_mut(|atom| atom.update(dt, self.bounds));

        // Update grenades
        let mut new_atoms = Vec::new();
        self.player.grenades.retain_mut(|grenade| {
            let keep = grenade.update(dt);
            if grenade.should_explode() {
                new_atoms.extend(grenade.explode());
            }
            keep
        });

        // Add exploded powder
        self.atoms.extend(new_atoms);

        // Handle powder interactions
        self.update_powder_physics(dt);

        // Handle ignition spread
        self.spread_ignition();
    }

    fn update_powder_physics(&mut self, dt: f32) {
        // Simple powder physics - atoms settle and stack
        for i in 0..self.atoms.len() {
            if self.atoms[i].velocity.length_squared() < 1.0 {
                // Try to settle downward
                let below_pos = self.atoms[i].position + Vec2::new(0.0, -self.atoms[i].size);
                let can_settle = !self.atoms.iter().any(|other|
                    other.position.distance(below_pos) < self.atoms[i].size && other.powder_type.density() > 0.0
                );

                if can_settle && below_pos.y > 0.0 {
                    self.atoms[i].position = below_pos;
                }
            }
        }
    }

    fn spread_ignition(&mut self) {
        // Spread fire between neighboring powder atoms
        for i in 0..self.atoms.len() {
            if self.atoms[i].is_ignited {
                for j in 0..self.atoms.len() {
                    if i != j && self.atoms[i].can_ignite_neighbor(&self.atoms[j]) {
                        self.atoms[j].ignite();
                    }
                }
            }
        }
    }

    fn apply_force_in_area(&mut self, center: Vec2, radius: f32, force: Vec2) {
        for atom in &mut self.atoms {
            let distance = atom.position.distance(center);
            if distance <= radius {
                let falloff = 1.0 - (distance / radius);
                atom.apply_force(force * falloff);
            }
        }
    }

    fn ignite_area(&mut self, center: Vec2, radius: f32) {
        for atom in &mut self.atoms {
            if atom.position.distance(center) <= radius {
                atom.ignite();
            }
        }
    }
}

#[derive(Resource)]
struct PowderWorldResource(PowderWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flinging Powder Atoms - Grenade Physics".to_string(),
                resolution: (1000.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(PowderWorldResource(PowderWorld::new(1000.0, 800.0)))
        .add_systems(Startup, setup_powder_demo)
        .add_systems(Update, (
            update_powder_world,
            render_powder_world,
            handle_powder_input,
            demonstrate_powder_physics,
        ).chain())
        .run();
}

fn setup_powder_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Create ground
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::new(1000.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(500.0, 10.0),
    ));
}

fn update_powder_world(mut world: ResMut<PowderWorldResource>, time: Res<Time>) {
    let dt = time.delta_seconds().min(1.0 / 30.0);
    world.0.update(dt);
}

fn render_powder_world(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    mut grenade_entities: Local<Vec<Entity>>,
    mut player_entity: Local<Option<Entity>>,
    world: Res<PowderWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in grenade_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }

    // Render powder atoms
    for atom in &world.0.atoms {
        let mut color = atom.powder_type.color();

        // Brighten if ignited
        if atom.is_ignited {
            color = Color::rgb(1.0, 0.5, 0.0);
        }

        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(atom.size, atom.size)),
                ..default()
            },
            transform: Transform::from_xyz(atom.position.x, atom.position.y, 1.0),
            ..default()
        }).id();
        atom_entities.push(entity);
    }

    // Render grenades
    for grenade in &world.0.player.grenades {
        if !grenade.exploded {
            let color = grenade.powder_type.color();
            let entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(8.0, 8.0)),
                    ..default()
                },
                transform: Transform::from_xyz(grenade.position.x, grenade.position.y, 1.0),
                ..default()
            }).id();
            grenade_entities.push(entity);
        }
    }

    // Render player
    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(20.0, 30.0)),
                ..default()
            },
            transform: Transform::from_xyz(world.0.player.position.x, world.0.player.position.y, 2.0),
            ..default()
        },

    )).id();
    *player_entity = Some(entity);
}

fn handle_powder_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<PowderWorldResource>,
) {
    // Powder type selection
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        world.0.player.selected_powder = PowderType::Sand;
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        world.0.player.selected_powder = PowderType::Gunpowder;
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        world.0.player.selected_powder = PowderType::ColoredPowder;
    }

    // Grenade throwing
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        world.0.player.throw_grenade(world_pos.origin.truncate());
                    }
                }
            }
        }
    }

    // Area effects
    if mouse_input.just_pressed(MouseButton::Right) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        // Apply force to blow powder around
                        world.0.apply_force_in_area(world_pos.origin.truncate(), 50.0, Vec2::new(0.0, 100.0));
                    }
                }
            }
        }
    }

    // Ignition
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        world.0.ignite_area(world_pos.origin.truncate(), 30.0);
                    }
                }
            }
        }
    }

    // Reset
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        *world = PowderWorldResource(PowderWorld::new(1000.0, 800.0));
    }
}

fn demonstrate_powder_physics(keyboard_input: Res<ButtonInput<KeyCode>>, world: Res<PowderWorldResource>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Flinging Powder Atoms Demo ===");
        println!("Selected Powder: {:?}", world.0.player.selected_powder);
        println!("Active Atoms: {}", world.0.atoms.len());
        println!("Active Grenades: {}", world.0.player.grenades.len());
        println!("");
        println!("Powder Types:");
        println!("1: Sand (safe, stacks well)");
        println!("2: Gunpowder (explosive when lit)");
        println!("3: Colored Powder (colorful, mildly flammable)");
        println!("");
        println!("Grenade Physics:");
        println!("- Arcs through air with gravity");
        println!("- Bounces on ground with friction");
        println!("- Explodes after 2 second fuse");
        println!("- Releases powder atoms in all directions");
        println!("");
        println!("Controls:");
        println!("1-3: Select powder type");
        println!("Left click: Throw grenade");
        println!("Right click: Blow powder around");
        println!("F: Ignite powder (gunpowder explodes!)");
        println!("R: Reset simulation");
        println!("H: Show this info");
        println!("===========================\n");
    }
}
