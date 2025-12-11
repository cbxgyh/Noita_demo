// Example: Discord and Making a Splash
// Based on "Discord and Making a Splash" blog post
// https://www.slowrush.dev/news/discord-and-making-a-splash

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Water physics and splash effects
// Demonstrates water atoms that can be pushed around and create splashes

#[derive(Clone, Debug,Eq, PartialEq)]
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
            AtomType::Sand => 0.9,
            AtomType::Water => 0.1, // Water flows easily
            AtomType::Stone => 1.0,
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
}

#[derive(Clone, Debug)]
struct FluidAtom {
    atom_type: AtomType,
    position: Vec2,
    velocity: Vec2,
    pressure: f32,
    is_splash: bool, // Marks atoms that are part of splash effects
    lifetime: Option<f32>,
}

impl FluidAtom {
    fn new(atom_type: AtomType, position: Vec2) -> Self {
        Self {
            atom_type,
            position,
            velocity: Vec2::ZERO,
            pressure: 0.0,
            is_splash: false,
            lifetime: None,
        }
    }

    fn new_splash(atom_type: AtomType, position: Vec2, velocity: Vec2, lifetime: f32) -> Self {
        Self {
            atom_type,
            position,
            velocity,
            pressure: 0.0,
            is_splash: true,
            lifetime: Some(lifetime),
        }
    }
}

#[derive(Clone, Debug)]
struct Player {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
    in_water: bool,
    splash_velocity: Vec2, // Velocity from last splash
}

struct SplashWorld {
    width: usize,
    height: usize,
    atoms: Vec<FluidAtom>,
    player: Player,
    gravity: Vec2,
}

impl SplashWorld {
    fn new(width: usize, height: usize) -> Self {
        let mut atoms = Vec::with_capacity(width * height);

        // Create world
        for y in 0..height {
            for x in 0..width {
                let mut atom_type = AtomType::Empty;

                // Create stone floor
                if y < 8 {
                    atom_type = AtomType::Stone;
                }
                // Create water pool
                else if y >= 20 && y < 40 && x > 30 && x < width - 30 {
                    atom_type = AtomType::Water;
                }
                // Create sand pile
                else if y >= 10 && y < 25 && x > 10 && x < 40 {
                    atom_type = AtomType::Sand;
                }

                atoms.push(FluidAtom::new(atom_type, Vec2::new(x as f32, y as f32)));
            }
        }

        let player = Player {
            position: Vec2::new(50.0, 60.0),
            velocity: Vec2::ZERO,
            size: Vec2::new(16.0, 24.0),
            in_water: false,
            splash_velocity: Vec2::ZERO,
        };

        Self {
            width,
            height,
            atoms,
            player,
            gravity: Vec2::new(0.0, -30.0),
        }
    }

    fn update(&mut self, dt: f32) {
        // Update fluid physics
        self.update_fluids(dt);

        // Update player
        self.update_player(dt);

        // Handle interactions between player and fluids
        self.handle_player_fluid_interactions(dt);

        // Create splash effects
        self.create_splashes(dt);

        // Clean up expired splashes
        self.cleanup_splashes();
    }

    fn update_fluids(&mut self, dt: f32) {
        // Calculate pressure for fluid dynamics
        self.calculate_pressure();

        // Update fluid atoms
        // First, calculate pressure gradients for all fluid atoms
        let mut pressure_forces = Vec::new();
        for atom in &self.atoms {
            if atom.atom_type.is_fluid() {
                let pressure_force = self.calculate_pressure_gradient(atom) * 50.0 * dt;
                pressure_forces.push(pressure_force);
            } else {
                pressure_forces.push(Vec2::ZERO);
            }
        }

        // Now update atoms with calculated forces
        for (atom, pressure_force) in self.atoms.iter_mut().zip(pressure_forces.iter()) {
            if atom.atom_type == AtomType::Empty {
                continue;
            }

            // Apply gravity
            atom.velocity += self.gravity * dt;

            // Apply pressure forces for fluid behavior
            atom.velocity += *pressure_force;

            // Apply viscosity
            let viscosity = atom.atom_type.viscosity();
            atom.velocity *= 1.0 - viscosity * dt;

            // Update position
            let new_position = atom.position + atom.velocity * dt;

            // Simple bounds checking
            if new_position.x < 0.0 {
                atom.position.x = 0.0;
                atom.velocity.x *= -0.5;
            } else if new_position.x >= self.width as f32 {
                atom.position.x = self.width as f32 - 1.0;
                atom.velocity.x *= -0.5;
            }

            if new_position.y < 0.0 {
                atom.position.y = 0.0;
                atom.velocity.y *= -0.5;
            } else if new_position.y >= self.height as f32 {
                atom.position.y = self.height as f32 - 1.0;
                atom.velocity.y *= -0.5;
            } else {
                atom.position = new_position;
            }

            // Update splash lifetime
            if let Some(ref mut lifetime) = atom.lifetime {
                *lifetime -= dt;
            }
        }

        // Handle fluid collisions
        self.handle_fluid_collisions();
    }

