// Example: Creative Barriers
// Based on "Creative Barriers" blog post
// https://www.slowrush.dev/news/creative-barriers

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Creative barriers and level design challenges
// Demonstrates barriers that encourage creative problem solving

#[derive(Clone, Debug, PartialEq)]
enum BarrierType {
    Wall,
    SpikeTrap,
    LaserBeam,
    ForceField,
    Teleporter,
    Switch,
    Door,
}

impl BarrierType {
    fn color(&self) -> Color {
        match self {
            BarrierType::Wall => Color::rgb(0.5, 0.5, 0.5),
            BarrierType::SpikeTrap => Color::rgb(0.8, 0.2, 0.2),
            BarrierType::LaserBeam => Color::rgb(1.0, 0.0, 0.0),
            BarrierType::ForceField => Color::rgba(0.0, 1.0, 1.0, 0.5),
            BarrierType::Teleporter => Color::rgb(1.0, 0.0, 1.0),
            BarrierType::Switch => Color::rgb(1.0, 1.0, 0.0),
            BarrierType::Door => Color::rgb(0.6, 0.3, 0.0),
        }
    }

    fn is_hazardous(&self) -> bool {
        matches!(self, BarrierType::SpikeTrap | BarrierType::LaserBeam)
    }

    fn is_interactive(&self) -> bool {
        matches!(self, BarrierType::Switch | BarrierType::Teleporter)
    }
}

#[derive(Clone, Debug)]
struct Barrier {
    barrier_type: BarrierType,
    position: Vec2,
    size: Vec2,
    activated: bool,
    target_position: Option<Vec2>, // For teleporters
    connected_door: Option<usize>, // For switches
}

impl Barrier {
    fn new(barrier_type: BarrierType, position: Vec2, size: Vec2) -> Self {
        Self {
            barrier_type,
            position,
            size,
            activated: false,
            target_position: None,
            connected_door: None,
        }
    }

    fn activate(&mut self) {
        self.activated = true;
    }

    fn deactivate(&mut self) {
        self.activated = false;
    }
}

#[derive(Clone, Debug)]
enum PlayerTool {
    None,
    WaterGun,
    FireExtinguisher,
    TeleportGun,
    SwitchTool,
}

#[derive(Clone, Debug)]
struct Player {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
    selected_tool: PlayerTool,
    health: f32,
    max_health: f32,
}

impl Player {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            size: Vec2::new(20.0, 30.0),
            selected_tool: PlayerTool::None,
            health: 100.0,
            max_health: 100.0,
        }
    }

    fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
        if self.health <= 0.0 {
            // Respawn
            self.position = Vec2::new(50.0, 50.0);
            self.velocity = Vec2::ZERO;
            self.health = self.max_health;
        }
    }

    fn use_tool(&self, target_pos: Vec2) -> Option<PlayerAction> {
        match self.selected_tool {
            PlayerTool::WaterGun => Some(PlayerAction::ShootWater(target_pos)),
            PlayerTool::FireExtinguisher => Some(PlayerAction::ExtinguishFire(target_pos)),
            PlayerTool::TeleportGun => Some(PlayerAction::TeleportTo(target_pos)),
            PlayerTool::SwitchTool => Some(PlayerAction::ActivateSwitch(target_pos)),
            PlayerTool::None => None,
        }
    }
}

#[derive(Clone, Debug)]
enum PlayerAction {
    ShootWater(Vec2),
    ExtinguishFire(Vec2),
    TeleportTo(Vec2),
    ActivateSwitch(Vec2),
}

#[derive(Clone, Debug)]
struct PlayerInput {
    move_x: f32,
    jump: bool,
}

struct CreativeLevel {
    barriers: Vec<Barrier>,
    player: Player,
    width: f32,
    height: f32,
    objectives: Vec<Objective>,
    completed_objectives: Vec<bool>,
}

#[derive(Clone, Debug)]
enum Objective {
    ReachPoint(Vec2, String),
    ActivateSwitches(usize, String),
    AvoidHazards(String),
    UseToolsCreatively(String),
}

