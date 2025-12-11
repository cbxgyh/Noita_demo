// Example: Bad Networks
// Based on "Bad Networks" blog post
// https://www.slowrush.dev/news/bad-networks

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::VecDeque;
use std::ops::Add;
// Handling bad network conditions
// Demonstrates network resilience, error correction, and adaptive strategies

#[derive(Clone, Debug)]
struct NetworkCondition {
    latency_ms: f64,
    jitter_ms: f64,
    packet_loss_percent: f32,
    duplication_percent: f32,
    corruption_percent: f32,
    out_of_order_percent: f32,
    bandwidth_kbps: f32,
}

impl NetworkCondition {
    fn excellent() -> Self {
        Self {
            latency_ms: 20.0,
            jitter_ms: 2.0,
            packet_loss_percent: 0.1,
            duplication_percent: 0.0,
            corruption_percent: 0.0,
            out_of_order_percent: 0.0,
            bandwidth_kbps: 1000.0,
        }
    }

    fn good() -> Self {
        Self {
            latency_ms: 50.0,
            jitter_ms: 5.0,
            packet_loss_percent: 0.5,
            duplication_percent: 0.1,
            corruption_percent: 0.1,
            out_of_order_percent: 1.0,
            bandwidth_kbps: 500.0,
        }
    }

    fn poor() -> Self {
        Self {
            latency_ms: 200.0,
            jitter_ms: 50.0,
            packet_loss_percent: 5.0,
            duplication_percent: 1.0,
            corruption_percent: 2.0,
            out_of_order_percent: 10.0,
            bandwidth_kbps: 100.0,
        }
    }

    fn terrible() -> Self {
        Self {
            latency_ms: 500.0,
            jitter_ms: 200.0,
            packet_loss_percent: 15.0,
            duplication_percent: 5.0,
            corruption_percent: 10.0,
            out_of_order_percent: 30.0,
            bandwidth_kbps: 20.0,
        }
    }

    fn satellite() -> Self {
        Self {
            latency_ms: 600.0,
            jitter_ms: 100.0,
            packet_loss_percent: 2.0,
            duplication_percent: 0.5,
            corruption_percent: 0.5,
            out_of_order_percent: 5.0,
            bandwidth_kbps: 50.0,
        }
    }

    fn mobile_4g() -> Self {
        Self {
            latency_ms: 80.0,
            jitter_ms: 20.0,
            packet_loss_percent: 3.0,
            duplication_percent: 0.5,
            corruption_percent: 1.0,
            out_of_order_percent: 8.0,
            bandwidth_kbps: 200.0,
        }
    }
}

#[derive(Clone, Debug)]
struct NetworkPacket {
    id: u64,
    sequence_number: u32,
    data: Vec<u8>,
    send_time: f64,
    size_bytes: usize,
    requires_ack: bool,
    priority: PacketPriority,
}

#[derive(Copy,Clone, Debug, PartialEq)]
enum PacketPriority {
    Critical,    // State updates, important inputs
    Important,   // Regular updates
    Optional,    // Cosmetic data, can be dropped
}

impl NetworkPacket {
    fn new(id: u64, sequence_number: u32, data: Vec<u8>, priority: PacketPriority) -> Self {
        let len =data.len();
        Self {
            id,
            sequence_number,
            data,
            send_time: 0.0,
            size_bytes: len,
            requires_ack: matches!(priority, PacketPriority::Critical),
            priority,
        }
    }
}

#[derive(Clone, Debug)]
struct PacketDelivery {
    packet: NetworkPacket,
    delivery_time: f64,
    corrupted: bool,
    duplicated: bool,
}

#[derive(Clone, Debug)]
struct ReliabilitySystem {
    sent_packets: VecDeque<NetworkPacket>,
    received_sequences: std::collections::HashSet<u32>,
    ack_waiting_list: Vec<u64>,
    resend_queue: VecDeque<NetworkPacket>,
    max_resend_attempts: u32,
    rtt_estimate: f64,
    congestion_window: usize,
}

