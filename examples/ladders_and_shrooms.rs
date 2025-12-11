// Example: Ladders and Shrooms
// Based on "Ladders and Shrooms" blog post
// https://www.slowrush.dev/news/ladders-and-shrooms

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Platforming with ladders and environmental objects
// Demonstrates climbing mechanics and interactive world elements

#[derive(Clone, Debug)]
enum WorldObject {
    Ladder,
    Mushroom,
    Platform,
    Spikes,
    Spring,
}

impl WorldObject {
    fn color(&self) -> Color {
        match self {
            WorldObject::Ladder => Color::rgb(0.6, 0.4, 0.2),
            WorldObject::Mushroom => Color::rgb(0.8, 0.3, 0.8),
            WorldObject::Platform => Color::rgb(0.5, 0.5, 0.5),
            WorldObject::Spikes => Color::rgb(0.8, 0.2, 0.2),
            WorldObject::Spring => Color::rgb(0.2, 0.8, 0.2),
        }
    }

    fn is_climbable(&self) -> bool {
        matches!(self, WorldObject::Ladder)
    }

    fn is_hazardous(&self) -> bool {
        matches!(self, WorldObject::Spikes)
    }

    fn is_bouncy(&self) -> bool {
        matches!(self, WorldObject::Spring)
    }
}

#[derive(Clone, Debug)]
struct InteractiveObject {
    object_type: WorldObject,
    position: Vec2,
    size: Vec2,
    bounds: Rect,
}

impl InteractiveObject {
    fn new(object_type: WorldObject, position: Vec2, size: Vec2) -> Self {
        let bounds = Rect::from_center_size(position, size);
        Self {
            object_type,
            position,
            size,
            bounds,
        }
    }

    fn contains_point(&self, point: Vec2) -> bool {
        self.bounds.contains(point)
    }
    fn intersects_rect(&self, other: Rect) -> bool {
        let intersection = self.bounds.intersect(other);
        intersection.width() > 0.0 && intersection.height() > 0.0
    }
}

#[derive(Clone, Debug)]
enum PlayerState {
    Grounded,
    Jumping,
    Falling,
    Climbing { ladder_x: f32 },
}

#[derive(Clone, Debug)]
struct Player {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
    state: PlayerState,
    can_climb: bool,
    jump_power: f32,
    move_speed: f32,
    health: f32,
    max_health: f32,
}

