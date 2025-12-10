// Example: A Networked Monster
// Based on "A Networked Monster" blog post
// https://www.slowrush.dev/news/a-networked-monster

use std::collections::VecDeque;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Networked monster AI and synchronization
// Demonstrates networked entity behavior, AI synchronization, and predictive movement

#[derive(Clone, Debug)]
struct NetworkedEntity {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    health: f32,
    max_health: f32,
    entity_type: EntityType,
    last_update_time: f64,
    predicted_position: Vec2,
    authority: NetworkAuthority,
}

#[derive(Clone, Debug, PartialEq)]
enum EntityType {
    Player,
    Monster,
    Projectile,
}

#[derive(Clone, Debug, PartialEq)]
enum NetworkAuthority {
    Local,
    Remote,
    Server,
}

#[derive(Clone, Debug)]
enum MonsterState {
    Idle,
    Chasing { target_id: u32 },
    Attacking { target_id: u32, attack_timer: f32 },
    Fleeing { from_id: u32 },
}

#[derive(Clone, Debug)]
struct MonsterAI {
    state: MonsterState,
    detection_radius: f32,
    attack_range: f32,
    attack_damage: f32,
    attack_cooldown: f32,
    current_cooldown: f32,
    move_speed: f32,
    last_known_target_pos: Vec2,
}

impl MonsterAI {
    fn new() -> Self {
        Self {
            state: MonsterState::Idle,
            detection_radius: 150.0,
            attack_range: 30.0,
            attack_damage: 25.0,
            attack_cooldown: 2.0,
            current_cooldown: 0.0,
            move_speed: 80.0,
            last_known_target_pos: Vec2::ZERO,
        }
    }