impl ReliabilitySystem {
    fn new() -> Self {
        Self {
            sent_packets: VecDeque::new(),
            received_sequences: std::collections::HashSet::new(),
            ack_waiting_list: Vec::new(),
            resend_queue: VecDeque::new(),
            max_resend_attempts: 5,
            rtt_estimate: 0.1,
            congestion_window: 10,
        }
    }

    fn send_packet(&mut self, packet: NetworkPacket, current_time: f64) -> Option<NetworkPacket> {
        let mut packet = packet;
        packet.send_time = current_time;

        // Add to sent packets for potential resend
        if packet.requires_ack {
            self.sent_packets.push_back(packet.clone());
            self.ack_waiting_list.push(packet.id);
        }

        Some(packet)
    }

    fn receive_ack(&mut self, packet_id: u64, current_time: f64) {
        // Remove from waiting list
        self.ack_waiting_list.retain(|&id| id != packet_id);

        // Update RTT estimate
        if let Some(packet) = self.sent_packets.iter().find(|p| p.id == packet_id) {
            let rtt = current_time - packet.send_time;
            self.rtt_estimate = self.rtt_estimate * 0.9 + rtt * 0.1; // Exponential moving average
        }
    }

    fn check_timeouts(&mut self, current_time: f64) -> Vec<NetworkPacket> {
        let mut timed_out_packets = Vec::new();
        let timeout_threshold = self.rtt_estimate * 2.0; // 2x RTT timeout

        // Check for timed out packets
        while let Some(packet) = self.sent_packets.front() {
            if current_time - packet.send_time > timeout_threshold {
                let packet = self.sent_packets.pop_front().unwrap();
                timed_out_packets.push(packet);
            } else {
                break;
            }
        }

        // Resend timed out packets (with backoff)
        for packet in &timed_out_packets {
            if packet.requires_ack {
                // Implement exponential backoff
                let backoff_delay = timeout_threshold * 2.0f64.powf(rand::random::<f32>() as f64);
                // In real implementation, would schedule resend
                println!("Resending packet {} after timeout", packet.id);
            }
        }

        timed_out_packets
    }

    fn process_received_packet(&mut self, packet: &NetworkPacket) -> bool {
        // Check for duplicates
        if self.received_sequences.contains(&packet.sequence_number) {
            println!("Duplicate packet received: {}", packet.sequence_number);
            return false; // Discard duplicate
        }

        // Check sequence number for out-of-order detection
        self.received_sequences.insert(packet.sequence_number);

        true // Accept packet
    }
}

#[derive(Clone, Debug)]
struct CongestionControl {
    slow_start: bool,
    congestion_avoidance: bool,
    fast_retransmit: bool,
    fast_recovery: bool,
    ssthresh: usize,
    cwnd: usize,
    duplicate_ack_count: u32,
}

impl CongestionControl {
    fn new() -> Self {
        Self {
            slow_start: true,
            congestion_avoidance: false,
            fast_retransmit: false,
            fast_recovery: false,
            ssthresh: 64, // Slow start threshold
            cwnd: 1,      // Congestion window starts at 1
            duplicate_ack_count: 0,
        }
    }

    fn on_packet_loss(&mut self) {
        self.ssthresh = (self.cwnd / 2).max(1);
        self.cwnd = 1;
        self.slow_start = true;
        self.congestion_avoidance = false;
        println!("Packet loss detected - entering slow start");
    }

    fn on_ack_received(&mut self) {
        if self.slow_start {
            self.cwnd += 1;
            if self.cwnd >= self.ssthresh {
                self.slow_start = false;
                self.congestion_avoidance = true;
                println!("Switching to congestion avoidance");
            }
        } else if self.congestion_avoidance {
            // Additive increase
            self.cwnd += 1;
        }
    }