impl CreativeLevel {
    fn new(width: f32, height: f32) -> Self {
        let mut barriers = Vec::new();

        // Create a challenging level with various barriers
        barriers.push(Barrier::new(BarrierType::Wall, Vec2::new(200.0, 100.0), Vec2::new(20.0, 200.0)));
        barriers.push(Barrier::new(BarrierType::SpikeTrap, Vec2::new(150.0, 50.0), Vec2::new(100.0, 20.0)));
        barriers.push(Barrier::new(BarrierType::LaserBeam, Vec2::new(300.0, 150.0), Vec2::new(200.0, 5.0)));
        barriers.push(Barrier::new(BarrierType::ForceField, Vec2::new(100.0, 200.0), Vec2::new(150.0, 10.0)));
        barriers.push(Barrier::new(BarrierType::Teleporter, Vec2::new(50.0, 250.0), Vec2::new(30.0, 30.0)));
        barriers.push(Barrier::new(BarrierType::Switch, Vec2::new(350.0, 50.0), Vec2::new(20.0, 20.0)));
        barriers.push(Barrier::new(BarrierType::Door, Vec2::new(400.0, 100.0), Vec2::new(20.0, 100.0)));

        // Set up teleporter target
        if let Some(teleporter) = barriers.get_mut(4) {
            teleporter.target_position = Some(Vec2::new(450.0, 250.0));
        }

        // Connect switch to door
        if let Some(switch) = barriers.get_mut(5) {
            switch.connected_door = Some(6);
        }

        let player = Player::new(Vec2::new(50.0, 50.0));

        let objectives = vec![
            Objective::ReachPoint(Vec2::new(500.0, 300.0), "Reach the goal point".to_string()),
            Objective::ActivateSwitches(1, "Activate the switch to open the door".to_string()),
            Objective::AvoidHazards("Avoid spikes and lasers".to_string()),
            Objective::UseToolsCreatively("Use tools creatively to overcome barriers".to_string()),
        ];

        Self {
            barriers,
            player,
            width,
            height,
            objectives,
            completed_objectives: vec![false; 4],
        }
    }

    fn update(&mut self, dt: f32, input: &PlayerInput, actions: &[PlayerAction]) {
        // Update player physics
        self.player.velocity.x = input.move_x * 150.0;
        self.player.velocity.y -= 300.0 * dt; // Gravity

        let mut new_position = self.player.position + self.player.velocity * dt;

        // Collision detection with barriers
        let player_rect = Rect::from_center_size(new_position, self.player.size);

        for barrier in &self.barriers {
            let barrier_rect = Rect::from_center_size(barrier.position, barrier.size);
            let intersection = player_rect.intersect(barrier_rect);

            if intersection.width() > 0.0 && intersection.height() > 0.0 {
                match barrier.barrier_type {
                    BarrierType::Wall => {
                        // Block movement
                        if (new_position.x - self.player.position.x).abs() > (new_position.y - self.player.position.y).abs() {
                            new_position.x = self.player.position.x;
                        } else {
                            new_position.y = self.player.position.y;
                        }
                        self.player.velocity = Vec2::ZERO;
                    }
                    BarrierType::Door => {
                        // Only block if door is not activated (closed)
                        if !barrier.activated {
                            if (new_position.x - self.player.position.x).abs() > (new_position.y - self.player.position.y).abs() {
                                new_position.x = self.player.position.x;
                            } else {
                                new_position.y = self.player.position.y;
                            }
                            self.player.velocity = Vec2::ZERO;
                        }
                    }
                    BarrierType::SpikeTrap => {
                        self.player.take_damage(50.0 * dt);
                    }
                    BarrierType::LaserBeam => {
                        self.player.take_damage(100.0 * dt);
                    }
                    BarrierType::ForceField => {
                        // Push player back
                        let push_dir = (self.player.position - barrier.position).normalize();
                        self.player.velocity += push_dir * 200.0 * dt;
                    }
                    BarrierType::Teleporter => {
                        if let Some(target) = barrier.target_position {
                            self.player.position = target;
                            self.player.velocity = Vec2::ZERO;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Ground collision (simple)
        if new_position.y <= 20.0 {
            new_position.y = 20.0;
            self.player.velocity.y = 0.0;

            // Handle jump
            if input.jump {
                self.player.velocity.y = 200.0;
            }
        }

        self.player.position = new_position;

        // Handle player actions
        for action in actions {
            match action {
                PlayerAction::ActivateSwitch(pos) => {
                    let mut doors_to_activate = Vec::new();
                    for barrier in &mut self.barriers {
                        if barrier.barrier_type == BarrierType::Switch {
                            let switch_rect = Rect::from_center_size(barrier.position, barrier.size);
                            if switch_rect.contains(*pos) {
                                barrier.activate();
                                // Collect door indices to activate
                                if let Some(door_idx) = barrier.connected_door {
                                    doors_to_activate.push(door_idx);
                                }
                            }
                        }
                    }
                    // Open connected doors
                    for door_idx in doors_to_activate {
                        if let Some(door) = self.barriers.get_mut(door_idx) {
                            door.activate(); // Door becomes passable
                        }
                    }
                }
                PlayerAction::TeleportTo(pos) => {
                    self.player.position = *pos;
                    self.player.velocity = Vec2::ZERO;
                }
                _ => {} // Other actions would be handled by other systems
            }
        }

        // Check objectives
        self.check_objectives();
    }

    fn check_objectives(&mut self) {
        // Objective 1: Reach goal point
        if !self.completed_objectives[0] {
            let distance = self.player.position.distance(Vec2::new(500.0, 300.0));
            if distance < 30.0 {
                self.completed_objectives[0] = true;
                println!("✓ Objective 1 completed: Reached goal point!");
            }
        }

        // Objective 2: Activate switch
        if !self.completed_objectives[1] {
            let switches_activated = self.barriers.iter()
                .filter(|b| b.barrier_type == BarrierType::Switch && b.activated)
                .count();
            if switches_activated >= 1 {
                self.completed_objectives[1] = true;
                println!("✓ Objective 2 completed: Activated switch!");
            }
        }

        // Objective 3: Survive (checked continuously)
        self.completed_objectives[2] = self.player.health > 0.0;

        // Objective 4: Use creative tools (would need more complex logic)
        self.completed_objectives[3] = self.player.health > 50.0; // Simple proxy
    }

    fn get_completion_percentage(&self) -> f32 {
        let completed = self.completed_objectives.iter().filter(|&&c| c).count();
        completed as f32 / self.objectives.len() as f32 * 100.0
    }
}

#[derive(Resource)]
struct CreativeLevelResource(CreativeLevel);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Creative Barriers - Puzzle Solving with Physics".to_string(),
                resolution: (600.0, 400.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(CreativeLevelResource(CreativeLevel::new(600.0, 400.0)))
        .add_systems(Startup, setup_creative_demo)
        .add_systems(Update, (
            handle_creative_input,
            update_creative_level,
            render_creative_level,
            display_creative_info,
        ).chain())
        .run();
}

fn setup_creative_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_creative_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut level: ResMut<CreativeLevelResource>,
) {
    // Tool selection
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        level.0.player.selected_tool = PlayerTool::WaterGun;
        println!("Selected: Water Gun");
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        level.0.player.selected_tool = PlayerTool::FireExtinguisher;
        println!("Selected: Fire Extinguisher");
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        level.0.player.selected_tool = PlayerTool::TeleportGun;
        println!("Selected: Teleport Gun");
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        level.0.player.selected_tool = PlayerTool::SwitchTool;
        println!("Selected: Switch Tool");
    }

    // Player movement
    let mut move_x = 0.0;
    if keyboard_input.pressed(KeyCode::KeyA) {
        move_x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        move_x = 1.0;
    }
    let jump = keyboard_input.just_pressed(KeyCode::Space);

    let input = PlayerInput { move_x, jump };

    // Handle tool usage
    let mut actions = Vec::new();
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        if let Some(action) = level.0.player.use_tool(world_pos.origin.truncate()) {
                            actions.push(action);
                        }
                    }
                }
            }
        }
    }

    level.0.update(1.0 / 60.0, &input, &actions);
}