    fn update(&mut self, monster: &mut NetworkedEntity, entities: &[NetworkedEntity], dt: f32) {
        self.current_cooldown -= dt;

        // Use std::mem::replace to temporarily take ownership of state to avoid borrowing conflicts
        // Replace with Idle temporarily (will be set to correct state at end of match)
        // match &mut self.state {
        //             MonsterState::Idle => {
        //                 // Look for targets
        //                 if let Some(target) = self.find_nearest_target(monster, entities) {
        //                     self.state = MonsterState::Chasing { target_id: target.id };
        //                     self.last_known_target_pos = target.position;
        //                     println!("Monster {}: Found target {}", monster.id, target.id);
        //                 }
        //             }
        //             MonsterState::Chasing { target_id } => {
        //                 if let Some(target) = entities.iter().find(|e| e.id == *target_id) {
        //                     let distance = monster.position.distance(target.position);
        //
        //                     if distance <= self.attack_range && self.current_cooldown <= 0.0 {
        //                         // Start attacking
        //                         self.state = MonsterState::Attacking {
        //                             target_id: *target_id,
        //                             attack_timer: 1.0,
        //                         };
        //                         self.current_cooldown = self.attack_cooldown;
        //                         println!("Monster {}: Attacking target {}", monster.id, target_id);
        //                     } else if distance <= self.detection_radius {
        //                         // Move toward target
        //                         let direction = (target.position - monster.position).normalize();
        //                         monster.velocity = direction * self.move_speed;
        //                         self.last_known_target_pos = target.position;
        //                     } else {
        //                         // Lost target, go back to idle
        //                         self.state = MonsterState::Idle;
        //                         monster.velocity = Vec2::ZERO;
        //                         println!("Monster {}: Lost target {}", monster.id, target_id);
        //                     }
        //                 } else {
        //                     // Target disappeared, go back to idle
        //                     self.state = MonsterState::Idle;
        //                     monster.velocity = Vec2::ZERO;
        //                 }
        //             }
        //             MonsterState::Attacking { target_id, attack_timer } => {
        //                 *attack_timer -= dt;
        //                 monster.velocity = Vec2::ZERO; // Stop moving while attacking
        //
        //                 if *attack_timer <= 0.0 {
        //                     // Attack finished, check if target still exists
        //                     if entities.iter().any(|e| e.id == *target_id) {
        //                         self.state = MonsterState::Chasing { target_id: *target_id };
        //                     } else {
        //                         self.state = MonsterState::Idle;
        //                     }
        //                 }
        //             }
        //             MonsterState::Fleeing { from_id } => {
        //                 if let Some(threat) = entities.iter().find(|e| e.id == *from_id) {
        //                     let direction = (monster.position - threat.position).normalize();
        //                     monster.velocity = direction * (self.move_speed * 1.5);
        //
        //                     // Check if far enough to stop fleeing
        //                     if monster.position.distance(threat.position) > self.detection_radius * 1.5 {
        //                         self.state = MonsterState::Idle;
        //                         monster.velocity = Vec2::ZERO;
        //                     }
        //                 } else {
        //                     self.state = MonsterState::Idle;
        //                     monster.velocity = Vec2::ZERO;
        //                 }
        //             }
        //         }
        // if matches!(self.state, MonsterState::Idle) {
        //             // Look for targets
        //             if let Some(target) = self.find_nearest_target(monster, entities) {
        //                 let target_id = target.id;
        //                 let target_pos = target.position;
        //                 self.state = MonsterState::Chasing { target_id };
        //                 self.last_known_target_pos = target_pos;
        //                 println!("Monster {}: Found target {}", monster.id, target_id);
        //                 return; // State changed, process next frame
        //             }
        //         }
        // MonsterState 未实现 Default。改用 std::mem::replace 替代 std::mem::take：   ##
        let current_state = std::mem::replace(&mut self.state, MonsterState::Idle);
        
        self.state = match current_state {
            MonsterState::Idle => {
                // Look for targets
                if let Some(target) = self.find_nearest_target(monster, entities) {
                    let target_id = target.id;
                    self.last_known_target_pos = target.position;
                    println!("Monster {}: Found target {}", monster.id, target_id);
                    MonsterState::Chasing { target_id }
                } else {
                    MonsterState::Idle
                }
            }
            MonsterState::Chasing { target_id } => {
                if let Some(target) = entities.iter().find(|e| e.id == target_id) {
                    let distance = monster.position.distance(target.position);

                    if distance <= self.attack_range && self.current_cooldown <= 0.0 {
                        // Start attacking
                        self.current_cooldown = self.attack_cooldown;
                        println!("Monster {}: Attacking target {}", monster.id, target_id);
                        MonsterState::Attacking {
                            target_id,
                            attack_timer: 1.0,
                        }
                    } else if distance <= self.detection_radius {
                        // Move toward target
                        let direction = (target.position - monster.position).normalize();
                        monster.velocity = direction * self.move_speed;
                        self.last_known_target_pos = target.position;
                        MonsterState::Chasing { target_id }
                    } else {
                        // Lost target, go back to idle
                        monster.velocity = Vec2::ZERO;
                        println!("Monster {}: Lost target {}", monster.id, target_id);
                        MonsterState::Idle
                    }
                } else {
                    // Target disappeared, go back to idle
                    monster.velocity = Vec2::ZERO;
                    MonsterState::Idle
                }
            }
            MonsterState::Attacking { target_id, mut attack_timer } => {
                attack_timer -= dt;
                monster.velocity = Vec2::ZERO; // Stop moving while attacking

                if attack_timer <= 0.0 {
                    // Attack finished, check if target still exists
                    if entities.iter().any(|e| e.id == target_id) {
                        MonsterState::Chasing { target_id }
                    } else {
                        MonsterState::Idle
                    }
                } else {
                    MonsterState::Attacking { target_id, attack_timer }
                }
            }
            MonsterState::Fleeing { from_id } => {
                if let Some(threat) = entities.iter().find(|e| e.id == from_id) {
                    let direction = (monster.position - threat.position).normalize();
                    monster.velocity = direction * (self.move_speed * 1.5);

                    // Check if far enough to stop fleeing
                    if monster.position.distance(threat.position) > self.detection_radius * 1.5 {
                        monster.velocity = Vec2::ZERO;
                        MonsterState::Idle
                    } else {
                        MonsterState::Fleeing { from_id }
                    }
                } else {
                    monster.velocity = Vec2::ZERO;
                    MonsterState::Idle
                }
            }
        };
    }

    fn find_nearest_target<'a>(&self, monster: &NetworkedEntity, entities: &'a[NetworkedEntity]) -> Option<&'a NetworkedEntity> {
        entities.iter()
            .filter(|e| e.entity_type == EntityType::Player && e.health > 0.0)
            .filter(|e| monster.position.distance(e.position) <= self.detection_radius)
            .min_by(|a, b| {
                monster.position.distance(a.position)
                    .partial_cmp(&monster.position.distance(b.position))
                    .unwrap()
            })
    }

    fn take_damage(&mut self, damage: f32, attacker_pos: Vec2) {
        // If health is low, consider fleeing
        if damage > 50.0 && rand::random::<f32>() < 0.3 {
            // Would need access to monster's health here
            // For now, just a chance to flee
        }
    }
}

#[derive(Clone, Debug)]
struct NetworkMessage {
    entity_id: u32,
    timestamp: f64,
    position: Vec2,
    velocity: Vec2,
    health: f32,
    message_type: NetworkMessageType,
}

