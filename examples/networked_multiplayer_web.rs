// Example: Networked Multiplayer on the Web
// Based on "Networked Multiplayer on the Web" blog post
// https://www.slowrush.dev/news/networked-multiplayer-on-the-web

use std::collections::VecDeque;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Web-based networked multiplayer
// Demonstrates browser networking, WebRTC, and web-specific optimizations

#[derive(Clone, Debug)]
enum WebTransportType {
    WebRTC,
    WebSocket,
    WebTransport, // Future API
}

#[derive(Clone, Debug)]
struct WebRTCConnection {
    peer_id: String,
    connection_state: RTCConnectionState,
    data_channel: Option<WebRTCDataChannel>,
    local_description: Option<String>,
    remote_description: Option<String>,
    ice_candidates: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
enum RTCConnectionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

#[derive(Clone, Debug)]
struct WebRTCDataChannel {
    label: String,
    id: u16,
    ready_state: DataChannelState,
    buffered_amount: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum DataChannelState {
    Connecting,
    Open,
    Closing,
    Closed,
}

#[derive(Clone, Debug)]
struct WebSocketConnection {
    url: String,
    ready_state: WebSocketState,
    buffered_amount: usize,
    last_ping: f64,
    ping_interval: f64,
}

#[derive(Clone, Debug, PartialEq)]
enum WebSocketState {
    Connecting,
    Open,
    Closing,
    Closed,
}

#[derive(Clone, Debug)]
enum NetworkMessage {
    PlayerUpdate {
        player_id: String,
        position: Vec2,
        velocity: Vec2,
        timestamp: f64,
    },
    GameState {
        tick: u32,
        entities: Vec<EntityState>,
        timestamp: f64,
    },
    Chat {
        player_id: String,
        message: String,
        timestamp: f64,
    },
    Ping {
        timestamp: f64,
    },
    Pong {
        timestamp: f64,
    },
    JoinGame {
        player_name: String,
    },
    LeaveGame {
        player_id: String,
    },
}

#[derive(Clone, Debug)]
struct EntityState {
    id: String,
    entity_type: String,
    position: Vec2,
    health: f32,
}

#[derive(Clone, Debug)]
struct WebNetworkManager {
    transport_type: WebTransportType,
    webrtc_connections: Vec<WebRTCConnection>,
    websocket_fallback: Option<WebSocketConnection>,
    local_player_id: String,
    game_room_id: String,
    is_host: bool,
    message_queue: VecDeque<NetworkMessage>,
    sent_messages: Vec<(NetworkMessage, f64)>, // Message and send time
    received_messages: Vec<NetworkMessage>,
    rtt_measurements: Vec<f64>,
    bandwidth_up_kbps: f32,
    bandwidth_down_kbps: f32,
}

impl WebNetworkManager {
    fn new(transport_type: WebTransportType, local_player_id: String, is_host: bool) -> Self {
        Self {
            transport_type,
            webrtc_connections: Vec::new(),
            websocket_fallback: None,
            local_player_id,
            game_room_id: "default_room".to_string(),
            is_host,
            message_queue: VecDeque::new(),
            sent_messages: Vec::new(),
            received_messages: Vec::new(),
            rtt_measurements: Vec::new(),
            bandwidth_up_kbps: 0.0,
            bandwidth_down_kbps: 0.0,
        }
    }

