// Example: Networking Stats
// Based on "Networking Stats" blog post
// https://www.slowrush.dev/news/networking-stats

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::VecDeque;

// Networking statistics and monitoring
// Demonstrates network performance metrics, latency tracking, and optimization

#[derive(Clone, Debug)]
struct NetworkStats {
    total_packets_sent: u64,
    total_packets_received: u64,
    total_bytes_sent: u64,
    total_bytes_received: u64,
    packets_lost: u64,
    average_rtt: f64,
    min_rtt: f64,
    max_rtt: f64,
    jitter: f64,
    packets_per_second: f32,
    bytes_per_second: f32,
    compression_ratio: f32,
    last_update_time: f64,
    rtt_samples: VecDeque<f64>,
}

impl NetworkStats {
    fn new() -> Self {
        Self {
            total_packets_sent: 0,
            total_packets_received: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            packets_lost: 0,
            average_rtt: 0.0,
            min_rtt: f64::INFINITY,
            max_rtt: 0.0,
            jitter: 0.0,
            packets_per_second: 0.0,
            bytes_per_second: 0.0,
            compression_ratio: 1.0,
            last_update_time: 0.0,
            rtt_samples: VecDeque::with_capacity(100),
        }
    }

    fn record_packet_sent(&mut self, size_bytes: u64) {
        self.total_packets_sent += 1;
        self.total_bytes_sent += size_bytes;
    }

    fn record_packet_received(&mut self, size_bytes: u64) {
        self.total_packets_received += 1;
        self.total_bytes_received += size_bytes;
    }

    fn record_packet_lost(&mut self) {
        self.packets_lost += 1;
    }

    fn record_rtt(&mut self, rtt: f64) {
        self.rtt_samples.push_back(rtt);

        if self.rtt_samples.len() > 100 {
            self.rtt_samples.pop_front();
        }

        // Update min/max
        self.min_rtt = self.min_rtt.min(rtt);
        self.max_rtt = self.max_rtt.max(rtt);

        // Calculate average
        let sum: f64 = self.rtt_samples.iter().sum();
        self.average_rtt = sum / self.rtt_samples.len() as f64;

        // Calculate jitter (variation in RTT)
        if self.rtt_samples.len() > 1 {
            let variance = self.rtt_samples.iter()
                .map(|&sample| (sample - self.average_rtt).powi(2))
                .sum::<f64>() / (self.rtt_samples.len() - 1) as f64;
            self.jitter = variance.sqrt();
        }
    }

    fn update_rates(&mut self, current_time: f64, dt: f64) {
        if dt > 0.0 {
            let time_diff = current_time - self.last_update_time;
            if time_diff >= 1.0 { // Update rates every second
                self.packets_per_second = self.total_packets_sent as f32 / time_diff as f32;
                self.bytes_per_second = self.total_bytes_sent as f32 / time_diff as f32;

                self.last_update_time = current_time;

                // Reset counters for next measurement period
                self.total_packets_sent = 0;
                self.total_bytes_sent = 0;
                self.total_packets_received = 0;
                self.total_bytes_received = 0;
            }
        }
    }

    fn get_packet_loss_rate(&self) -> f32 {
        let total_packets = self.total_packets_sent + self.total_packets_received + self.packets_lost;
        if total_packets > 0 {
            self.packets_lost as f32 / total_packets as f32 * 100.0
        } else {
            0.0
        }
    }

    fn get_bandwidth_usage(&self) -> String {
        let bytes_per_sec = self.bytes_per_second;
        if bytes_per_sec >= 1024.0 * 1024.0 {
            format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
        } else if bytes_per_sec >= 1024.0 {
            format!("{:.1} KB/s", bytes_per_sec / 1024.0)
        } else {
            format!("{:.0} B/s", bytes_per_sec)
        }
    }
}

#[derive(Clone, Debug)]
struct NetworkPacket {
    id: u64,
    size_bytes: u64,
    send_time: f64,
    receive_time: Option<f64>,
    packet_type: PacketType,
    compressed: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum PacketType {
    PositionUpdate,
    InputUpdate,
    StateSync,
    Ping,
    Pong,
    ChatMessage,
    EntitySpawn,
    EntityDestroy,
}

impl NetworkPacket {
    fn new(id: u64, packet_type: PacketType, size_bytes: u64, send_time: f64) -> Self {
        Self {
            id,
            size_bytes,
            send_time,
            receive_time: None,
            packet_type,
            compressed: false,
        }
    }

