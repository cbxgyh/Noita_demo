// Example: Multiplayer Fever Dreams
// Based on "Multiplayer Fever Dreams" blog post
// https://www.slowrush.dev/news/multiplayer-fever-dreams

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::VecDeque;

// Advanced multiplayer networking concepts
// Demonstrates rollback, prediction, and network state management

#[derive(Clone, Debug)]
struct Player {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    input: PlayerInput,
    predicted_position: Vec2,
    last_acknowledged_input: u32,
}

#[derive(Clone, Debug,Default)]
struct PlayerInput {
    frame: u32,
    move_x: f32,
    jump: bool,
}

#[derive(Clone, Debug)]
struct NetworkState {
    frame: u32,
    player_positions: Vec<Vec2>,
    atom_world_hash: u64,
}

#[derive(Resource)]
struct NetworkManager {
    is_server: bool,
    local_player_id: u32,
    remote_player_id: u32,
    current_frame: u32,
    input_delay: u32,
    rollback_frames: u32,
    ping: f32,
    packet_loss: f32,
}

#[derive(Resource)]
struct InputBuffer {
    local_inputs: VecDeque<PlayerInput>,
    remote_inputs: VecDeque<PlayerInput>,
    max_size: usize,
}

#[derive(Resource)]
struct StateHistory {
    states: VecDeque<NetworkState>,
    max_states: usize,
}

#[derive(Resource)]
struct PredictionSystem {
    predicted_states: VecDeque<NetworkState>,
    max_predictions: usize,
}

#[derive(Resource)]
struct GameWorld {
    players: Vec<Player>,
    width: f32,
    height: f32,
    atom_positions: Vec<Vec2>, // Simplified atom world for demo
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self {
            is_server: true,
            local_player_id: 0,
            remote_player_id: 1,
            current_frame: 0,
            input_delay: 3, // 3 frames of input delay
            rollback_frames: 5, // Can rollback up to 5 frames
            ping: 50.0, // 50ms
            packet_loss: 0.02, // 2% packet loss
        }
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            local_inputs: VecDeque::new(),
            remote_inputs: VecDeque::new(),
            max_size: 60, // 1 second at 60fps
        }
    }
}

impl Default for StateHistory {
    fn default() -> Self {
        Self {
            states: VecDeque::new(),
            max_states: 20,
        }
    }
}

impl Default for PredictionSystem {
    fn default() -> Self {
        Self {
            predicted_states: VecDeque::new(),
            max_predictions: 10,
        }
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self {
            players: vec![
                Player {
                    id: 0,
                    position: Vec2::new(100.0, 100.0),
                    velocity: Vec2::ZERO,
                    input: PlayerInput::default(),
                    predicted_position: Vec2::new(100.0, 100.0),
                    last_acknowledged_input: 0,
                },
                Player {
                    id: 1,
                    position: Vec2::new(200.0, 100.0),
                    velocity: Vec2::ZERO,
                    input: PlayerInput::default(),
                    predicted_position: Vec2::new(200.0, 100.0),
                    last_acknowledged_input: 0,
                },
            ],
            width: 400.0,
            height: 300.0,
            atom_positions: Vec::new(),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Multiplayer Fever Dreams - Advanced Networking".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(NetworkManager::default())
        .insert_resource(InputBuffer::default())
        .insert_resource(StateHistory::default())
        .insert_resource(PredictionSystem::default())
        .insert_resource(GameWorld::default())
        .add_systems(Startup, setup_network_demo)
        .add_systems(Update, (
            update_network_simulation,
            handle_player_input,
            simulate_network_conditions,
            perform_rollback_if_needed,
            update_predictions,
            render_network_demo,
            display_network_stats,
        ).chain())
        .run();
}

fn setup_network_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Spawn some visual elements
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(400.0, 20.0)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, -140.0, 0.0),
        ..default()
    });
}

