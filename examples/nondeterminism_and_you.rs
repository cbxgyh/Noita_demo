// Example: Nondeterminism And You
// Based on "Nondeterminism And You" blog post
// https://www.slowrush.dev/news/nondeterminism-and-you

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Non-deterministic vs deterministic simulation
// Demonstrates the challenges of network synchronization and determinism

#[derive(Clone, Debug)]
struct DeterministicRandom {
    seed: u64,
}

impl DeterministicRandom {
    fn new(seed: u64) -> Self {
        Self { seed }
    }

    fn next_u32(&mut self) -> u32 {
        // Simple linear congruential generator
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.seed >> 16) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn next_bool(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }

    fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        let range = max - min;
        min + (self.next_u32() as i32 % (range + 1))
    }
}

#[derive(Clone, Debug)]
struct SimulationMode {
    deterministic: bool,
    rng: DeterministicRandom,
    tick_number: u32,
}

impl SimulationMode {
    fn new(deterministic: bool, seed: u64) -> Self {
        Self {
            deterministic,
            rng: DeterministicRandom::new(seed),
            tick_number: 0,
        }
    }

    fn random_force(&mut self) -> Vec2 {
        if self.deterministic {
            let angle = self.rng.next_f32() * std::f32::consts::PI * 2.0;
            let magnitude = self.rng.next_f32() * 50.0 + 25.0;
            Vec2::new(angle.cos(), angle.sin()) * magnitude
        } else {
            let angle = rand::random::<f32>() * std::f32::consts::PI * 2.0;
            let magnitude = rand::random::<f32>() * 50.0 + 25.0;
            Vec2::new(angle.cos(), angle.sin()) * magnitude
        }
    }

    fn random_position_offset(&mut self) -> Vec2 {
        if self.deterministic {
            Vec2::new(
                (self.rng.next_f32() - 0.5) * 20.0,
                (self.rng.next_f32() - 0.5) * 20.0,
            )
        } else {
            Vec2::new(
                (rand::random::<f32>() - 0.5) * 20.0,
                (rand::random::<f32>() - 0.5) * 20.0,
            )
        }
    }

    fn random_particle_count(&mut self) -> usize {
        if self.deterministic {
            5 + self.rng.range_i32(0, 5) as usize
        } else {
            5 + rand::random::<usize>() % 6
        }
    }

    fn advance_tick(&mut self) {
        self.tick_number += 1;
    }
}

#[derive(Clone, Debug)]
struct PhysicsParticle {
    position: Vec2,
    velocity: Vec2,
    mass: f32,
    id: u32,
    last_update_tick: u32,
}

impl PhysicsParticle {
    fn new(position: Vec2, id: u32) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            mass: 1.0,
            id,
            last_update_tick: 0,
        }
    }

    fn apply_force(&mut self, force: Vec2, dt: f32) {
        let acceleration = force / self.mass;
        self.velocity += acceleration * dt;
    }

    fn update(&mut self, dt: f32, tick_number: u32) {
        // Apply gravity
        self.velocity.y -= 300.0 * dt;

        // Update position
        self.position += self.velocity * dt;

        // Simple boundary collision
        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.x *= -0.8;
        }
        if self.position.x > 800.0 {
            self.position.x = 800.0;
            self.velocity.x *= -0.8;
        }
        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y *= -0.5;
        }

        self.last_update_tick = tick_number;
    }
}

#[derive(Clone, Debug)]
struct NondeterminismDemo {
    deterministic_sim: SimulationMode,
    nondeterministic_sim: SimulationMode,
    particles_det: Vec<PhysicsParticle>,
    particles_nondet: Vec<PhysicsParticle>,
    next_particle_id: u32,
    sync_events: Vec<SyncEvent>,
    desync_detected: bool,
    last_sync_tick: u32,
}

#[derive(Clone, Debug)]
enum SyncEvent {
    ParticleCreated { id: u32, position: Vec2, tick: u32 },
    ForceApplied { particle_id: u32, force: Vec2, tick: u32 },
}

impl NondeterminismDemo {
    fn new() -> Self {
        let seed = 12345; // Same seed for both simulations initially

        Self {
            deterministic_sim: SimulationMode::new(true, seed),
            nondeterministic_sim: SimulationMode::new(false, seed),
            particles_det: Vec::new(),
            particles_nondet: Vec::new(),
            next_particle_id: 0,
            sync_events: Vec::new(),
            desync_detected: false,
            last_sync_tick: 0,
        }
    }