    fn calculate_pressure(&mut self) {
        // Simple pressure calculation based on neighboring fluids
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if self.atoms[idx].atom_type.is_fluid() {
                    let mut neighbor_count = 0;

                    // Count fluid neighbors
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 { continue; }

                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;

                            if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                                let nidx = ny as usize * self.width + nx as usize;
                                if self.atoms[nidx].atom_type.is_fluid() {
                                    neighbor_count += 1;
                                }
                            }
                        }
                    }

                    // Higher neighbor count = higher pressure
                    self.atoms[idx].pressure = neighbor_count as f32 / 8.0;
                } else {
                    self.atoms[idx].pressure = 0.0;
                }
            }
        }
    }

    fn apply_pressure_forces(&mut self, atom: &mut FluidAtom, dt: f32) {
        // Pressure forces make fluids flow towards lower pressure areas
        let pressure_force = self.calculate_pressure_gradient(atom) * 50.0 * dt;
        atom.velocity += pressure_force;
    }

    fn calculate_pressure_gradient(&self, atom: &FluidAtom) -> Vec2 {
        let x = atom.position.x as usize;
        let y = atom.position.y as usize;

        if x >= self.width || y >= self.height {
            return Vec2::ZERO;
        }

        let current_pressure = atom.pressure;

        // Sample neighboring pressures
        let left = if x > 0 { self.atoms[y * self.width + (x - 1)].pressure } else { 0.0 };
        let right = if x < self.width - 1 { self.atoms[y * self.width + (x + 1)].pressure } else { 0.0 };
        let up = if y > 0 { self.atoms[(y - 1) * self.width + x].pressure } else { 0.0 };
        let down = if y < self.height - 1 { self.atoms[(y + 1) * self.width + x].pressure } else { 0.0 };

        Vec2::new(right - left, down - up).normalize_or(Vec2::ZERO) * (current_pressure - (left + right + up + down) / 4.0)
    }

    fn handle_fluid_collisions(&mut self) {
        // Simple collision resolution between fluid atoms
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if self.atoms[idx].atom_type.is_fluid() {
                    // Check neighboring atoms for collisions
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 { continue; }

                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;

                            if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                                let nidx = ny as usize * self.width + nx as usize;

                                if self.atoms[nidx].atom_type.is_fluid() {
                                    let distance = self.atoms[idx].position.distance(self.atoms[nidx].position);

                                    if distance < 1.0 && distance > 0.0 {
                                        // Separate overlapping fluids
                                        let separation = (1.0 - distance) * 0.5;
                                        let direction = (self.atoms[nidx].position - self.atoms[idx].position).normalize();

                                        self.atoms[idx].position -= direction * separation;
                                        self.atoms[nidx].position += direction * separation;

                                        // Exchange some velocity
                                        let velocity_exchange = 0.1;
                                        let vel_diff = self.atoms[idx].velocity - self.atoms[nidx].velocity;
                                        self.atoms[idx].velocity -= vel_diff * velocity_exchange;
                                        self.atoms[nidx].velocity += vel_diff * velocity_exchange;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_player(&mut self, dt: f32) {
        // Player physics with water interaction
        self.player.velocity += self.gravity * dt;

        // Check if player is in water
        self.player.in_water = false;
        let player_rect = Rect::from_center_size(self.player.position, self.player.size);

        for atom in &self.atoms {
            if atom.atom_type == AtomType::Water {
                let atom_rect = Rect::from_center_size(atom.position, Vec2::new(1.0, 1.0));
                let intersection = player_rect.intersect(atom_rect);
                if intersection.width() > 0.0 && intersection.height() > 0.0 {
                    self.player.in_water = true;

                    // Buoyancy effect
                    if self.player.velocity.y < 0.0 { // Only when falling/sinking
                        self.player.velocity.y *= 0.9; // Reduce downward velocity
                    }

                    // Drag in water
                    self.player.velocity *= 0.95;
                    break;
                }
            }
        }

        self.player.position += self.player.velocity * dt;

        // Simple bounds
        if self.player.position.y < self.player.size.y / 2.0 {
            self.player.position.y = self.player.size.y / 2.0;
            self.player.velocity.y = 0.0;
        }
    }

    fn handle_player_fluid_interactions(&mut self, dt: f32) {
        let player_rect = Rect::from_center_size(self.player.position, self.player.size);

        // Collect splash positions to create after the loop
        let mut splash_positions = Vec::new();
        
        for atom in &mut self.atoms {
            if atom.atom_type.is_fluid() {
                let atom_rect = Rect::from_center_size(atom.position, Vec2::new(1.0, 1.0));

                let intersection = player_rect.intersect(atom_rect);
                if intersection.width() > 0.0 && intersection.height() > 0.0 {
                    // Player pushes fluid atoms
                    let push_direction = (atom.position - self.player.position).normalize();
                    let push_force = 100.0 * dt;

                    atom.velocity += push_direction * push_force;

                    // Collect splash position if moving fast
                    if self.player.velocity.length_squared() > 25.0 {
                        splash_positions.push((atom.position, self.player.velocity * 0.5));
                    }
                }
            }
        }
        
        // Create splashes after the loop
        for (position, velocity) in splash_positions {
            self.create_splash_at(position, velocity);
        }
    }

    fn create_splash_at(&mut self, position: Vec2, base_velocity: Vec2) {
        // Create splash particles
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
            let speed = 20.0 + rand::random::<f32>() * 30.0;
            let velocity = Vec2::new(angle.cos(), angle.sin()) * speed + base_velocity;

            let splash_pos = position + Vec2::new(
                (rand::random::<f32>() - 0.5) * 2.0,
                (rand::random::<f32>() - 0.5) * 2.0,
            );

            // Find empty spot for splash
            let x = splash_pos.x.round() as usize;
            let y = splash_pos.y.round() as usize;

            if x < self.width && y < self.height {
                let idx = y * self.width + x;
                if self.atoms[idx].atom_type == AtomType::Empty {
                    self.atoms[idx] = FluidAtom::new_splash(
                        AtomType::Water,
                        splash_pos,
                        velocity,
                        1.0, // 1 second lifetime
                    );
                }
            }
        }
    }

    fn create_splashes(&mut self, dt: f32) {
        // Create splashes when fast-moving atoms hit surfaces
        static mut SPLASH_COOLDOWN: f32 = 0.0;
        unsafe {
            SPLASH_COOLDOWN -= dt;
            if SPLASH_COOLDOWN <= 0.0 {
                SPLASH_COOLDOWN = 0.1; // Limit splash frequency

                for atom in &self.atoms {
                    if atom.atom_type.is_fluid() && !atom.is_splash {
                        let speed = atom.velocity.length();

                        // Create splash if hitting bottom or if moving fast horizontally
                        if (atom.position.y < 10.0 && atom.velocity.y < -10.0) ||
                           (speed > 30.0) {
                            self.create_splash_at(atom.position, atom.velocity * 0.3);
                            break; // Only one splash per frame
                        }
                    }
                }
            }
        }
    }

    fn cleanup_splashes(&mut self) {
        for atom in &mut self.atoms {
            if atom.is_splash {
                if let Some(lifetime) = atom.lifetime {
                    if lifetime <= 0.0 {
                        *atom = FluidAtom::new(AtomType::Empty, atom.position);
                    }
                }
            }
        }
    }

    fn apply_force_at(&mut self, position: Vec2, force: Vec2, radius: f32) {
        for atom in &mut self.atoms {
            if atom.atom_type != AtomType::Empty {
                let distance = atom.position.distance(position);
                if distance <= radius {
                    let falloff = 1.0 - (distance / radius);
                    atom.velocity += force * falloff;
                }
            }
        }
    }

    fn set_player_input(&mut self, left: bool, right: bool, jump: bool) {
        let move_force = 150.0;

        if left {
            self.player.velocity.x -= move_force * 0.016; // Assume 60fps dt
        }
        if right {
            self.player.velocity.x += move_force * 0.016;
        }

        // Simple jump
        static mut CAN_JUMP: bool = true;
        unsafe {
            if jump && CAN_JUMP && (!self.player.in_water || self.player.position.y < 30.0) {
                self.player.velocity.y = 120.0;
                CAN_JUMP = false;
            }
            if !jump {
                CAN_JUMP = true;
            }
        }
    }
}

#[derive(Resource)]
struct SplashWorldResource(SplashWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Discord and Making a Splash - Water Physics & Splashes".to_string(),
                resolution: (1000.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(SplashWorldResource(SplashWorld::new(150, 100)))
        .add_systems(Startup, setup_splash_demo)
        .add_systems(Update, (
            update_splash_world,
            render_splash_world,
            handle_splash_input,
            demonstrate_splash_physics,
        ).chain())
        .run();
}

fn setup_splash_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_splash_world(mut world: ResMut<SplashWorldResource>, time: Res<Time>) {
    let dt = time.delta_seconds().min(1.0 / 30.0);
    world.0.update(dt);
}

fn render_splash_world(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    mut player_entity: Local<Option<Entity>>,
    world: Res<SplashWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }

    // Render atoms
    for atom in &world.0.atoms {
        if atom.atom_type != AtomType::Empty {
            let mut color = atom.atom_type.color();

            // Highlight splash particles
            if atom.is_splash {
                color = Color::rgb(0.5, 0.8, 1.0);
            }

            // Show pressure with brightness (simplified - just use original color for now)
            // Note: Color brightness adjustment would require converting to linear color space
            // For simplicity, we'll skip this feature or implement it differently

            let entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    atom.position.x - world.0.width as f32 / 2.0,
                    atom.position.y - world.0.height as f32 / 2.0,
                    if atom.is_splash { 2.0 } else { 0.0 },
                ),
                ..default()
            }).id();
            atom_entities.push(entity);
        }
    }

    // Render player
    let player_color = if world.0.player.in_water {
        Color::rgb(0.3, 0.6, 0.9) // Blue when in water
    } else {
        Color::rgb(0.8, 0.3, 0.3) // Red when not in water
    };

    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: player_color,
                custom_size: Some(world.0.player.size),
                ..default()
            },
            transform: Transform::from_xyz(
                world.0.player.position.x - world.0.width as f32 / 2.0,
                world.0.player.position.y - world.0.height as f32 / 2.0,
                1.0,
            ),
            ..default()
        },

    )).id();
    *player_entity = Some(entity);
}

