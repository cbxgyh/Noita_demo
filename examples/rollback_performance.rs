// Example: Rollback Performance
// Based on "Rollback Performance" blog post
// https://www.slowrush.dev/news/rollback-performance

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::VecDeque;

// Rollback networking performance optimization
// Demonstrates tick-based simulation, state snapshots, and rollback mechanics

#[derive(Clone, Debug)]
struct GameTick {
    tick_number: u32,
    timestamp: f64,
}

impl GameTick {
    fn new(tick_number: u32, timestamp: f64) -> Self {
        Self {
            tick_number,
            timestamp,
        }
    }
}

#[derive(Clone, Debug)]
struct PlayerState {
    position: Vec2,
    velocity: Vec2,
    health: f32,
    input_buffer: VecDeque<PlayerInput>,
    last_processed_tick: u32,
}

impl PlayerState {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            health: 100.0,
            input_buffer: VecDeque::new(),
            last_processed_tick: 0,
        }
    }

    fn apply_input(&mut self, input: &PlayerInput, dt: f32) {
        // Apply movement
        self.velocity.x = input.move_x * 150.0;

        // Apply gravity
        self.velocity.y -= 300.0 * dt;

        // Update position
        self.position += self.velocity * dt;

        // Simple ground collision
        if self.position.y <= 50.0 {
            self.position.y = 50.0;
            self.velocity.y = 0.0;

            if input.jump {
                self.velocity.y = 200.0;
            }
        }

        // Apply damage if hitting hazards
        if input.take_damage {
            self.health -= 10.0;
            if self.health <= 0.0 {
                // Respawn
                self.position = Vec2::new(100.0, 100.0);
                self.velocity = Vec2::ZERO;
                self.health = 100.0;
            }
        }
    }
}

#[derive(Clone, Debug)]
struct PlayerInput {
    move_x: f32,
    jump: bool,
    take_damage: bool,
    tick_number: u32,
}

impl PlayerInput {
    fn new(tick_number: u32) -> Self {
        Self {
            move_x: 0.0,
            jump: false,
            take_damage: false,
            tick_number,
        }
    }
}

#[derive(Clone, Debug)]
struct GameState {
    tick: GameTick,
    players: Vec<PlayerState>,
    projectiles: Vec<Projectile>,
    hazards: Vec<Hazard>,
}

impl GameState {
    fn new(tick_number: u32, timestamp: f64, player_count: usize) -> Self {
        let mut players = Vec::new();
        for i in 0..player_count {
            let x = 100.0 + i as f32 * 150.0;
            players.push(PlayerState::new(Vec2::new(x, 100.0)));
        }

        Self {
            tick: GameTick::new(tick_number, timestamp),
            players,
            projectiles: Vec::new(),
            hazards: vec![
                Hazard::new(Vec2::new(300.0, 100.0), 20.0),
                Hazard::new(Vec2::new(500.0, 100.0), 20.0),
            ],
        }
    }

    fn advance_tick(&mut self, inputs: &[PlayerInput], dt: f32) {
        self.tick.tick_number += 1;
        self.tick.timestamp += dt as f64;

        // Apply inputs to players
        for (player_idx, player) in self.players.iter_mut().enumerate() {
            if let Some(input) = inputs.get(player_idx) {
                player.apply_input(input, dt);
                player.last_processed_tick = input.tick_number;

                // Store input in buffer
                player.input_buffer.push_back(input.clone());
                if player.input_buffer.len() > 60 { // Keep 60 ticks of history
                    player.input_buffer.pop_front();
                }
            }
        }

        // Update projectiles
        self.projectiles.retain_mut(|proj| {
            proj.update(dt);
            proj.is_alive()
        });

        // Check collisions
        self.check_collisions();
    }

    fn check_collisions(&mut self) {
        // Player vs hazard collisions
        for player in &mut self.players {
            for hazard in &self.hazards {
                let distance = player.position.distance(hazard.position);
                if distance < hazard.radius + 15.0 {
                    // Simulate damage input
                    let damage_input = PlayerInput {
                        move_x: 0.0,
                        jump: false,
                        take_damage: true,
                        tick_number: self.tick.tick_number,
                    };
                    player.apply_input(&damage_input, 1.0 / 60.0);
                }
            }
        }
    }

