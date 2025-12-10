// Example: Sorta Working Networking
// Based on "Sorta Working Networking" blog post
// https://www.slowrush.dev/news/sorta-working-networking

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::VecDeque;

// Sorta working networking implementation
// Demonstrates basic client-server architecture, state synchronization, and rollback

#[derive(Clone, Debug, PartialEq)]
enum NetworkRole {
    Server,
    Client { server_entity_id: u32 },
}

#[derive(Clone, Debug)]
struct NetworkedGameState {
    tick: u32,
    timestamp: f64,
    entities: Vec<NetworkedEntity>,
    inputs: Vec<PlayerInput>,
}

#[derive(Clone, Debug)]
struct NetworkedEntity {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    health: f32,
    entity_type: EntityType,
    owner: Option<u32>, // Player ID who owns this entity
    last_authoritative_update: u32, // Tick when last authoritative update was received
}

#[derive(Clone, Debug, PartialEq)]
enum EntityType {
    Player,
    Projectile,
    Environment,
}

#[derive(Clone, Debug)]
struct PlayerInput {
    player_id: u32,
    tick: u32,
    move_x: f32,
    jump: bool,
    shoot: bool,
    timestamp: f64,
}

impl PlayerInput {
    fn new(player_id: u32, tick: u32) -> Self {
        Self {
            player_id,
            tick,
            move_x: 0.0,
            jump: false,
            shoot: false,
            timestamp: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
enum NetworkPacket {
    StateUpdate(NetworkedGameState),
    InputUpdate(Vec<PlayerInput>),
    JoinRequest { player_name: String },
    JoinResponse { player_id: u32, initial_state: NetworkedGameState },
    Ping { timestamp: f64 },
    Pong { timestamp: f64 },
}

#[derive(Clone, Debug)]
struct NetworkConnection {
    role: NetworkRole,
    local_player_id: u32,
    outgoing_packets: VecDeque<NetworkPacket>,
    incoming_packets: VecDeque<NetworkPacket>,
    rtt: f64, // Round trip time
    last_ping_time: f64,
    packet_loss: f32,
}

impl NetworkConnection {
    fn new(role: NetworkRole, local_player_id: u32) -> Self {
        Self {
            role,
            local_player_id,
            outgoing_packets: VecDeque::new(),
            incoming_packets: VecDeque::new(),
            rtt: 0.1, // 100ms default
            last_ping_time: 0.0,
            packet_loss: 0.02, // 2% packet loss
        }
    }

    fn send_packet(&mut self, packet: NetworkPacket) {
        // Simulate packet loss
        if rand::random::<f32>() >= self.packet_loss {
            self.outgoing_packets.push_back(packet);
        }
    }

    fn receive_packet(&mut self, packet: NetworkPacket) {
        self.incoming_packets.push_back(packet);
    }

    fn process_outgoing_packets(&mut self, current_time: f64) -> Vec<NetworkPacket> {
        // In real networking, these would be sent over the network
        // Here we just return them for the simulation
        let mut packets = Vec::new();
        while let Some(packet) = self.outgoing_packets.pop_front() {
            packets.push(packet);
        }
        packets
    }

    fn get_incoming_packets(&mut self) -> Vec<NetworkPacket> {
        let mut packets = Vec::new();
        while let Some(packet) = self.incoming_packets.pop_front() {
            packets.push(packet);
        }
        packets
    }

    fn measure_ping(&mut self, current_time: f64) {
        if current_time - self.last_ping_time > 1.0 { // Ping every second
            self.send_packet(NetworkPacket::Ping { timestamp: current_time });
            self.last_ping_time = current_time;
        }
    }

    fn update_rtt(&mut self, ping_timestamp: f64, current_time: f64) {
        self.rtt = current_time - ping_timestamp;
    }
}

#[derive(Clone, Debug)]
struct InputBuffer {
    inputs: VecDeque<PlayerInput>,
    buffer_size: usize,
}

impl InputBuffer {
    fn new(buffer_size: usize) -> Self {
        Self {
            inputs: VecDeque::with_capacity(buffer_size),
            buffer_size,
        }
    }

    fn add_input(&mut self, input: PlayerInput) {
        self.inputs.push_back(input);
        if self.inputs.len() > self.buffer_size {
            self.inputs.pop_front();
        }
    }

    fn get_input_for_tick(&self, tick: u32) -> Option<&PlayerInput> {
        self.inputs.iter().find(|input| input.tick == tick)
    }

    fn get_latest_input(&self) -> Option<&PlayerInput> {
        self.inputs.back()
    }
}

#[derive(Clone, Debug)]
struct PredictionState {
    predicted_states: VecDeque<NetworkedGameState>,
    max_predictions: usize,
}

impl PredictionState {
    fn new(max_predictions: usize) -> Self {
        Self {
            predicted_states: VecDeque::with_capacity(max_predictions),
            max_predictions,
        }
    }

    fn add_prediction(&mut self, state: NetworkedGameState) {
        self.predicted_states.push_back(state);
        if self.predicted_states.len() > self.max_predictions {
            self.predicted_states.pop_front();
        }
    }

    fn get_prediction(&self, tick: u32) -> Option<&NetworkedGameState> {
        self.predicted_states.iter().find(|state| state.tick == tick)
    }

    fn clear_predictions_after(&mut self, tick: u32) {
        self.predicted_states.retain(|state| state.tick <= tick);
    }
}

#[derive(Resource)]
struct SortaWorkingNetworkingDemo {
    game_state: NetworkedGameState,
    connection: NetworkConnection,
    input_buffer: InputBuffer,
    prediction: PredictionState,
    current_time: f64,
    tick_timer: f32,
    tick_rate: f32,
    last_processed_tick: u32,
}

impl SortaWorkingNetworkingDemo {
    fn new(is_server: bool) -> Self {
        let role = if is_server {
            NetworkRole::Server
        } else {
            NetworkRole::Client { server_entity_id: 0 }
        };

        let local_player_id = if is_server { 0 } else { 1 };

        let mut initial_entities = Vec::new();

        // Create players
        for i in 0..2 {
            initial_entities.push(NetworkedEntity {
                id: i,
                position: Vec2::new(100.0 + i as f32 * 200.0, 200.0),
                velocity: Vec2::ZERO,
                health: 100.0,
                entity_type: EntityType::Player,
                owner: Some(i),
                last_authoritative_update: 0,
            });
        }

        let game_state = NetworkedGameState {
            tick: 0,
            timestamp: 0.0,
            entities: initial_entities,
            inputs: Vec::new(),
        };

        Self {
            game_state: game_state.clone(),
            connection: NetworkConnection::new(role, local_player_id),
            input_buffer: InputBuffer::new(60), // 60 ticks of input buffer
            prediction: PredictionState::new(10), // 10 ticks of prediction
            current_time: 0.0,
            tick_timer: 0.0,
            tick_rate: 1.0 / 30.0, // 30 TPS
            last_processed_tick: 0,
        }
    }

    fn update(&mut self, dt: f32, local_input: PlayerInput) {
        self.current_time += dt as f64;
        self.tick_timer += dt;

        // Measure ping
        self.connection.measure_ping(self.current_time);

        // Buffer local input
        let mut input_with_timestamp = local_input.clone();
        input_with_timestamp.timestamp = self.current_time;
        self.input_buffer.add_input(input_with_timestamp.clone());

        // Send input to server
        if matches!(self.connection.role, NetworkRole::Client { .. }) {
            self.connection.send_packet(NetworkPacket::InputUpdate(vec![input_with_timestamp]));
        }

        // Process ticks
        while self.tick_timer >= self.tick_rate {
            self.tick_timer -= self.tick_rate;
            self.process_tick();
        }

        // Process network packets
        self.process_network_packets();

        // Handle prediction reconciliation
        self.reconcile_predictions();
    }

    fn process_tick(&mut self) {
        self.game_state.tick += 1;
        self.game_state.timestamp = self.current_time;

        // Gather inputs for this tick
        let mut tick_inputs = Vec::new();

        // Add local player input
        if let Some(local_input) = self.input_buffer.get_input_for_tick(self.game_state.tick) {
            tick_inputs.push(local_input.clone());
        } else if let Some(latest_input) = self.input_buffer.get_latest_input() {
            // Use latest input if no input for this tick
            let mut fallback_input = latest_input.clone();
            fallback_input.tick = self.game_state.tick;
            tick_inputs.push(fallback_input);
        }

        // Add remote player inputs (simplified - in real game would come from network)
        for entity in &self.game_state.entities {
            if entity.owner != Some(self.connection.local_player_id) {
                // Simulate remote input
                let mut remote_input = PlayerInput::new(entity.id, self.game_state.tick);
                remote_input.move_x = (rand::random::<f32>() - 0.5) * 2.0;
                remote_input.jump = rand::random::<f32>() < 0.05;
                tick_inputs.push(remote_input);
            }
        }

        self.game_state.inputs = tick_inputs;

        // Update game state
        self.update_game_state();

        // Store prediction for rollback
        self.prediction.add_prediction(self.game_state.clone());
    }

    fn update_game_state(&mut self) {
        for entity in &mut self.game_state.entities {
            // Find input for this entity
            if let Some(input) = self.game_state.inputs.iter().find(|i| i.player_id == entity.id) {
                // Apply input
                entity.velocity.x = input.move_x * 120.0;

                if input.jump && entity.position.y <= 60.0 {
                    entity.velocity.y = 180.0;
                }

                // Shoot projectile
                if input.shoot && matches!(self.connection.role, NetworkRole::Server) {
                    // Would create projectile entity
                }
            }

            // Apply physics
            entity.velocity.y -= 300.0 * self.tick_rate;

            // Update position
            entity.position += entity.velocity * self.tick_rate;

            // Boundary constraints
            entity.position.x = entity.position.x.clamp(0.0, 800.0);
            if entity.position.y < 50.0 {
                entity.position.y = 50.0;
                entity.velocity.y = 0.0;
            }
        }
    }

    fn process_network_packets(&mut self) {
        let packets = self.connection.get_incoming_packets();

        for packet in packets {
            match packet {
                NetworkPacket::StateUpdate(server_state) => {
                    // Server state update
                    if self.game_state.tick < server_state.tick {
                        // Need to rollback and replay
                        self.rollback_and_replay(server_state);
                    } else {
                        // Direct state application
                        self.apply_server_state(server_state);
                    }
                }
                NetworkPacket::InputUpdate(inputs) => {
                    if matches!(self.connection.role, NetworkRole::Server) {
                        // Server received client input
                        for input in inputs {
                            // Would validate and broadcast to other clients
                        }
                    }
                }
                NetworkPacket::Pong { timestamp } => {
                    self.connection.update_rtt(timestamp, self.current_time);
                }
                _ => {}
            }
        }

        // Simulate receiving server updates (in real networking, this would come from network)
        if matches!(self.connection.role, NetworkRole::Client { .. }) && rand::random::<f32>() < 0.1 {
            let simulated_server_state = self.game_state.clone();
            self.connection.receive_packet(NetworkPacket::StateUpdate(simulated_server_state));
        }
    }

    fn rollback_and_replay(&mut self, server_state: NetworkedGameState) {
        println!("🔄 Rolling back from tick {} to {}", self.game_state.tick, server_state.tick);

        // Apply server state
        self.game_state = server_state;

        // Clear predictions after this tick
        self.prediction.clear_predictions_after(self.game_state.tick);

        // Replay inputs from server tick onwards
        let start_tick = self.game_state.tick + 1;
        let end_tick = start_tick + 10; // Replay next 10 ticks

        for tick in start_tick..end_tick {
            self.game_state.tick = tick;
            self.process_tick();
        }
    }

    fn apply_server_state(&mut self, server_state: NetworkedGameState) {
        // Apply server state directly (for entities we don't own)
        for server_entity in &server_state.entities {
            if let Some(local_entity) = self.game_state.entities.iter_mut()
                .find(|e| e.id == server_entity.id && e.owner != Some(self.connection.local_player_id)) {
                *local_entity = server_entity.clone();
            }
        }
    }

    fn reconcile_predictions(&mut self) {
        // Check if our predictions match server state
        // In a real implementation, this would compare with received server updates
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sorta Working Networking - Client-Server Architecture".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(SortaWorkingNetworkingDemo::new(true)) // Start as server
        .add_systems(Startup, setup_networking_demo)
        .add_systems(Update, (
            handle_networking_input,
            update_networking_simulation,
            render_networking_demo,
            display_networking_info,
        ).chain())
        .run();
}

fn setup_networking_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_networking_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<SortaWorkingNetworkingDemo>,
) {
    let mut input = PlayerInput::new(demo.connection.local_player_id, demo.game_state.tick + 1);

    // Movement
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.move_x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.move_x = 1.0;
    }

    input.jump = keyboard_input.just_pressed(KeyCode::Space);
    input.shoot = keyboard_input.just_pressed(KeyCode::KeyF);

    demo.update(1.0 / 60.0, input);

    // Switch between client and server
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        let is_server = matches!(demo.connection.role, NetworkRole::Server);
        *demo = SortaWorkingNetworkingDemo::new(!is_server);
        println!("Switched to {}", if is_server { "Client" } else { "Server" });
    }
}

fn update_networking_simulation(time: Res<Time>, mut demo: ResMut<SortaWorkingNetworkingDemo>) {
    // Updates are handled in input system
}

fn render_networking_demo(
    mut commands: Commands,
    mut entity_entities: Local<Vec<Entity>>,
    demo: Res<SortaWorkingNetworkingDemo>,
) {
    // Clear previous frame
    for entity in entity_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render entities
    for entity in &demo.game_state.entities {
        let color = if entity.owner == Some(demo.connection.local_player_id) {
            Color::rgb(0.2, 0.8, 0.2) // Local player - green
        } else {
            Color::rgb(0.2, 0.4, 0.8) // Remote player - blue
        };

        let entity_bundle = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(20.0, 30.0)),
                    ..default()
                },
                transform: Transform::from_xyz(entity.position.x, entity.position.y, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    format!("P{}: {:.0}", entity.id + 1, entity.health),
                    TextStyle {
                        font_size: 10.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 18.0, 2.0),
                ..default()
            },
        )).id();
        entity_entities.push(entity_bundle);
    }

    // Render network role indicator
    let role_text = match demo.connection.role {
        NetworkRole::Server => "SERVER",
        NetworkRole::Client { .. } => "CLIENT",
    };

    commands.spawn(Text2dBundle {
        text: Text::from_section(
            role_text,
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        transform: Transform::from_xyz(350.0, 350.0, 3.0),
        ..default()
    });
}