    fn connect_to_room(&mut self, room_id: String) {
        self.game_room_id = room_id.clone();

        match self.transport_type {
            WebTransportType::WebRTC => {
                // In real WebRTC, this would:
                // 1. Create RTCPeerConnection
                // 2. Create data channel
                // 3. Generate offer/answer
                // 4. Exchange via signaling server

                println!("Attempting WebRTC connection to room: {}", room_id);

                // Simulate connection establishment
                let mut connection = WebRTCConnection {
                    peer_id: "remote_player_1".to_string(),
                    connection_state: RTCConnectionState::Connecting,
                    data_channel: Some(WebRTCDataChannel {
                        label: "game_data".to_string(),
                        id: 1,
                        ready_state: DataChannelState::Connecting,
                        buffered_amount: 0,
                    }),
                    local_description: None,
                    remote_description: None,
                    ice_candidates: Vec::new(),
                };

                // Simulate quick connection
                connection.connection_state = RTCConnectionState::Connected;
                if let Some(channel) = &mut connection.data_channel {
                    channel.ready_state = DataChannelState::Open;
                }

                self.webrtc_connections.push(connection);
                println!("WebRTC connection established!");
            }
            WebTransportType::WebSocket => {
                // Fallback to WebSocket
                let ws_url = format!("wss://game-server.example.com/room/{}", room_id);
                self.websocket_fallback = Some(WebSocketConnection {
                    url: ws_url,
                    ready_state: WebSocketState::Connecting,
                    buffered_amount: 0,
                    last_ping: 0.0,
                    ping_interval: 5.0,
                });

                // Simulate connection
                if let Some(ws) = &mut self.websocket_fallback {
                    ws.ready_state = WebSocketState::Open;
                }
                println!("WebSocket fallback connection established!");
            }
            WebTransportType::WebTransport => {
                println!("WebTransport not yet implemented in this demo");
            }
        }
    }

    fn send_message(&mut self, message: NetworkMessage, current_time: f64) {
        // Add to sent messages for RTT tracking
        if matches!(message, NetworkMessage::Ping { .. }) {
            self.sent_messages.push((message.clone(), current_time));
        }

        // Serialize message (simplified)
        let message_size = self.estimate_message_size(&message);

        // Update bandwidth stats
        self.bandwidth_up_kbps += message_size as f32 / 1024.0 * 60.0; // Rough estimate

        self.message_queue.push_back(message);
    }

    fn receive_message(&mut self, message: NetworkMessage, current_time: f64) {
        // Update bandwidth stats
        let message_size = self.estimate_message_size(&message);
        self.bandwidth_down_kbps += message_size as f32 / 1024.0 * 60.0;

        // Handle ping/pong for RTT
        if let NetworkMessage::Pong { timestamp } = message {
            if let Some((NetworkMessage::Ping { .. }, ping_time)) = self.sent_messages
                .iter()
                .find(|(msg, _)| matches!(msg, NetworkMessage::Ping { .. })) {
                let rtt = current_time - ping_time;
                self.rtt_measurements.push(rtt);
                if self.rtt_measurements.len() > 10 {
                    self.rtt_measurements.remove(0);
                }
            }
        }

        self.received_messages.push(message);
    }

    fn update(&mut self, current_time: f64, dt: f64) {
        // Handle ping/pong for WebSocket connections
        if let Some(ws) = &mut self.websocket_fallback {
            if matches!(ws.ready_state, WebSocketState::Open) {
                if current_time - ws.last_ping > ws.ping_interval {
                    ws.last_ping = current_time;
                    self.send_message(NetworkMessage::Ping { timestamp: current_time }, current_time);
                }
            }
        }

        // Simulate receiving messages
        self.simulate_incoming_messages(current_time);

        // Clean up old sent messages
        self.sent_messages.retain(|(_, send_time)| current_time - send_time < 30.0);

        // Decay bandwidth stats
        self.bandwidth_up_kbps *= 0.95;
        self.bandwidth_down_kbps *= 0.95;
    }

    fn simulate_incoming_messages(&mut self, current_time: f64) {
        // Simulate receiving messages from remote players
        if rand::random::<f32>() < 0.3 { // 30% chance per update
            let message = match rand::random::<u32>() % 4 {
                0 => NetworkMessage::PlayerUpdate {
                    player_id: "remote_player_1".to_string(),
                    position: Vec2::new(
                        rand::random::<f32>() * 400.0 + 200.0,
                        rand::random::<f32>() * 300.0 + 150.0,
                    ),
                    velocity: Vec2::new(
                        (rand::random::<f32>() - 0.5) * 100.0,
                        (rand::random::<f32>() - 0.5) * 100.0,
                    ),
                    timestamp: current_time,
                },
                1 => NetworkMessage::Chat {
                    player_id: "remote_player_1".to_string(),
                    message: "Hello from the web!".to_string(),
                    timestamp: current_time,
                },
                2 => NetworkMessage::Pong { timestamp: current_time - 0.05 }, // Simulate 50ms RTT
                _ => NetworkMessage::GameState {
                    tick: (current_time * 60.0) as u32,
                    entities: vec![
                        EntityState {
                            id: "entity_1".to_string(),
                            entity_type: "player".to_string(),
                            position: Vec2::new(300.0, 200.0),
                            health: 100.0,
                        }
                    ],
                    timestamp: current_time,
                },
            };

            self.receive_message(message, current_time);
        }
    }