impl Player {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            size: Vec2::new(16.0, 24.0),
            state: PlayerState::Falling,
            can_climb: false,
            jump_power: 200.0,
            move_speed: 120.0,
            health: 100.0,
            max_health: 100.0,
        }
    }

    fn update(&mut self, dt: f32, input: &PlayerInput, world_objects: &[InteractiveObject]) {
        // Handle climbing state transitions
        self.update_climbing_state(world_objects);

        match &self.state {
            PlayerState::Grounded => {
                self.velocity.x = input.move_x * self.move_speed;

                if input.jump {
                    self.velocity.y = self.jump_power;
                    self.state = PlayerState::Jumping;
                }
            }
            PlayerState::Jumping => {
                self.velocity.x = input.move_x * self.move_speed;
                self.velocity.y -= 300.0 * dt; // Gravity

                if self.velocity.y <= 0.0 {
                    self.state = PlayerState::Falling;
                }
            }
            PlayerState::Falling => {
                self.velocity.x = input.move_x * self.move_speed;
                self.velocity.y -= 300.0 * dt;

                // Check for ground collision (simplified)
                if self.position.y <= -100.0 {
                    self.position.y = -100.0;
                    self.velocity.y = 0.0;
                    self.state = PlayerState::Grounded;
                }
            }
            PlayerState::Climbing { ladder_x } => {
                self.position.x = *ladder_x; // Lock to ladder
                self.velocity.x = 0.0;

                if input.move_y != 0.0 {
                    self.velocity.y = input.move_y * 80.0; // Climbing speed
                } else {
                    self.velocity.y = 0.0; // Stop on ladder
                }

                // Exit climbing if jump pressed or move away from ladder
                if input.jump || (input.move_x != 0.0 && !self.can_climb) {
                    self.state = PlayerState::Falling;
                    self.velocity.x = input.move_x * self.move_speed;
                }
            }
        }

        // Update position
        self.position += self.velocity * dt;

        // Check interactions with world objects
        self.check_object_interactions(world_objects, dt);
    }

    fn update_climbing_state(&mut self, world_objects: &[InteractiveObject]) {
        let player_rect = Rect::from_center_size(self.position, self.size);

        self.can_climb = false;

        for obj in world_objects {
            if obj.object_type.is_climbable() {
                if obj.intersects_rect(player_rect) {
                    self.can_climb = true;

                    // Enter climbing state if pressing up/down near ladder
                    if let PlayerState::Grounded | PlayerState::Falling = self.state {
                        // Auto-enter climbing when pressing up near ladder
                        // (would check input here in full implementation)
                    }
                    break;
                }
            }
        }

        // Exit climbing if no longer on ladder
        if let PlayerState::Climbing { .. } = self.state {
            if !self.can_climb {
                self.state = PlayerState::Falling;
            }
        }
    }

    fn check_object_interactions(&mut self, world_objects: &[InteractiveObject], dt: f32) {
        let player_rect = Rect::from_center_size(self.position, self.size);

        for obj in world_objects {
            if obj.intersects_rect(player_rect) {
                match obj.object_type {
                    WorldObject::Mushroom => {
                        // Bouncy mushroom
                        if self.velocity.y < 0.0 { // Only when falling down
                            self.velocity.y = 150.0; // Bounce up
                            self.state = PlayerState::Jumping;
                        }
                    }
                    WorldObject::Spikes => {
                        // Take damage from spikes
                        self.take_damage(20.0 * dt);
                    }
                    WorldObject::Spring => {
                        // Super bounce
                        if self.velocity.y < 0.0 {
                            self.velocity.y = 250.0;
                            self.state = PlayerState::Jumping;
                        }
                    }
                    WorldObject::Ladder => {
                        // Handle climbing (already handled in update_climbing_state)
                    }
                    WorldObject::Platform => {
                        // Platforms provide solid ground
                        // (collision handled separately in full physics)
                    }
                }
            }
        }
    }

    fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
        if self.health <= 0.0 {
            // Player death - respawn
            self.position = Vec2::new(0.0, 50.0);
            self.velocity = Vec2::ZERO;
            self.health = self.max_health;
            self.state = PlayerState::Falling;
        }
    }

    fn try_enter_climbing(&mut self, ladder_x: f32) {
        if self.can_climb {
            self.state = PlayerState::Climbing { ladder_x };
            self.velocity = Vec2::ZERO;
        }
    }
}

#[derive(Clone, Debug)]
struct PlayerInput {
    move_x: f32,
    move_y: f32,
    jump: bool,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            move_x: 0.0,
            move_y: 0.0,
            jump: false,
        }
    }
}

struct PlatformingWorld {
    player: Player,
    objects: Vec<InteractiveObject>,
    width: f32,
    height: f32,
}

impl PlatformingWorld {
    fn new(width: f32, height: f32) -> Self {
        let player = Player::new(Vec2::new(0.0, 50.0));

        let mut objects = Vec::new();

        // Create ground platforms
        objects.push(InteractiveObject::new(
            WorldObject::Platform,
            Vec2::new(0.0, -120.0),
            Vec2::new(400.0, 20.0),
        ));

        // Create floating platforms
        for i in 0..3 {
            let x = -150.0 + i as f32 * 150.0;
            objects.push(InteractiveObject::new(
                WorldObject::Platform,
                Vec2::new(x, -50.0 + i as f32 * 30.0),
                Vec2::new(80.0, 10.0),
            ));
        }

        // Create ladders
        objects.push(InteractiveObject::new(
            WorldObject::Ladder,
            Vec2::new(-100.0, -80.0),
            Vec2::new(10.0, 60.0),
        ));

        objects.push(InteractiveObject::new(
            WorldObject::Ladder,
            Vec2::new(100.0, -70.0),
            Vec2::new(10.0, 50.0),
        ));

        // Create mushrooms (bouncy)
        objects.push(InteractiveObject::new(
            WorldObject::Mushroom,
            Vec2::new(-50.0, -110.0),
            Vec2::new(20.0, 15.0),
        ));

        objects.push(InteractiveObject::new(
            WorldObject::Mushroom,
            Vec2::new(50.0, -110.0),
            Vec2::new(20.0, 15.0),
        ));

        // Create spikes (hazardous)
        objects.push(InteractiveObject::new(
            WorldObject::Spikes,
            Vec2::new(0.0, -115.0),
            Vec2::new(30.0, 10.0),
        ));

        // Create springs (super bouncy)
        objects.push(InteractiveObject::new(
            WorldObject::Spring,
            Vec2::new(150.0, -115.0),
            Vec2::new(15.0, 10.0),
        ));

        Self {
            player,
            objects,
            width,
            height,
        }
    }