fn update_creative_level(
    time: Res<Time>,
    mut level: ResMut<CreativeLevelResource>,
) {
    // Level updates are handled in input system
}

fn render_creative_level(
    mut commands: Commands,
    mut barrier_entities: Local<Vec<Entity>>,
    mut player_entity: Local<Option<Entity>>,
    mut objective_entities: Local<Vec<Entity>>,
    level: Res<CreativeLevelResource>,
) {
    // Clear previous frame
    for entity in barrier_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }
    for entity in objective_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render barriers
    for barrier in &level.0.barriers {
        let color = if barrier.activated && barrier.barrier_type == BarrierType::Door {
            Color::rgba(0.6, 0.3, 0.0, 0.3) // Semi-transparent when open
        } else {
            barrier.barrier_type.color()
        };

        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(barrier.size),
                ..default()
            },
            transform: Transform::from_xyz(barrier.position.x, barrier.position.y, 0.0),
            ..default()
        }).id();
        barrier_entities.push(entity);
    }

    // Render player
    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(level.0.player.size),
                ..default()
            },
            transform: Transform::from_xyz(level.0.player.position.x, level.0.player.position.y, 1.0),
            ..default()
        },

    )).id();
    *player_entity = Some(entity);

    // Render objectives/goal
    let goal_pos = Vec2::new(500.0, 300.0);
    let goal_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 0.0),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(goal_pos.x, goal_pos.y, 1.0),
            ..default()
        },

    )).id();
    objective_entities.push(goal_entity);
}

fn display_creative_info(keyboard_input: Res<ButtonInput<KeyCode>>, level: Res<CreativeLevelResource>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        let completion = level.0.get_completion_percentage();

        println!("\n=== Creative Barriers Demo ===");
        println!("Completion: {:.1}%", completion);
        println!("Objectives:");
        for (i, objective) in level.0.objectives.iter().enumerate() {
            let status = if level.0.completed_objectives[i] { "✓" } else { "○" };
            match objective {
                Objective::ReachPoint(_, desc) |
                Objective::ActivateSwitches(_, desc) |
                Objective::AvoidHazards(desc) |
                Objective::UseToolsCreatively(desc) => {
                    println!("  {} {}", status, desc);
                }
            }
        }
        println!("\nBarriers:");
        println!("  Gray: Walls (block movement)");
        println!("  Red: Spikes (damage)");
        println!("  Bright Red: Lasers (heavy damage)");
        println!("  Cyan: Force fields (push back)");
        println!("  Magenta: Teleporters");
        println!("  Yellow: Switches");
        println!("  Brown: Doors (open when switch activated)");
        println!("\nTools:");
        println!("  1: Water Gun | 2: Fire Extinguisher");
        println!("  3: Teleport Gun | 4: Switch Tool");
        println!("  Left click: Use selected tool");
        println!("\nControls:");
        println!("  A/D: Move | Space: Jump");
        println!("  H: Show this info");
        println!("=======================\n");
    }
}