    fn estimate_message_size(&self, message: &NetworkMessage) -> usize {
        match message {
            NetworkMessage::PlayerUpdate { .. } => 48, // position + velocity + timestamp + id
            NetworkMessage::GameState { entities, .. } => 16 + entities.len() * 32, // header + entity data
            NetworkMessage::Chat { message, .. } => 32 + message.len(), // header + message
            NetworkMessage::Ping { .. } | NetworkMessage::Pong { .. } => 16,
            NetworkMessage::JoinGame { player_name } => 16 + player_name.len(),
            NetworkMessage::LeaveGame { .. } => 16,
        }
    }

    fn get_average_rtt(&self) -> f64 {
        if self.rtt_measurements.is_empty() {
            0.0
        } else {
            self.rtt_measurements.iter().sum::<f64>() / self.rtt_measurements.len() as f64
        }
    }

    fn get_connection_quality(&self) -> WebConnectionQuality {
        let avg_rtt = self.get_average_rtt();

        if self.webrtc_connections.iter().any(|c| c.connection_state == RTCConnectionState::Connected) {
            if avg_rtt < 0.05 {
                WebConnectionQuality::Excellent
            } else if avg_rtt < 0.1 {
                WebConnectionQuality::Good
            } else if avg_rtt < 0.2 {
                WebConnectionQuality::Fair
            } else {
                WebConnectionQuality::Poor
            }
        } else if let Some(ws) = &self.websocket_fallback {
            if ws.ready_state == WebSocketState::Open {
                WebConnectionQuality::WebSocketFallback
            } else {
                WebConnectionQuality::Disconnected
            }
        } else {
            WebConnectionQuality::Disconnected
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum WebConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    WebSocketFallback,
    Disconnected,
}

impl WebConnectionQuality {
    fn description(&self) -> &'static str {
        match self {
            WebConnectionQuality::Excellent => "Excellent (WebRTC)",
            WebConnectionQuality::Good => "Good (WebRTC)",
            WebConnectionQuality::Fair => "Fair (WebRTC)",
            WebConnectionQuality::Poor => "Poor (WebRTC)",
            WebConnectionQuality::WebSocketFallback => "WebSocket Fallback",
            WebConnectionQuality::Disconnected => "Disconnected",
        }
    }

    fn color(&self) -> Color {
        match self {
            WebConnectionQuality::Excellent => Color::rgb(0.2, 0.8, 0.2),
            WebConnectionQuality::Good => Color::rgb(0.6, 0.8, 0.2),
            WebConnectionQuality::Fair => Color::rgb(0.8, 0.6, 0.2),
            WebConnectionQuality::Poor => Color::rgb(0.8, 0.2, 0.2),
            WebConnectionQuality::WebSocketFallback => Color::rgb(0.4, 0.4, 0.8),
            WebConnectionQuality::Disconnected => Color::rgb(0.3, 0.3, 0.3),
        }
    }
}

#[derive(Resource)]
struct WebMultiplayerDemo {
    network_manager: WebNetworkManager,
    local_player: PlayerState,
    remote_players: Vec<PlayerState>,
    chat_messages: VecDeque<(String, String, f64)>, // (player_id, message, timestamp)
    current_time: f64,
}

#[derive(Clone, Debug)]
struct PlayerState {
    id: String,
    name: String,
    position: Vec2,
    velocity: Vec2,
    health: f32,
    last_update: f64,
}

impl PlayerState {
    fn new(id: String, name: String, position: Vec2) -> Self {
        Self {
            id,
            name,
            position,
            velocity: Vec2::ZERO,
            health: 100.0,
            last_update: 0.0,
        }
    }
}

impl WebMultiplayerDemo {
    fn new() -> Self {
        let local_player_id = format!("player_{}", rand::random::<u32>());
        let network_manager = WebNetworkManager::new(
            WebTransportType::WebRTC,
            local_player_id.clone(),
            false, // Start as client
        );

        let local_player = PlayerState::new(
            local_player_id,
            "Local Player".to_string(),
            Vec2::new(200.0, 300.0),
        );

        Self {
            network_manager,
            local_player,
            remote_players: Vec::new(),
            chat_messages: VecDeque::new(),
            current_time: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.current_time += dt as f64;

        // Update local player physics
        self.local_player.velocity.y -= 300.0 * dt;
        self.local_player.position += self.local_player.velocity * dt;

        // Boundary constraints
        if self.local_player.position.y < 50.0 {
            self.local_player.position.y = 50.0;
            self.local_player.velocity.y = 0.0;
        }
        self.local_player.position.x = self.local_player.position.x.clamp(0.0, 800.0);

        // Send player updates
        if rand::random::<f32>() < 0.5 { // Send updates 50% of the time
            let update = NetworkMessage::PlayerUpdate {
                player_id: self.local_player.id.clone(),
                position: self.local_player.position,
                velocity: self.local_player.velocity,
                timestamp: self.current_time,
            };
            self.network_manager.send_message(update, self.current_time);
        }

        // Update network manager
        self.network_manager.update(self.current_time, dt as f64);

        // Process received messages
        let received_messages = self.network_manager.received_messages.clone();
        self.network_manager.received_messages.clear();

        for message in received_messages {
            self.process_network_message(message);
        }
    }

    fn process_network_message(&mut self, message: NetworkMessage) {
        match message {
            NetworkMessage::PlayerUpdate { player_id, position, velocity, .. } => {
                if player_id != self.local_player.id {
                    // Update or create remote player
                    if let Some(remote_player) = self.remote_players.iter_mut()
                        .find(|p| p.id == player_id) {
                        remote_player.position = position;
                        remote_player.velocity = velocity;
                        remote_player.last_update = self.current_time;
                    } else {
                        let new_player = PlayerState::new(
                            player_id.clone(),
                            format!("Remote Player {}", self.remote_players.len() + 1),
                            position,
                        );
                        self.remote_players.push(new_player);
                        println!("New remote player joined: {}", player_id);
                    }
                }
            }
            NetworkMessage::Chat { player_id, message, .. } => {
                self.chat_messages.push_back((player_id, message, self.current_time));
                if self.chat_messages.len() > 10 {
                    self.chat_messages.pop_front();
                }
            }
            NetworkMessage::GameState { entities, .. } => {
                // Update game state from server
                for entity in entities {
                    // In a real game, this would update the game world
                    println!("Entity update: {} at ({:.1}, {:.1})",
                            entity.id, entity.position.x, entity.position.y);
                }
            }
            _ => {}
        }
    }

    fn send_chat_message(&mut self, message: String) {
        let chat_msg = NetworkMessage::Chat {
            player_id: self.local_player.id.clone(),
            message,
            timestamp: self.current_time,
        };
        self.network_manager.send_message(chat_msg, self.current_time);
    }

    fn connect_to_game(&mut self, room_id: String) {
        self.network_manager.connect_to_room(room_id);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Networked Multiplayer on the Web - WebRTC & WebSocket".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(WebMultiplayerDemo::new())
        .add_systems(Startup, setup_web_multiplayer_demo)
        .add_systems(Update, (
            handle_web_multiplayer_input,
            update_web_multiplayer_simulation,
            render_web_multiplayer_demo,
            display_web_multiplayer_info,
        ).chain())
        .run();
}

fn setup_web_multiplayer_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_web_multiplayer_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<WebMultiplayerDemo>,
) {
    demo.update(1.0 / 60.0);

    // Player movement
    if keyboard_input.pressed(KeyCode::KeyA) {
        demo.local_player.velocity.x = -120.0;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        demo.local_player.velocity.x = 120.0;
    } else {
        demo.local_player.velocity.x *= 0.8; // Friction
    }

    if keyboard_input.just_pressed(KeyCode::Space) && demo.local_player.position.y <= 55.0 {
        demo.local_player.velocity.y = 180.0;
    }

    // Connect to game
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        demo.connect_to_game("web_demo_room".to_string());
    }