    fn get_send_window(&self) -> usize {
        self.cwnd
    }
}

#[derive(Clone, Debug)]
struct NetworkSimulator {
    conditions: NetworkCondition,
    packet_queue: VecDeque<PacketDelivery>,
    next_packet_id: u64,
    sequence_counter: u32,
    reliability_system: ReliabilitySystem,
    congestion_control: CongestionControl,
    current_time: f64,
}

impl NetworkSimulator {
    fn new(conditions: NetworkCondition) -> Self {
        Self {
            conditions,
            packet_queue: VecDeque::new(),
            next_packet_id: 0,
            sequence_counter: 0,
            reliability_system: ReliabilitySystem::new(),
            congestion_control: CongestionControl::new(),
            current_time: 0.0,
        }
    }

    fn send_packet(&mut self, data: Vec<u8>, priority: PacketPriority) -> u64 {
        let packet_id = self.next_packet_id;
        self.next_packet_id += 1;

        let sequence_number = self.sequence_counter;
        self.sequence_counter += 1;

        let packet = NetworkPacket::new(packet_id, sequence_number, data, priority);

        if let Some(packet) = self.reliability_system.send_packet(packet, self.current_time) {
            self.simulate_packet_delivery(packet);
        }

        packet_id
    }

    fn simulate_packet_delivery(&mut self, packet: NetworkPacket) {
        // Simulate network conditions
        let base_delay = self.conditions.latency_ms / 1000.0;
        let jitter = (rand::random::<f64>() - 0.5) * 2.0 * (self.conditions.jitter_ms / 1000.0);
        let total_delay = base_delay + jitter;

        let delivery_time = self.current_time + total_delay;

        // Simulate packet loss
        let packet_lost = rand::random::<f32>() < (self.conditions.packet_loss_percent / 100.0);

        if !packet_lost {
            // Check congestion control
            if self.packet_queue.len() < self.congestion_control.get_send_window() {
                let mut delivery = PacketDelivery {
                    packet,
                    delivery_time,
                    corrupted: rand::random::<f32>() < (self.conditions.corruption_percent / 100.0),
                    duplicated: false,
                };

                // Simulate duplication
                if rand::random::<f32>() < (self.conditions.duplication_percent / 100.0) {
                    let duplicate = PacketDelivery {
                        packet: delivery.packet.clone(),
                        delivery_time: delivery_time + rand::random::<f64>() * 0.1,
                        corrupted: delivery.corrupted,
                        duplicated: true,
                    };
                    self.packet_queue.push_back(duplicate);
                }

                // Simulate out-of-order delivery
                if rand::random::<f32>() < (self.conditions.out_of_order_percent / 100.0) {
                    delivery.delivery_time += rand::random::<f64>() * 0.2;
                }

                self.packet_queue.push_back(delivery);
            } else {
                println!("Packet dropped due to congestion control");
            }
        } else {
            self.congestion_control.on_packet_loss();
        }
    }

    fn receive_packets(&mut self) -> Vec<NetworkPacket> {
        let mut received_packets = Vec::new();

        while let Some(delivery) = self.packet_queue.front() {
            if delivery.delivery_time <= self.current_time {
                let delivery = self.packet_queue.pop_front().unwrap();

                if !delivery.corrupted && self.reliability_system.process_received_packet(&delivery.packet) {
                    // Send ACK for reliable packets
                    if delivery.packet.requires_ack {
                        // In real implementation, would send ACK
                        self.reliability_system.receive_ack(delivery.packet.id, self.current_time);
                        self.congestion_control.on_ack_received();
                    }
                    received_packets.push(delivery.packet);
                } else if delivery.corrupted {
                    println!("Corrupted packet received: {}", delivery.packet.id);
                }
            } else {
                break;
            }
        }

        received_packets
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;

        // Check for timeouts and resend
        let timed_out = self.reliability_system.check_timeouts(self.current_time);
        for packet in timed_out {
            if packet.requires_ack {
                println!("Resending timed out packet: {}", packet.id);
                self.simulate_packet_delivery(packet);
            }
        }
    }