    fn create_particle(&mut self, base_position: Vec2) {
        let tick = self.deterministic_sim.tick_number;

        // Deterministic simulation
        let pos_offset_det = self.deterministic_sim.random_position_offset();
        let particle_det = PhysicsParticle::new(base_position + pos_offset_det, self.next_particle_id);
        self.particles_det.push(particle_det);

        // Nondeterministic simulation
        let pos_offset_nondet = self.nondeterministic_sim.random_position_offset();
        let particle_nondet = PhysicsParticle::new(base_position + pos_offset_nondet, self.next_particle_id);
        self.particles_nondet.push(particle_nondet);

        // Record sync event
        self.sync_events.push(SyncEvent::ParticleCreated {
            id: self.next_particle_id,
            position: base_position,
            tick,
        });

        self.next_particle_id += 1;
    }

    fn apply_random_forces(&mut self) {
        let tick = self.deterministic_sim.tick_number;

        // Apply to deterministic particles
        for particle in &mut self.particles_det {
            if self.deterministic_sim.random_bool(0.3) {
                let force = self.deterministic_sim.random_force();
                particle.apply_force(force, 1.0 / 60.0);

                self.sync_events.push(SyncEvent::ForceApplied {
                    particle_id: particle.id,
                    force,
                    tick,
                });
            }
        }

        // Apply to nondeterministic particles (using different RNG)
        for particle in &mut self.particles_nondet {
            if rand::random::<f32>() < 0.3 {
                let force = self.nondeterministic_sim.random_force();
                particle.apply_force(force, 1.0 / 60.0);
            }
        }
    }

    fn update(&mut self, dt: f32) {
        // Update both simulations
        self.deterministic_sim.advance_tick();
        self.nondeterministic_sim.advance_tick();

        let current_tick = self.deterministic_sim.tick_number;

        // Update deterministic particles
        for particle in &mut self.particles_det {
            particle.update(dt, current_tick);
        }

        // Update nondeterministic particles
        for particle in &mut self.particles_nondet {
            particle.update(dt, current_tick);
        }

        // Check for desync every 60 ticks
        if current_tick % 60 == 0 && current_tick > 0 {
            self.check_desync();
            self.last_sync_tick = current_tick;
        }
    }

    fn check_desync(&mut self) {
        if self.particles_det.len() != self.particles_nondet.len() {
            self.desync_detected = true;
            println!("🚨 DESYNC DETECTED: Different particle counts!");
            return;
        }

        for i in 0..self.particles_det.len() {
            let det = &self.particles_det[i];
            let nondet = &self.particles_nondet[i];

            let position_diff = (det.position - nondet.position).length();
            let velocity_diff = (det.velocity - nondet.velocity).length();

            if position_diff > 1.0 || velocity_diff > 1.0 {
                self.desync_detected = true;
                println!("🚨 DESYNC DETECTED: Particle {} desynced!", det.id);
                println!("  Position diff: {:.2}, Velocity diff: {:.2}", position_diff, velocity_diff);
                break;
            }
        }

        if !self.desync_detected {
            println!("✅ Simulations still synchronized at tick {}", self.deterministic_sim.tick_number);
        }
    }

    fn resync_simulations(&mut self) {
        println!("🔄 Attempting to resync simulations...");

        // In a real game, you'd send authoritative state from server
        // Here we'll just reset the nondeterministic simulation to match deterministic

        self.nondeterministic_sim = self.deterministic_sim.clone();
        self.particles_nondet = self.particles_det.clone();
        self.desync_detected = false;

        println!("✅ Simulations resynced");
    }