#[derive(Clone, Debug)]
enum NetworkMessageType {
    PositionUpdate,
    HealthUpdate,
    Attack { target_id: u32 },
    Death,
}

#[derive(Clone, Debug)]
struct NetworkSimulation {
    latency: f64,
    packet_loss: f32,
    message_queue: VecDeque<NetworkMessage>,
}

impl NetworkSimulation {
    fn new(latency: f64, packet_loss: f32) -> Self {
        Self {
            latency,
            packet_loss,
            message_queue: VecDeque::new(),
        }
    }

    fn send_message(&mut self, message: NetworkMessage) {
        // Simulate packet loss
        if rand::random::<f32>() >= self.packet_loss {
            self.message_queue.push_back(message);
        }
    }

    fn receive_messages(&mut self, current_time: f64) -> Vec<NetworkMessage> {
        let mut received = Vec::new();

        while let Some(msg) = self.message_queue.front() {
            if current_time >= msg.timestamp + self.latency {
                received.push(self.message_queue.pop_front().unwrap());
            } else {
                break;
            }
        }

        received
    }
}

#[derive(Clone, Debug,Resource)]
struct NetworkedMonsterDemo {
    entities: Vec<NetworkedEntity>,
    monster_ais: Vec<MonsterAI>,
    network_sim: NetworkSimulation,
    local_player_id: u32,
    next_entity_id: u32,
    current_time: f64,
    tick_timer: f32,
}