    // Send chat message
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        demo.send_chat_message("Hello from Bevy Web!".to_string());
    }

    // Switch transport type
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        demo.network_manager.transport_type = match demo.network_manager.transport_type {
            WebTransportType::WebRTC => WebTransportType::WebSocket,
            WebTransportType::WebSocket => WebTransportType::WebRTC,
            WebTransportType::WebTransport => WebTransportType::WebRTC,
        };
        println!("Switched to {:?}", demo.network_manager.transport_type);
    }
}

fn update_web_multiplayer_simulation(time: Res<Time>, mut demo: ResMut<WebMultiplayerDemo>) {
    // Updates are handled in input system
}

fn render_web_multiplayer_demo(
    mut commands: Commands,
    mut player_entities: Local<Vec<Entity>>,
    mut chat_entities: Local<Vec<Entity>>,
    demo: Res<WebMultiplayerDemo>,
) {
    // Clear previous frame
    for entity in player_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in chat_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render local player
    let local_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2), // Green for local
                custom_size: Some(Vec2::new(20.0, 30.0)),
                ..default()
            },
            transform: Transform::from_xyz(demo.local_player.position.x, demo.local_player.position.y, 1.0),
            ..default()
        },
        Text2dBundle {
            text: Text::from_section(
                &demo.local_player.name,
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
    player_entities.push(local_entity);

    // Render remote players
    for remote_player in &demo.remote_players {
        let remote_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.4, 0.8), // Blue for remote
                    custom_size: Some(Vec2::new(20.0, 30.0)),
                    ..default()
                },
                transform: Transform::from_xyz(remote_player.position.x, remote_player.position.y, 1.0),
                ..default()
            },

        )).id();
        player_entities.push(remote_entity);
    }

    // Render connection status
    let quality = demo.network_manager.get_connection_quality();
    let status_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: quality.color(),
                custom_size: Some(Vec2::new(40.0, 40.0)),
                ..default()
            },
            transform: Transform::from_xyz(700.0, 550.0, 2.0),
            ..default()
        },

    )).id();
    player_entities.push(status_entity);

    // Render recent chat messages
    let mut y_offset = 100.0;
    for (player_id, message, _) in demo.chat_messages.iter().rev().take(5) {
        let chat_entity = commands.spawn(Text2dBundle {
            text: Text::from_section(
                format!("{}: {}", player_id, message),
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(50.0, y_offset, 3.0),
            ..default()
        }).id();
        chat_entities.push(chat_entity);
        y_offset += 20.0;
    }
}