    fn get_sync_stats(&self) -> (f32, usize, usize) {
        let total_particles = self.particles_det.len();
        let synced_particles = self.particles_det.iter().zip(&self.particles_nondet)
            .filter(|(det, nondet)| {
                let pos_diff = (det.position - nondet.position).length();
                let vel_diff = (det.velocity - nondet.velocity).length();
                pos_diff < 1.0 && vel_diff < 1.0
            })
            .count();

        let sync_percentage = if total_particles > 0 {
            synced_particles as f32 / total_particles as f32 * 100.0
        } else {
            100.0
        };

        (sync_percentage, synced_particles, total_particles)
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Nondeterminism And You - Deterministic vs Non-Deterministic Simulation".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(NondeterminismDemo::new())
        .add_systems(Startup, setup_nondeterminism_demo)
        .add_systems(Update, (
            handle_nondeterminism_input,
            update_nondeterminism_simulation,
            render_nondeterminism_demo,
            display_nondeterminism_info,
        ).chain())
        .run();
}

fn setup_nondeterminism_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Add some initial particles
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(10.0, 10.0)),
            ..default()
        },
        transform: Transform::from_xyz(400.0, 300.0, 0.0),
        ..default()
    });
}

fn handle_nondeterminism_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<NondeterminismDemo>,
) {
    // Get mouse position
    let mouse_pos = if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    world_pos.origin.truncate()
                } else {
                    Vec2::ZERO
                }
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        }
    } else {
        Vec2::ZERO
    };

    // Create particles
    if mouse_input.just_pressed(MouseButton::Left) {
        demo.create_particle(mouse_pos);
    }

    // Apply random forces
    if keyboard_input.just_pressed(KeyCode::Space) {
        demo.apply_random_forces();
    }

    // Resync simulations
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        demo.resync_simulations();
    }

    // Check sync status
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        demo.check_desync();
    }
}

fn update_nondeterminism_simulation(time: Res<Time>, mut demo: ResMut<NondeterminismDemo>) {
    demo.update(time.delta_seconds().min(1.0 / 30.0));
}

fn render_nondeterminism_demo(
    mut commands: Commands,
    mut particle_entities: Local<Vec<Entity>>,
    demo: Res<NondeterminismDemo>,
) {
    // Clear previous frame
    for entity in particle_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render deterministic particles (left side)
    for particle in &demo.particles_det {
        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.8, 0.2), // Green for deterministic
                    custom_size: Some(Vec2::new(8.0, 8.0)),
                    ..default()
                },
                transform: Transform::from_xyz(particle.position.x - 200.0, particle.position.y, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    format!("{}", particle.id),
                    TextStyle {
                        font_size: 8.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ..default()
            },
        )).id();
        particle_entities.push(entity);
    }

    // Render nondeterministic particles (right side)
    for particle in &demo.particles_nondet {
        let color = if demo.desync_detected {
            Color::rgb(0.8, 0.2, 0.2) // Red when desynced
        } else {
            Color::rgb(0.2, 0.4, 0.8) // Blue for nondeterministic
        };

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(8.0, 8.0)),
                    ..default()
                },
                transform: Transform::from_xyz(particle.position.x + 200.0, particle.position.y, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    format!("{}", particle.id),
                    TextStyle {
                        font_size: 8.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ..default()
            },
        )).id();
        particle_entities.push(entity);
    }

    // Render center divider
    let divider_entity = commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(2.0, 600.0)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 300.0, 0.0),
        ..default()
    }).id();
    particle_entities.push(divider_entity);
}

fn display_nondeterminism_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<NondeterminismDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        let (sync_percent, synced, total) = demo.get_sync_stats();

        println!("\n=== Nondeterminism And You Demo ===");
        println!("Current Tick: {}", demo.deterministic_sim.tick_number);
        println!("Desync Detected: {}", demo.desync_detected);
        println!("Sync Percentage: {:.1}%", sync_percent);
        println!("Synced Particles: {}/{}", synced, total);
        println!("Total Sync Events: {}", demo.sync_events.len());
        println!("Last Sync Check: Tick {}", demo.last_sync_tick);

        println!("\nSimulation Status:");
        println!("  Deterministic RNG Seed: {}", demo.deterministic_sim.rng.seed);
        println!("  Deterministic Particles: {}", demo.particles_det.len());
        println!("  Non-deterministic Particles: {}", demo.particles_nondet.len());

        println!("\nControls:");
        println!("  Left Click: Create particles");
        println!("  Space: Apply random forces");
        println!("  R: Resync simulations");
        println!("  S: Check sync status");
        println!("  H: Show this info");
        println!("\nNote: Left=Deterministic, Right=Non-deterministic");
        println!("======================\n");
    }
}