impl NetworkedMonsterDemo {
    fn new() -> Self {
        let mut entities = Vec::new();
        let mut monster_ais = Vec::new();
        let mut next_id = 0;

        // Create local player
        let player = NetworkedEntity {
            id: next_id,
            position: Vec2::new(100.0, 300.0),
            velocity: Vec2::ZERO,
            health: 100.0,
            max_health: 100.0,
            entity_type: EntityType::Player,
            last_update_time: 0.0,
            predicted_position: Vec2::new(100.0, 300.0),
            authority: NetworkAuthority::Local,
        };
        entities.push(player);
        let local_player_id = next_id;
        next_id += 1;

        // Create monsters
        for i in 0..3 {
            let monster = NetworkedEntity {
                id: next_id,
                position: Vec2::new(300.0 + i as f32 * 150.0, 200.0 + i as f32 * 50.0),
                velocity: Vec2::ZERO,
                health: 150.0,
                max_health: 150.0,
                entity_type: EntityType::Monster,
                last_update_time: 0.0,
                predicted_position: Vec2::new(300.0 + i as f32 * 150.0, 200.0 + i as f32 * 50.0),
                authority: NetworkAuthority::Server,
            };
            entities.push(monster);
            monster_ais.push(MonsterAI::new());
            next_id += 1;
        }

        Self {
            entities,
            monster_ais,
            network_sim: NetworkSimulation::new(0.1, 0.05), // 100ms latency, 5% packet loss
            local_player_id,
            next_entity_id: next_id,
            current_time: 0.0,
            tick_timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.current_time += dt as f64;
        self.tick_timer += dt;

        // Update local player
        if let Some(player) = self.entities.iter_mut().find(|e| e.id == self.local_player_id) {
            // Player input would be handled elsewhere
            // For demo, add some simple movement
            player.velocity.y -= 300.0 * dt;

            // Ground collision
            if player.position.y <= 50.0 {
                player.position.y = 50.0;
                player.velocity.y = 0.0;
            }

            player.position += player.velocity * dt;
        }

        // Update monster AI (only for server-authoritative monsters)
        // 在同一个作用域内，self.entities 被可变借用（通过 iter_mut()），然后又试图不可变借用（通过 iter() 和 &self.entities），导致借用冲突。
        // 解决方案：
        // 先收集所有怪物的索引（使用不可变借用）
        // 对每个怪物，先克隆整个 entities 向量作为快照
        // 获取怪物的可变引用
        // 使用快照调用 update 方法
        // 这样避免了同时可变和不可变借用 self.entities。虽然克隆有性能开销，但在这个场景下是可接受的。如果需要更高性能，可以考虑使用 unsafe 代码或重新设计数据结构。
        // for i in 0..self.monster_ais.len() {  ##
        //             if let Some(monster) = self.entities.iter_mut().find(|e| e.entity_type == EntityType::Monster) {
        //                 let entity_index = self.entities.iter().position(|e| e.id == monster.id).unwrap();
        //                 if entity_index > 0 { // Skip player
        //                     self.monster_ais[i].update(monster, &self.entities, dt);
        //                 }
        //             }
        //         }
        // First, find monster indices to avoid borrowing conflicts
        let monster_indices: Vec<usize> = self.entities
            .iter()
            .enumerate()
            .filter(|(_, e)| e.entity_type == EntityType::Monster)
            .map(|(idx, _)| idx)
            .collect();
        
        for (i, &monster_idx) in monster_indices.iter().enumerate() {
            if i < self.monster_ais.len() && monster_idx < self.entities.len() {
                // We need both mutable monster and immutable entities
                // Use a workaround: create a temporary immutable reference to entities
                // by borrowing it before getting the mutable monster reference
                let entities_snapshot: Vec<NetworkedEntity> = self.entities.clone();
                let monster = &mut self.entities[monster_idx];
                self.monster_ais[i].update(monster, &entities_snapshot, dt);
            }
        }

        // Physics update for all entities
        for entity in &mut self.entities {
            // Apply velocity
            entity.position += entity.velocity * dt;

            // Boundary constraints
            entity.position.x = entity.position.x.clamp(0.0, 800.0);
            entity.position.y = entity.position.y.max(50.0);

            // Update predicted position
            entity.predicted_position = entity.position + entity.velocity * 0.1; // 100ms prediction
        }

        // Network synchronization (every 50ms)
        if self.tick_timer >= 0.05 {
            self.tick_timer = 0.0;
            self.sync_network();
        }

        // Process received network messages
        let messages = self.network_sim.receive_messages(self.current_time);
        for message in messages {
            self.process_network_message(message);
        }
    }

    fn sync_network(&mut self) {
        // Send position updates for all entities
        for entity in &self.entities {
            if entity.authority == NetworkAuthority::Server || entity.authority == NetworkAuthority::Local {
                let message = NetworkMessage {
                    entity_id: entity.id,
                    timestamp: self.current_time,
                    position: entity.position,
                    velocity: entity.velocity,
                    health: entity.health,
                    message_type: NetworkMessageType::PositionUpdate,
                };
                self.network_sim.send_message(message);
            }
        }
    }

    fn process_network_message(&mut self, message: NetworkMessage) {
        if let Some(entity) = self.entities.iter_mut().find(|e| e.id == message.entity_id) {
            // Only update if we're not the authority for this entity
            if entity.authority != NetworkAuthority::Local {
                entity.position = message.position;
                entity.velocity = message.velocity;
                entity.health = message.health;
                entity.last_update_time = message.timestamp;

                // Update predicted position
                entity.predicted_position = entity.position + entity.velocity * self.network_sim.latency as f32;
            }
        }
    }

    fn get_local_player(&self) -> Option<&NetworkedEntity> {
        self.entities.iter().find(|e| e.id == self.local_player_id)
    }

    fn get_local_player_mut(&mut self) -> Option<&mut NetworkedEntity> {
        self.entities.iter_mut().find(|e| e.id == self.local_player_id)
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "A Networked Monster - AI & Network Synchronization".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(NetworkedMonsterDemo::new())
        .add_systems(Startup, setup_networked_monster_demo)
        .add_systems(Update, (
            handle_networked_monster_input,
            update_networked_monster_simulation,
            render_networked_monster_demo,
            display_networked_monster_info,
        ))
        .run();
}

fn setup_networked_monster_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_networked_monster_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<NetworkedMonsterDemo>,
) {
    if let Some(player) = demo.get_local_player_mut() {
        // Movement
        player.velocity.x = 0.0;

        if keyboard_input.pressed(KeyCode::KeyA) {
            player.velocity.x = -120.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            player.velocity.x = 120.0;
        }

        // Jump
        if keyboard_input.just_pressed(KeyCode::Space) && player.position.y <= 55.0 {
            player.velocity.y = 180.0;
        }

        // Simulate taking damage
        if keyboard_input.just_pressed(KeyCode::KeyH) {
            player.health -= 20.0;
            if player.health <= 0.0 {
                player.health = 100.0;
                player.position = Vec2::new(100.0, 300.0);
                println!("Player respawned!");
            }
        }
    }

    // Adjust network conditions
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        demo.network_sim.latency += 0.02;
        println!("Latency increased to {:.0}ms", demo.network_sim.latency * 1000.0);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        demo.network_sim.latency = (demo.network_sim.latency - 0.02).max(0.0);
        println!("Latency decreased to {:.0}ms", demo.network_sim.latency * 1000.0);
    }
}

fn update_networked_monster_simulation(time: Res<Time>, mut demo: ResMut<NetworkedMonsterDemo>) {
    demo.update(time.delta_seconds().min(1.0 / 30.0));
}