    fn update(&mut self, dt: f32, input: &PlayerInput) {
        self.player.update(dt, input, &self.objects);
    }
}

#[derive(Resource)]
struct PlatformingWorldResource(PlatformingWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ladders and Shrooms - Platforming with Interactive Objects".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(PlatformingWorldResource(PlatformingWorld::new(800.0, 600.0)))
        .add_systems(Startup, setup_platforming_demo)
        .add_systems(Update, (
            handle_platforming_input,
            update_platforming_world,
            render_platforming_world,
            display_platforming_info,
        ).chain())
        .run();
}

fn setup_platforming_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_platforming_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<PlatformingWorldResource>,
) {
    let mut input = PlayerInput::default();

    // Movement
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.move_x = -1.0;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        input.move_x = 1.0;
    }

    // Climbing
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        input.move_y = 1.0;
        // for obj in &world.0.objects {
        //     if obj.object_type.is_climbable() && obj.contains_point(world.0.player.position) {
        //         world.0.player.try_enter_climbing(obj.position.x);
        //         break;
        //     }
        // }
        let player_position = world.0.player.position; // Save position to avoid borrow conflict
        let mut should_climb = false;
        let mut climb_x = 0.0;
        for obj in &world.0.objects {
            if obj.object_type.is_climbable() && obj.contains_point(player_position) {
                should_climb = true;
                climb_x = obj.position.x;
                break;
            }
        }
        if should_climb {
            world.0.player.try_enter_climbing(climb_x);
        }
    } else if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        input.move_y = -1.0;
    }

    // Jumping
    input.jump = keyboard_input.just_pressed(KeyCode::Space);

    world.0.update(1.0 / 60.0, &input); // Assume 60fps
}

fn update_platforming_world(
    mut world: ResMut<PlatformingWorldResource>,
    time: Res<Time>,
) {
    // World update is handled in input handler for simplicity
}

fn render_platforming_world(
    mut commands: Commands,
    mut object_entities: Local<Vec<Entity>>,
    mut player_entity: Local<Option<Entity>>,
    world: Res<PlatformingWorldResource>,
) {
    // Clear previous frame
    for entity in object_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }

    // Render world objects
    for obj in &world.0.objects {
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: obj.object_type.color(),
                custom_size: Some(obj.size),
                ..default()
            },
            transform: Transform::from_xyz(obj.position.x, obj.position.y, 0.0),
            ..default()
        }).id();
        object_entities.push(entity);
    }

    // Render player
    let state_color = match world.0.player.state {
        PlayerState::Grounded => Color::rgb(0.2, 0.8, 0.2),
        PlayerState::Jumping => Color::rgb(0.2, 0.6, 0.8),
        PlayerState::Falling => Color::rgb(0.8, 0.4, 0.2),
        PlayerState::Climbing { .. } => Color::rgb(0.8, 0.8, 0.2),
    };

    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: state_color,
                custom_size: Some(world.0.player.size),
                ..default()
            },
            transform: Transform::from_xyz(world.0.player.position.x, world.0.player.position.y, 1.0),
            ..default()
        },

    )).id();
    *player_entity = Some(entity);
}

fn display_platforming_info(keyboard_input: Res<ButtonInput<KeyCode>>, world: Res<PlatformingWorldResource>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        let state_name = match &world.0.player.state {
            PlayerState::Grounded => "Grounded",
            PlayerState::Jumping => "Jumping",
            PlayerState::Falling => "Falling",
            PlayerState::Climbing { .. } => "Climbing",
        };

        println!("\n=== Ladders and Shrooms Platforming Demo ===");
        println!("Player State: {}", state_name);
        println!("Position: ({:.1}, {:.1})", world.0.player.position.x, world.0.player.position.y);
        println!("Health: {:.0}/{}", world.0.player.health, world.0.player.max_health);
        println!("Can Climb: {}", world.0.player.can_climb);
        println!("");
        println!("World Objects:");
        println!("- Brown: Ladders (climb with W/S)");
        println!("- Purple: Mushrooms (bounce)");
        println!("- Red: Spikes (damage)");
        println!("- Green: Springs (super bounce)");
        println!("- Gray: Platforms");
        println!("");
        println!("Controls:");
        println!("A/D: Move left/right");
        println!("W/S: Climb up/down ladders");
        println!("Space: Jump");
        println!("H: Show this info");
        println!("============================\n");
    }
}