    fn set_conditions(&mut self, conditions: NetworkCondition) {

        println!("Network conditions changed to: {:.0}ms latency, {:.1}% loss",
                conditions.latency_ms, conditions.packet_loss_percent);
        self.conditions = conditions;
    }
}

#[derive(Resource)]
struct BadNetworksDemo {
    simulator: NetworkSimulator,
    current_time: f64,
    packet_send_timer: f32,
    stats_packets_sent: u64,
    stats_packets_received: u64,
    condition_names: Vec<&'static str>,
    current_condition: usize,
}

impl BadNetworksDemo {
    fn new() -> Self {
        let condition_names = vec![
            "Excellent", "Good", "Poor", "Terrible", "Satellite", "Mobile 4G"
        ];

        Self {
            simulator: NetworkSimulator::new(NetworkCondition::good()),
            current_time: 0.0,
            packet_send_timer: 0.0,
            stats_packets_sent: 0,
            stats_packets_received: 0,
            condition_names,
            current_condition: 1, // Start with "Good"
        }
    }

    fn cycle_network_condition(&mut self) {
        self.current_condition = (self.current_condition + 1) % self.condition_names.len();

        let conditions = match self.current_condition {
            0 => NetworkCondition::excellent(),
            1 => NetworkCondition::good(),
            2 => NetworkCondition::poor(),
            3 => NetworkCondition::terrible(),
            4 => NetworkCondition::satellite(),
            5 => NetworkCondition::mobile_4g(),
            _ => NetworkCondition::good(),
        };

        self.simulator.set_conditions(conditions);
        println!("Switched to {} network conditions", self.condition_names[self.current_condition]);
    }

    fn update(&mut self, dt: f32) {
        self.current_time += dt as f64;
        self.packet_send_timer += dt;

        // Send packets periodically
        if self.packet_send_timer >= 0.1 { // 10 packets per second
            self.packet_send_timer = 0.0;

            // Send different types of packets
            let priorities = [
                PacketPriority::Critical,
                PacketPriority::Important,
                PacketPriority::Optional,
            ];
            let priority = priorities[rand::random::<usize>() % priorities.len()];
            let data_size = match priority {
                PacketPriority::Critical => 64,
                PacketPriority::Important => 32,
                PacketPriority::Optional => 16,
            };

            let data = vec![0u8; data_size];
            self.simulator.send_packet(data, priority);
            self.stats_packets_sent += 1;
        }

        // Receive packets
        let received = self.simulator.receive_packets();
        self.stats_packets_received += received.len() as u64;

        // Update simulator
        self.simulator.update(dt as f64);
    }

    fn get_packet_delivery_rate(&self) -> f32 {
        if self.stats_packets_sent > 0 {
            self.stats_packets_received as f32 / self.stats_packets_sent as f32 * 100.0
        } else {
            0.0
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bad Networks - Resilience & Error Correction".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(BadNetworksDemo::new())
        .add_systems(Startup, setup_bad_networks_demo)
        .add_systems(Update, (
            handle_bad_networks_input,
            update_bad_networks_simulation,
            render_bad_networks_demo,
            display_bad_networks_info,
        ).chain())
        .run();
}

fn setup_bad_networks_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_bad_networks_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<BadNetworksDemo>,
) {
    demo.update(1.0 / 60.0);

    // Cycle network conditions
    if keyboard_input.just_pressed(KeyCode::Space) {
        demo.cycle_network_condition();
    }

    // Manual packet send
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        let data = vec![42u8; 32]; // Send a test packet
        demo.simulator.send_packet(data, PacketPriority::Important);
        demo.stats_packets_sent += 1;
        println!("Manual packet sent");
    }
}

fn update_bad_networks_simulation(time: Res<Time>, mut demo: ResMut<BadNetworksDemo>) {
    // Updates are handled in input system
}