fn render_networked_monster_demo(
    mut commands: Commands,
    mut entity_entities: Local<Vec<Entity>>,
    demo: Res<NetworkedMonsterDemo>,
) {
    // Clear previous frame
    for entity in entity_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render all entities
    for entity in &demo.entities {
        let color = match entity.entity_type {
            EntityType::Player => {
                if entity.id == demo.local_player_id {
                    Color::srgb(0.2, 0.8, 0.2) // Local player - green
                } else {
                    Color::srgb(0.2, 0.4, 0.8) // Remote player - blue
                }
            }
            EntityType::Monster => {
                if entity.health > 100.0 {
                    Color::srgb(0.8, 0.2, 0.2) // Healthy monster - red
                } else {
                    Color::srgb(0.6, 0.2, 0.2) // Damaged monster - dark red
                }
            }
            EntityType::Projectile => Color::srgb(1.0, 1.0, 0.0), // Yellow
        };

        let size = match entity.entity_type {
            EntityType::Player => Vec2::new(20.0, 30.0),
            EntityType::Monster => Vec2::new(25.0, 35.0),
            EntityType::Projectile => Vec2::new(5.0, 5.0),
        };

        // Spawn sprite entity
        let sprite_entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(entity.position.x, entity.position.y, 1.0),
            ..default()
        }).id();
        entity_entities.push(sprite_entity);

        // Spawn text entity as a child or separate entity
        let text_entity = commands.spawn(Text2dBundle {
            text: Text::from_section(
                format!("HP: {:.0}", entity.health),
                TextStyle {
                    font_size: 8.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(entity.position.x, entity.position.y + 18.0, 2.0),
            ..default()
        }).id();
        entity_entities.push(text_entity);

        // Render predicted position (subtle)
        if entity.authority != NetworkAuthority::Local {
            // Create a semi-transparent version of the color
            // color: Color::srgba(color.r(), color.g(), color.b(), 0.3),  等同于 predicted_color
            let predicted_color = match entity.entity_type {
                EntityType::Player => {
                    if entity.id == demo.local_player_id {
                        Color::srgba(0.2, 0.8, 0.2, 0.3) // Local player - green
                    } else {
                        Color::srgba(0.2, 0.4, 0.8, 0.3) // Remote player - blue
                    }
                }
                EntityType::Monster => {
                    if entity.health > 100.0 {
                        Color::srgba(0.8, 0.2, 0.2, 0.3) // Healthy monster - red
                    } else {
                        Color::srgba(0.6, 0.2, 0.2, 0.3) // Damaged monster - dark red
                    }
                }
                EntityType::Projectile => Color::srgba(1.0, 1.0, 0.0, 0.3), // Yellow
            };
            
            let predict_entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: predicted_color,
                    custom_size: Some(size * 0.8),
                    ..default()
                },
                transform: Transform::from_xyz(entity.predicted_position.x, entity.predicted_position.y, 0.5),
                ..default()
            }).id();
            entity_entities.push(predict_entity);
        }
    }
}

fn display_networked_monster_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<NetworkedMonsterDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        println!("\n=== Networked Monster Demo ===");
        println!("Current Time: {:.2}s", demo.current_time);
        println!("Network Latency: {:.0}ms", demo.network_sim.latency * 1000.0);
        println!("Packet Loss: {:.1}%", demo.network_sim.packet_loss * 100.0);
        println!("Queued Messages: {}", demo.network_sim.message_queue.len());

        println!("\nEntities:");
        for entity in &demo.entities {
            let authority_str = match entity.authority {
                NetworkAuthority::Local => "Local",
                NetworkAuthority::Remote => "Remote",
                NetworkAuthority::Server => "Server",
            };

            println!("  {}: {:?} at ({:.1}, {:.1}) HP: {:.0} [{}]",
                    entity.id,
                    entity.entity_type,
                    entity.position.x,
                    entity.position.y,
                    entity.health,
                    authority_str);
        }

        println!("\nMonster AI States:");
        for (i, ai) in demo.monster_ais.iter().enumerate() {
            let state_str = match &ai.state {
                MonsterState::Idle => "Idle".to_string(),
                MonsterState::Chasing { target_id } => format!("Chasing {}", target_id),
                MonsterState::Attacking { target_id, attack_timer } => format!("Attacking {} ({:.1}s)", target_id, attack_timer),
                MonsterState::Fleeing { from_id } => format!("Fleeing from {}", from_id),
            };
            println!("  Monster {}: {}", i, state_str);
        }

        println!("\nControls:");
        println!("  A/D: Move | Space: Jump | H: Take damage");
        println!("  ↑/↓: Adjust latency");
        println!("  I: Show this info");
        println!("======================\n");
    }
}