fn display_networking_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<SortaWorkingNetworkingDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        println!("\n=== Sorta Working Networking Demo ===");
        println!("Role: {:?}", demo.connection.role);
        println!("Local Player ID: {}", demo.connection.local_player_id);
        println!("Current Tick: {}", demo.game_state.tick);
        println!("RTT: {:.0}ms", demo.connection.rtt * 1000.0);
        println!("Packet Loss: {:.1}%", demo.connection.packet_loss * 100.0);
        println!("Input Buffer Size: {}", demo.input_buffer.inputs.len());
        println!("Prediction Buffer Size: {}", demo.prediction.predicted_states.len());
        println!("Outgoing Packets: {}", demo.connection.outgoing_packets.len());
        println!("Incoming Packets: {}", demo.connection.incoming_packets.len());

        println!("\nEntities:");
        for entity in &demo.game_state.entities {
            println!("  ID: {} | Pos: ({:.1}, {:.1}) | HP: {:.0} | Owner: {:?}",
                    entity.id, entity.position.x, entity.position.y, entity.health, entity.owner);
        }

        println!("\nControls:");
        println!("  A/D: Move | Space: Jump | F: Shoot");
        println!("  T: Toggle Server/Client");
        println!("  I: Show this info");
        println!("======================\n");
    }
}