    fn rollback_to_tick(&mut self, target_tick: u32, input_history: &[Vec<PlayerInput>]) -> bool {
        if target_tick >= self.tick.tick_number {
            return false; // Cannot rollback to future
        }

        println!("🔄 Rolling back from tick {} to {}", self.tick.tick_number, target_tick);

        // Find the most recent state before target tick
        // In a real implementation, you'd have a state history buffer
        let ticks_to_rewind = self.tick.tick_number - target_tick;

        // Simplified rollback: reset to initial state and replay
        let initial_state = GameState::new(target_tick, self.tick.timestamp - (ticks_to_rewind as f64 * (1.0 / 60.0)), self.players.len());

        *self = initial_state;

        // Replay inputs from target tick onwards
        for tick_offset in 0..ticks_to_rewind {
            let replay_tick = target_tick + tick_offset;
            let mut replay_inputs = Vec::new();

            for player_idx in 0..self.players.len() {
                if let Some(player_inputs) = input_history.get(player_idx) {
                    // Find input for this tick
                    if let Some(input) = player_inputs.iter().find(|i| i.tick_number == replay_tick) {
                        replay_inputs.push(input.clone());
                    } else {
                        // Use default input if none found
                        replay_inputs.push(PlayerInput::new(replay_tick));
                    }
                }
            }

            self.advance_tick(&replay_inputs, 1.0 / 60.0);
        }

        true
    }
}

#[derive(Clone, Debug)]
struct Projectile {
    position: Vec2,
    velocity: Vec2,
    lifetime: f32,
    owner: usize, // Player index
}

impl Projectile {
    fn new(position: Vec2, direction: Vec2, owner: usize) -> Self {
        Self {
            position,
            velocity: direction.normalize() * 200.0,
            lifetime: 3.0,
            owner,
        }
    }

    fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
        self.lifetime -= dt;
    }

    fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }
}

#[derive(Clone, Debug)]
struct Hazard {
    position: Vec2,
    radius: f32,
}

impl Hazard {
    fn new(position: Vec2, radius: f32) -> Self {
        Self { position, radius }
    }
}

#[derive(Clone, Debug)]
struct NetworkMessage {
    tick_number: u32,
    player_inputs: Vec<PlayerInput>,
    timestamp: f64,
}

#[derive(Clone, Debug)]
struct RollbackStats {
    rollbacks_performed: u32,
    average_rollback_distance: f32,
    total_simulation_time: f64,
    ticks_simulated: u32,
}

impl RollbackStats {
    fn new() -> Self {
        Self {
            rollbacks_performed: 0,
            average_rollback_distance: 0.0,
            total_simulation_time: 0.0,
            ticks_simulated: 0,
        }
    }

    fn record_rollback(&mut self, distance: u32) {
        self.rollbacks_performed += 1;
        self.average_rollback_distance = (self.average_rollback_distance * (self.rollbacks_performed - 1) as f32 + distance as f32) / self.rollbacks_performed as f32;
    }
}

#[derive(Resource)]
struct RollbackDemo {
    game_state: GameState,
    input_history: Vec<Vec<PlayerInput>>,
    network_messages: VecDeque<NetworkMessage>,
    stats: RollbackStats,
    local_player: usize,
    current_tick: u32,
    tick_timer: f32,
    tick_rate: f32,
    simulated_latency: f32,
}

impl RollbackDemo {
    fn new() -> Self {
        let game_state = GameState::new(0, 0.0, 2);
        let input_history = vec![Vec::new(); 2];

        Self {
            game_state,
            input_history,
            network_messages: VecDeque::new(),
            stats: RollbackStats::new(),
            local_player: 0,
            current_tick: 0,
            tick_timer: 0.0,
            tick_rate: 1.0 / 60.0, // 60 TPS
            simulated_latency: 0.1, // 100ms latency
        }
    }