    fn mark_received(&mut self, receive_time: f64) {
        self.receive_time = Some(receive_time);
    }

    fn get_rtt(&self) -> Option<f64> {
        self.receive_time.map(|rt| rt - self.send_time)
    }
}

#[derive(Clone, Debug)]
struct NetworkOptimizer {
    compression_enabled: bool,
    delta_encoding: bool,
    interpolation_enabled: bool,
    prediction_enabled: bool,
    adaptive_rate_control: bool,
    target_packet_rate: f32,
    current_packet_rate: f32,
}

impl NetworkOptimizer {
    fn new() -> Self {
        Self {
            compression_enabled: true,
            delta_encoding: true,
            interpolation_enabled: true,
            prediction_enabled: true,
            adaptive_rate_control: true,
            target_packet_rate: 30.0, // 30 packets per second
            current_packet_rate: 30.0,
        }
    }

    fn optimize_packet(&self, packet: &mut NetworkPacket) {
        if self.compression_enabled {
            // Simulate compression (reduce size by 30%)
            packet.size_bytes = (packet.size_bytes as f32 * 0.7) as u64;
            packet.compressed = true;
        }
    }

    fn should_send_packet(&self, dt: f32) -> bool {
        if self.adaptive_rate_control {
            let target_interval = 1.0 / self.target_packet_rate;
            rand::random::<f32>() < (dt / target_interval).min(1.0)
        } else {
            true
        }
    }