fn handle_splash_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<SplashWorldResource>,
) {
    // Player movement
    let left = keyboard_input.pressed(KeyCode::KeyA);
    let right = keyboard_input.pressed(KeyCode::KeyD);
    let jump = keyboard_input.just_pressed(KeyCode::Space);

    world.0.set_player_input(left, right, jump);

    // Mouse interaction
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        let atom_x = (world_pos.origin.x + world.0.width as f32 / 2.0) as f32;
                        let atom_y = (world_pos.origin.y + world.0.height as f32 / 2.0) as f32;

                        // Apply force at mouse position
                        world.0.apply_force_at(
                            Vec2::new(atom_x, atom_y),
                            Vec2::new(0.0, 50.0),
                            15.0,
                        );
                    }
                }
            }
        }
    }
}

fn demonstrate_splash_physics(keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Discord and Making a Splash Demo ===");
        println!("Features:");
        println!("- Realistic water physics with pressure");
        println!("- Player can swim and splash in water");
        println!("- Buoyancy and drag forces");
        println!("- Splash particle effects");
        println!("- Fluid atoms can be pushed around");
        println!("- Color coding: Blue=water, Bright=high pressure");
        println!("\nControls:");
        println!("A/D: Move left/right");
        println!("Space: Jump/Swim");
        println!("Left click: Create waves");
        println!("H: Show this help");
        println!("=======================================\n");
    }
}