fn display_web_multiplayer_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<WebMultiplayerDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        println!("\n=== Web Multiplayer Demo ===");
        println!("Transport Type: {:?}", demo.network_manager.transport_type);
        println!("Local Player: {} ({})", demo.local_player.name, demo.local_player.id);
        println!("Remote Players: {}", demo.remote_players.len());
        println!("Connection Quality: {}", demo.network_manager.get_connection_quality().description());
        println!("RTT: {:.0}ms", demo.network_manager.get_average_rtt() * 1000.0);
        println!("Upload: {:.1} KB/s", demo.network_manager.bandwidth_up_kbps);
        println!("Download: {:.1} KB/s", demo.network_manager.bandwidth_down_kbps);
        println!("Room ID: {}", demo.network_manager.game_room_id);
        println!("Is Host: {}", demo.network_manager.is_host);

        println!("\nWebRTC Connections:");
        for connection in &demo.network_manager.webrtc_connections {
            println!("  {}: {:?}", connection.peer_id, connection.connection_state);
        }

        if let Some(ws) = &demo.network_manager.websocket_fallback {
            println!("WebSocket: {:?}", ws.ready_state);
        }

        println!("\nRecent Chat Messages:");
        for (player_id, message, timestamp) in demo.chat_messages.iter().rev().take(3) {
            println!("  [{}] {}: {}", timestamp, player_id, message);
        }

        println!("\nControls:");
        println!("  A/D: Move | Space: Jump");
        println!("  C: Connect to game room");
        println!("  T: Send chat message");
        println!("  S: Switch transport (WebRTC/WebSocket)");
        println!("  I: Show this info");
        println!("\nConnection indicator colors:");
        println!("  Green: Excellent WebRTC | Blue: Good WebRTC");
        println!("  Orange: Fair WebRTC | Red: Poor WebRTC");
        println!("  Purple: WebSocket Fallback | Gray: Disconnected");
        println!("======================\n");
    }
}