    fn adjust_rate_based_on_stats(&mut self, stats: &NetworkStats) {
        if self.adaptive_rate_control {
            // Increase rate if latency is low and packet loss is acceptable
            if stats.average_rtt < 0.05 && stats.get_packet_loss_rate() < 2.0 {
                self.target_packet_rate = (self.target_packet_rate * 1.1).min(60.0);
            }
            // Decrease rate if latency is high or packet loss is significant
            else if stats.average_rtt > 0.15 || stats.get_packet_loss_rate() > 5.0 {
                self.target_packet_rate = (self.target_packet_rate * 0.9).max(10.0);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct NetworkMonitor {
    packets_in_flight: Vec<NetworkPacket>,
    next_packet_id: u64,
    stats: NetworkStats,
    optimizer: NetworkOptimizer,
    simulated_latency: f64,
    simulated_packet_loss: f32,
}

impl NetworkMonitor {
    fn new() -> Self {
        Self {
            packets_in_flight: Vec::new(),
            next_packet_id: 0,
            stats: NetworkStats::new(),
            optimizer: NetworkOptimizer::new(),
            simulated_latency: 0.1, // 100ms
            simulated_packet_loss: 0.02, // 2%
        }
    }

    fn send_packet(&mut self, packet_type: PacketType, base_size: u64, current_time: f64) {
        let mut packet = NetworkPacket::new(
            self.next_packet_id,
            packet_type,
            base_size,
            current_time,
        );

        self.optimizer.optimize_packet(&mut packet);
        self.stats.record_packet_sent(packet.size_bytes);

        // Simulate network conditions
        if rand::random::<f32>() >= self.simulated_packet_loss {
            // Packet will be "received" after latency
            let receive_time = current_time + self.simulated_latency + (rand::random::<f64>() - 0.5) * 0.02; // Add jitter
            packet.receive_time = Some(receive_time);
            self.stats.record_packet_received(packet.size_bytes);

            // Record RTT
            if let Some(rtt) = packet.get_rtt() {
                self.stats.record_rtt(rtt);
            }
        } else {
            self.stats.record_packet_lost();
        }

        self.packets_in_flight.push(packet);
        self.next_packet_id += 1;

        // Clean up old packets
        self.packets_in_flight.retain(|p| current_time - p.send_time < 10.0); // Keep 10 seconds of history
    }

    fn update(&mut self, current_time: f64, dt: f64) {
        self.stats.update_rates(current_time, dt);
        self.optimizer.adjust_rate_based_on_stats(&self.stats);

        // Process received packets
        self.packets_in_flight.retain(|packet| {
            if let Some(receive_time) = packet.receive_time {
                if receive_time <= current_time {
                    // Packet has been "received"
                    return false; // Remove from in-flight
                }
            }
            true // Keep in-flight
        });
    }

    fn get_connection_quality(&self) -> ConnectionQuality {
        let rtt = self.stats.average_rtt;
        let packet_loss = self.stats.get_packet_loss_rate();

        if rtt < 0.05 && packet_loss < 1.0 {
            ConnectionQuality::Excellent
        } else if rtt < 0.1 && packet_loss < 3.0 {
            ConnectionQuality::Good
        } else if rtt < 0.2 && packet_loss < 5.0 {
            ConnectionQuality::Fair
        } else {
            ConnectionQuality::Poor
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl ConnectionQuality {
    fn color(&self) -> Color {
        match self {
            ConnectionQuality::Excellent => Color::rgb(0.2, 0.8, 0.2),
            ConnectionQuality::Good => Color::rgb(0.8, 0.8, 0.2),
            ConnectionQuality::Fair => Color::rgb(0.8, 0.4, 0.2),
            ConnectionQuality::Poor => Color::rgb(0.8, 0.2, 0.2),
        }
    }

    fn description(&self) -> &'static str {
        match self {
            ConnectionQuality::Excellent => "Excellent",
            ConnectionQuality::Good => "Good",
            ConnectionQuality::Fair => "Fair",
            ConnectionQuality::Poor => "Poor",
        }
    }
}

#[derive(Resource)]
struct NetworkingStatsDemo {
    monitor: NetworkMonitor,
    current_time: f64,
    tick_timer: f32,
    packet_send_timer: f32,
}

impl NetworkingStatsDemo {
    fn new() -> Self {
        Self {
            monitor: NetworkMonitor::new(),
            current_time: 0.0,
            tick_timer: 0.0,
            packet_send_timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.current_time += dt as f64;
        self.tick_timer += dt;
        self.packet_send_timer += dt;

        // Send packets at regular intervals
        if self.packet_send_timer >= 1.0 / self.monitor.optimizer.target_packet_rate as f32 {
            self.packet_send_timer = 0.0;

            // Send different types of packets
            let packet_types = [
                PacketType::PositionUpdate,
                PacketType::InputUpdate,
                PacketType::StateSync,
                PacketType::Ping,
            ];

            let packet_type = packet_types[rand::random::<usize>() % packet_types.len()].clone();

            // Base packet sizes (uncompressed)
            let base_size = match packet_type {
                PacketType::PositionUpdate => 24, // Position + velocity
                PacketType::InputUpdate => 8,     // Input state
                PacketType::StateSync => 128,     // Game state
                PacketType::Ping => 4,            // Just timestamp
                _ => 16,
            };

            self.monitor.send_packet(packet_type, base_size, self.current_time);
        }

        self.monitor.update(self.current_time, dt as f64);
    }

    fn adjust_network_conditions(&mut self, latency_change: f64, packet_loss_change: f32) {
        self.monitor.simulated_latency = (self.monitor.simulated_latency + latency_change).max(0.01);
        self.monitor.simulated_packet_loss = (self.monitor.simulated_packet_loss + packet_loss_change).max(0.0).min(0.5);
        println!("Network conditions: {:.0}ms latency, {:.1}% packet loss",
                self.monitor.simulated_latency * 1000.0,
                self.monitor.simulated_packet_loss * 100.0);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Networking Stats - Performance Monitoring".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(NetworkingStatsDemo::new())
        .add_systems(Startup, setup_networking_stats_demo)
        .add_systems(Update, (
            handle_networking_stats_input,
            update_networking_stats_simulation,
            render_networking_stats_demo,
            display_networking_stats_info,
        ).chain())
        .run();
}

fn setup_networking_stats_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_networking_stats_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<NetworkingStatsDemo>,
) {
    demo.update(1.0 / 60.0);

    // Adjust network conditions
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        demo.adjust_network_conditions(-0.01, 0.0); // Decrease latency
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        demo.adjust_network_conditions(0.01, 0.0); // Increase latency
    }
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        demo.adjust_network_conditions(0.0, -0.005); // Decrease packet loss
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        demo.adjust_network_conditions(0.0, 0.005); // Increase packet loss
    }

    // Reset conditions
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        demo.monitor.simulated_latency = 0.1;
        demo.monitor.simulated_packet_loss = 0.02;
        println!("Network conditions reset");
    }
}

fn update_networking_stats_simulation(time: Res<Time>, mut demo: ResMut<NetworkingStatsDemo>) {
    // Updates are handled in input system
}

fn render_networking_stats_demo(
    mut commands: Commands,
    mut stat_entities: Local<Vec<Entity>>,
    demo: Res<NetworkingStatsDemo>,
) {
    // Clear previous frame
    for entity in stat_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    let quality = demo.monitor.get_connection_quality();
    let quality_color = quality.color();

    // Render connection quality indicator
    let quality_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: quality_color,
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(350.0, 250.0, 1.0),
            ..default()
        },

    )).id();
    stat_entities.push(quality_entity);

    // Render packet visualization
    let mut y_offset = 200.0;
    for packet in demo.monitor.packets_in_flight.iter().rev().take(10) {
        let alpha = if packet.receive_time.is_some() { 0.5 } else { 1.0 };
        let color = match packet.packet_type {
            PacketType::PositionUpdate => Color::rgba(0.2, 0.8, 0.2, alpha),
            PacketType::InputUpdate => Color::rgba(0.8, 0.2, 0.2, alpha),
            PacketType::StateSync => Color::rgba(0.2, 0.2, 0.8, alpha),
            PacketType::Ping => Color::rgba(0.8, 0.8, 0.2, alpha),
            _ => Color::rgba(0.5, 0.5, 0.5, alpha),
        };

        let packet_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(packet.size_bytes as f32 / 10.0, 8.0)),
                    ..default()
                },
                transform: Transform::from_xyz(100.0, y_offset, 1.0),
                ..default()
            },

        )).id();
        stat_entities.push(packet_entity);
        y_offset -= 12.0;
    }
}