fn update_network_simulation(
    mut network: ResMut<NetworkManager>,
    mut game_world: ResMut<GameWorld>,
    mut input_buffer: ResMut<InputBuffer>,
    mut state_history: ResMut<StateHistory>,
    time: Res<Time>,
) {
    network.current_frame += 1;
    let dt = time.delta_seconds();

    let game_worldw =game_world.width;
    let game_worldh =game_world.height;

    // Update game world
    for player in &mut game_world.players {
        update_player_physics(player, dt, game_worldw, game_worldh);

        // Store predicted position
        player.predicted_position = player.position;
    }

    // Save state for rollback
    save_game_state(&network, &game_world, &mut state_history);
}

fn update_player_physics(player: &mut Player, dt: f32, world_width: f32, world_height: f32) {
    // Apply input
    let move_speed = 150.0;
    player.velocity.x = player.input.move_x * move_speed;

    // Apply gravity
    player.velocity.y -= 300.0 * dt;

    // Update position
    player.position += player.velocity * dt;

    // Ground collision (simple)
    if player.position.y < -100.0 {
        player.position.y = -100.0;
        player.velocity.y = 0.0;

        // Handle jump
        if player.input.jump {
            player.velocity.y = 200.0;
        }
    }

    // World bounds
    player.position.x = player.position.x.clamp(-world_width / 2.0 + 10.0, world_width / 2.0 - 10.0);
}

fn save_game_state(
    network: &NetworkManager,
    game_world: &GameWorld,
    state_history: &mut StateHistory,
) {
    let state = NetworkState {
        frame: network.current_frame,
        player_positions: game_world.players.iter().map(|p| p.position).collect(),
        atom_world_hash: 0, // Simplified
    };

    state_history.states.push_back(state);

    // Keep only recent states
    while state_history.states.len() > state_history.max_states {
        state_history.states.pop_front();
    }
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut input_buffer: ResMut<InputBuffer>,
    mut game_world: ResMut<GameWorld>,
    network: Res<NetworkManager>,
) {
    // Create input for current frame
    let input = PlayerInput {
        frame: network.current_frame,
        move_x: if keyboard_input.pressed(KeyCode::KeyA) {
            -1.0
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            1.0
        } else {
            0.0
        },
        jump: keyboard_input.just_pressed(KeyCode::Space),
    };

    // Apply input locally
    if let Some(local_player) = game_world.players.get_mut(network.local_player_id as usize) {
        local_player.input = input.clone();
    }

    // Buffer input for network transmission
    input_buffer.local_inputs.push_back(input);

    // Keep buffer size manageable
    while input_buffer.local_inputs.len() > input_buffer.max_size {
        input_buffer.local_inputs.pop_front();
    }
}

fn simulate_network_conditions(
    mut network: ResMut<NetworkManager>,
    mut input_buffer: ResMut<InputBuffer>,
    mut game_world: ResMut<GameWorld>,
) {
    // Simulate receiving remote input with delay and packet loss
    static mut SIMULATION_FRAME: u32 = 0;
    unsafe {
        SIMULATION_FRAME += 1;

        // Simulate packet reception every few frames
        if SIMULATION_FRAME % (network.input_delay + 1) == 0 {
            // Simulate packet loss
            if rand::random::<f32>() >= network.packet_loss {
                // Create fake remote input
                let remote_input = PlayerInput {
                    frame: network.current_frame.saturating_sub(network.input_delay),
                    move_x: (rand::random::<f32>() - 0.5) * 2.0,
                    jump: rand::random::<f32>() < 0.1,
                };



                // Apply remote input
                if let Some(remote_player) = game_world.players.get_mut(network.remote_player_id as usize) {
                    remote_player.input = remote_input.clone();
                }
                input_buffer.remote_inputs.push_back(remote_input);
            }
        }
    }
}