    fn update(&mut self, dt: f32, local_input: PlayerInput) {
        self.tick_timer += dt;
        self.stats.total_simulation_time += dt as f64;

        // Collect local input
        local_input.tick_number = self.current_tick;
        self.input_history[self.local_player].push(local_input.clone());

        // Simulate receiving network messages with latency
        let delayed_tick = (self.current_tick as f32 - (self.simulated_latency / self.tick_rate)) as u32;
        if delayed_tick > 0 {
            let delayed_input = PlayerInput::new(delayed_tick);
            let network_msg = NetworkMessage {
                tick_number: delayed_tick,
                player_inputs: vec![delayed_input],
                timestamp: self.stats.total_simulation_time - self.simulated_latency as f64,
            };
            self.network_messages.push_back(network_msg);
        }

        // Process network messages and handle rollbacks
        while let Some(msg) = self.network_messages.front() {
            if msg.timestamp <= self.stats.total_simulation_time {
                let msg = self.network_messages.pop_front().unwrap();

                // Check if we need to rollback
                if msg.tick_number < self.game_state.tick.tick_number {
                    let rollback_distance = self.game_state.tick.tick_number - msg.tick_number;
                    self.game_state.rollback_to_tick(msg.tick_number, &self.input_history);
                    self.stats.record_rollback(rollback_distance);
                }

                // Apply the network input
                self.game_state.advance_tick(&msg.player_inputs, self.tick_rate);
            } else {
                break;
            }
        }

        // Advance simulation if we're caught up
        while self.tick_timer >= self.tick_rate {
            self.tick_timer -= self.tick_rate;
            self.current_tick += 1;

            // Create inputs for all players (simplified)
            let mut inputs = Vec::new();
            for player_idx in 0..self.game_state.players.len() {
                if player_idx == self.local_player {
                    inputs.push(local_input.clone());
                } else {
                    // Simulate remote player input
                    let mut remote_input = PlayerInput::new(self.current_tick);
                    remote_input.move_x = (rand::random::<f32>() - 0.5) * 2.0;
                    remote_input.jump = rand::random::<bool>() && rand::random::<f32>() < 0.1;
                    inputs.push(remote_input);
                }
            }

            self.game_state.advance_tick(&inputs, self.tick_rate);
            self.stats.ticks_simulated += 1;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rollback Performance - Network Simulation".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(RollbackDemo::new())
        .add_systems(Startup, setup_rollback_demo)
        .add_systems(Update, (
            handle_rollback_input,
            update_rollback_simulation,
            render_rollback_demo,
            display_rollback_stats,
        ).chain())
        .run();
}

fn setup_rollback_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_rollback_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<RollbackDemo>,
) {
    let mut input = PlayerInput::new(demo.current_tick);

    // Movement
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.move_x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.move_x = 1.0;
    }

    input.jump = keyboard_input.just_pressed(KeyCode::Space);

    // Adjust latency
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        demo.simulated_latency = (demo.simulated_latency + 0.02).min(0.5);
        println!("Latency increased to {:.0}ms", demo.simulated_latency * 1000.0);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        demo.simulated_latency = (demo.simulated_latency - 0.02).max(0.0);
        println!("Latency decreased to {:.0}ms", demo.simulated_latency * 1000.0);
    }

    demo.update(1.0 / 60.0, input);
}

fn update_rollback_simulation(time: Res<Time>, mut demo: ResMut<RollbackDemo>) {
    // Updates are handled in input system
}

fn render_rollback_demo(
    mut commands: Commands,
    mut player_entities: Local<Vec<Entity>>,
    mut hazard_entities: Local<Vec<Entity>>,
    mut projectile_entities: Local<Vec<Entity>>,
    demo: Res<RollbackDemo>,
) {
    // Clear previous frame
    for entity in player_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in hazard_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in projectile_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render players
    for (i, player) in demo.game_state.players.iter().enumerate() {
        let color = if i == demo.local_player {
            Color::rgb(0.2, 0.8, 0.2) // Local player
        } else {
            Color::rgb(0.8, 0.2, 0.2) // Remote player
        };

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(20.0, 30.0)),
                    ..default()
                },
                transform: Transform::from_xyz(player.position.x, player.position.y, 1.0),
                ..default()
            },

        )).id();
        player_entities.push(entity);
    }

    // Render hazards
    for hazard in &demo.game_state.hazards {
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.8, 0.2, 0.2),
                custom_size: Some(Vec2::new(hazard.radius * 2.0, hazard.radius * 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(hazard.position.x, hazard.position.y, 0.0),
            ..default()
        }).id();
        hazard_entities.push(entity);
    }

    // Render projectiles
    for projectile in &demo.game_state.projectiles {
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 0.0),
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            transform: Transform::from_xyz(projectile.position.x, projectile.position.y, 1.0),
            ..default()
        }).id();
        projectile_entities.push(entity);
    }
}

fn display_rollback_stats(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<RollbackDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Rollback Performance Demo ===");
        println!("Current Tick: {}", demo.game_state.tick.tick_number);
        println!("Simulation Time: {:.2}s", demo.stats.total_simulation_time);
        println!("Ticks Simulated: {}", demo.stats.ticks_simulated);
        println!("Rollbacks Performed: {}", demo.stats.rollbacks_performed);
        println!("Avg Rollback Distance: {:.1} ticks", demo.stats.average_rollback_distance);
        println!("Simulated Latency: {:.0}ms", demo.simulated_latency * 1000.0);
        println!("Network Messages Queued: {}", demo.network_messages.len());

        println!("\nPlayer Positions:");
        for (i, player) in demo.game_state.players.iter().enumerate() {
            println!("  Player {}: ({:.1}, {:.1}) HP: {:.0}",
                    i + 1, player.position.x, player.position.y, player.health);
        }

        println!("\nControls:");
        println!("  A/D: Move | Space: Jump");
        println!("  ↑/↓: Adjust latency");
        println!("  H: Show this info");
        println!("=======================\n");
    }
}