fn display_networking_stats_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<NetworkingStatsDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        println!("\n=== Networking Stats Demo ===");
        println!("Connection Quality: {:?}", demo.monitor.get_connection_quality().description());
        println!("RTT: {:.1}ms (min: {:.1}ms, max: {:.1}ms)",
                demo.monitor.stats.average_rtt * 1000.0,
                demo.monitor.stats.min_rtt * 1000.0,
                demo.monitor.stats.max_rtt * 1000.0);
        println!("Jitter: {:.1}ms", demo.monitor.stats.jitter * 1000.0);
        println!("Packet Loss: {:.2}%", demo.monitor.stats.get_packet_loss_rate());
        println!("Packets/sec: {:.1}", demo.monitor.stats.packets_per_second);
        println!("Bandwidth: {}", demo.monitor.stats.get_bandwidth_usage());
        println!("Packets in Flight: {}", demo.monitor.packets_in_flight.len());
        println!("Compression Ratio: {:.2}", demo.monitor.stats.compression_ratio);

        println!("\nNetwork Conditions (simulated):");
        println!("Latency: {:.0}ms", demo.monitor.simulated_latency * 1000.0);
        println!("Packet Loss: {:.1}%", demo.monitor.simulated_packet_loss * 100.0);

        println!("\nOptimization Settings:");
        println!("Compression: {}", demo.monitor.optimizer.compression_enabled);
        println!("Delta Encoding: {}", demo.monitor.optimizer.delta_encoding);
        println!("Interpolation: {}", demo.monitor.optimizer.interpolation_enabled);
        println!("Prediction: {}", demo.monitor.optimizer.prediction_enabled);
        println!("Adaptive Rate: {}", demo.monitor.optimizer.adaptive_rate_control);
        println!("Target Packet Rate: {:.1} Hz", demo.monitor.optimizer.target_packet_rate);

        println!("\nControls:");
        println!("  ↑/↓: Adjust latency");
        println!("  ←/→: Adjust packet loss");
        println!("  R: Reset conditions");
        println!("  I: Show this info");
        println!("======================\n");
    }
}