fn perform_rollback_if_needed(
    mut network: ResMut<NetworkManager>,
    mut game_world: ResMut<GameWorld>,
    mut state_history: ResMut<StateHistory>,
    input_buffer: Res<InputBuffer>,
) {
    // Check for input discrepancies (simplified)
    static mut LAST_ROLLBACK_CHECK: u32 = 0;
    unsafe {
        if network.current_frame - LAST_ROLLBACK_CHECK > 60 { // Check every second
            LAST_ROLLBACK_CHECK = network.current_frame;

            // Simulate occasional rollback
            if rand::random::<f32>() < 0.1 { // 10% chance
                perform_rollback(&mut network, &mut game_world, &mut state_history);
            }
        }
    }
}

fn perform_rollback(
    network: &mut NetworkManager,
    game_world: &mut GameWorld,
    state_history: &mut StateHistory,
) {
    println!("🔄 Performing rollback at frame {}", network.current_frame);

    // Find state to rollback to
    let rollback_frames = 3;
    let target_frame = network.current_frame.saturating_sub(rollback_frames);

    // Find the state in history
    if let Some(rollback_state) = state_history.states.iter().find(|s| s.frame == target_frame) {
        println!("Rolling back to frame {}", rollback_state.frame);

        // Restore player positions
        for (i, position) in rollback_state.player_positions.iter().enumerate() {
            if let Some(player) = game_world.players.get_mut(i) {
                player.position = *position;
                player.predicted_position = *position;
            }
        }

        // Re-simulate from rollback point
        network.current_frame = target_frame;

        // In a real implementation, you would re-apply all inputs from the rollback point
        println!("Re-simulating from frame {}", target_frame);
    }
}

fn update_predictions(
    mut prediction_system: ResMut<PredictionSystem>,
    game_world: Res<GameWorld>,
    network: Res<NetworkManager>,
) {
    // Update prediction system
    let current_state = NetworkState {
        frame: network.current_frame,
        player_positions: game_world.players.iter().map(|p| p.predicted_position).collect(),
        atom_world_hash: 0,
    };

    prediction_system.predicted_states.push_back(current_state);

    while prediction_system.predicted_states.len() > prediction_system.max_predictions {
        prediction_system.predicted_states.pop_front();
    }
}

fn render_network_demo(
    mut commands: Commands,
    mut player_entities: Local<Vec<Entity>>,
    game_world: Res<GameWorld>,
    network: Res<NetworkManager>,
) {
    // Clear previous frame
    for entity in player_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render players
    for player in &game_world.players {
        let color = if player.id == network.local_player_id {
            Color::rgb(0.2, 0.8, 0.2) // Local player: Green
        } else {
            Color::rgb(0.8, 0.2, 0.2) // Remote player: Red
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
}

fn display_network_stats(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    network: Res<NetworkManager>,
    input_buffer: Res<InputBuffer>,
    state_history: Res<StateHistory>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Multiplayer Fever Dreams Demo ===");
        println!("Networking Concepts:");
        println!("- Input buffering with {} frame delay", network.input_delay);
        println!("- Rollback system (up to {} frames)", network.rollback_frames);
        println!("- Client-side prediction");
        println!("- Simulated packet loss: {:.1}%", network.packet_loss * 100.0);
        println!("- Simulated ping: {:.0}ms", network.ping);
        println!("");
        println!("Current State:");
        println!("- Frame: {}", network.current_frame);
        println!("- Local inputs buffered: {}", input_buffer.local_inputs.len());
        println!("- Remote inputs buffered: {}", input_buffer.remote_inputs.len());
        println!("- States in history: {}", state_history.states.len());
        println!("");
        println!("Controls:");
        println!("A/D: Move left/right");
        println!("Space: Jump");
        println!("H: Show this help");
        println!("=====================================\n");
    }

    // Periodically show stats
    static mut LAST_STATS: f32 = 0.0;
    unsafe {
        LAST_STATS += 0.016; // Assume 60fps
        if LAST_STATS > 5.0 { // Every 5 seconds
            LAST_STATS = 0.0;
            println!("Frame: {} | Ping: {:.0}ms | Local buffer: {} | Remote buffer: {}",
                    network.current_frame, network.ping,
                    input_buffer.local_inputs.len(), input_buffer.remote_inputs.len());
        }
    }
}