fn render_bad_networks_demo(
    mut commands: Commands,
    mut packet_entities: Local<Vec<Entity>>,
    demo: Res<BadNetworksDemo>,
) {
    // Clear previous frame
    for entity in packet_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render network condition indicator
    let condition_color = match demo.current_condition {
        0 => Color::rgb(0.2, 0.8, 0.2), // Excellent - green
        1 => Color::rgb(0.6, 0.8, 0.2), // Good - yellow-green
        2 => Color::rgb(0.8, 0.6, 0.2), // Poor - orange
        3 => Color::rgb(0.8, 0.2, 0.2), // Terrible - red
        4 => Color::rgb(0.4, 0.4, 0.8), // Satellite - blue
        5 => Color::rgb(0.6, 0.4, 0.8), // Mobile - purple
        _ => Color::rgb(0.5, 0.5, 0.5),
    };

    let condition_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: condition_color,
                custom_size: Some(Vec2::new(60.0, 60.0)),
                ..default()
            },
            transform: Transform::from_xyz(350.0, 250.0, 1.0),
            ..default()
        },

    )).id();
    packet_entities.push(condition_entity);

    // Render packet queue visualization
    let mut y_offset = 150.0;
    for delivery in demo.simulator.packet_queue.iter().take(8) {
        let color = if delivery.corrupted {
            Color::rgb(0.8, 0.2, 0.2) // Red for corrupted
        } else if delivery.duplicated {
            Color::rgb(0.8, 0.8, 0.2) // Yellow for duplicated
        } else {
            Color::rgb(0.2, 0.8, 0.2) // Green for normal
        };

        let packet_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(delivery.packet.size_bytes as f32 / 4.0, 8.0)),
                    ..default()
                },
                transform: Transform::from_xyz(100.0, y_offset, 1.0),
                ..default()
            },

        )).id();
        packet_entities.push(packet_entity);
        y_offset -= 15.0;
    }
}

fn display_bad_networks_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<BadNetworksDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        let delivery_rate = demo.get_packet_delivery_rate();

        println!("\n=== Bad Networks Demo ===");
        println!("Current Network: {}", demo.condition_names[demo.current_condition]);
        println!("Packets Sent: {}", demo.stats_packets_sent);
        println!("Packets Received: {}", demo.stats_packets_received);
        println!("Delivery Rate: {:.1}%", delivery_rate);
        println!("Packets in Queue: {}", demo.simulator.packet_queue.len());
        println!("Congestion Window: {}", demo.simulator.congestion_control.cwnd);
        println!("RTT Estimate: {:.0}ms", demo.simulator.reliability_system.rtt_estimate * 1000.0);
        println!("Packets Awaiting ACK: {}", demo.simulator.reliability_system.ack_waiting_list.len());

        println!("\nNetwork Conditions:");
        println!("Latency: {:.0}ms", demo.simulator.conditions.latency_ms);
        println!("Jitter: {:.0}ms", demo.simulator.conditions.jitter_ms);
        println!("Packet Loss: {:.1}%", demo.simulator.conditions.packet_loss_percent);
        println!("Duplication: {:.1}%", demo.simulator.conditions.duplication_percent);
        println!("Corruption: {:.1}%", demo.simulator.conditions.corruption_percent);
        println!("Out of Order: {:.1}%", demo.simulator.conditions.out_of_order_percent);
        println!("Bandwidth: {:.0} Kbps", demo.simulator.conditions.bandwidth_kbps);

        println!("\nControls:");
        println!("  Space: Cycle network conditions");
        println!("  S: Send manual packet");
        println!("  I: Show this info");
        println!("\nNetwork condition colors:");
        println!("  Green: Excellent | Yellow-Green: Good");
        println!("  Orange: Poor | Red: Terrible");
        println!("  Blue: Satellite | Purple: Mobile 4G");
        println!("======================\n");
    }
}
