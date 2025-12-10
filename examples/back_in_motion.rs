// Example: Back in Motion
// Based on "Back in Motion" blog post
// https://www.slowrush.dev/news/back-in-motion

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashMap;

// Animation and motion systems
// Demonstrates sprite animation, state machines, and smooth motion transitions

#[derive(Clone, Debug, PartialEq)]
enum AnimationState {
    Idle,
    Walking,
    Running,
    Jumping,
    Falling,
    Attacking,
    Hurt,
    Dead,
}

#[derive(Clone, Debug)]
struct AnimationFrame {
    sprite_index: usize,
    duration: f32,
}

#[derive(Clone, Debug)]
struct AnimationSequence {
    name: String,
    frames: Vec<AnimationFrame>,
    loop_: bool,
}

impl AnimationSequence {
    fn new(name: String, frames: Vec<AnimationFrame>, loop_: bool) -> Self {
        Self { name, frames, loop_ }
    }

    fn total_duration(&self) -> f32 {
        self.frames.iter().map(|f| f.duration).sum()
    }
}

#[derive(Clone, Debug)]
struct AnimationController {
    sequences: HashMap<String, AnimationSequence>,
    current_sequence: Option<String>,
    current_frame: usize,
    frame_timer: f32,
    playback_speed: f32,
    facing_right: bool,
}

impl AnimationController {
    fn new() -> Self {
        Self {
            sequences: HashMap::new(),
            current_sequence: None,
            current_frame: 0,
            frame_timer: 0.0,
            playback_speed: 1.0,
            facing_right: true,
        }
    }

    fn add_sequence(&mut self, sequence: AnimationSequence) {
        self.sequences.insert(sequence.name.clone(), sequence);
    }

    fn play(&mut self, sequence_name: &str, restart: bool) {
        if let Some(sequence) = self.sequences.get(sequence_name) {
            if restart || self.current_sequence.as_ref() != Some(&sequence.name) {
                self.current_sequence = Some(sequence.name.clone());
                self.current_frame = 0;
                self.frame_timer = 0.0;
            }
        }
    }

    fn update(&mut self, dt: f32) {
        if let Some(sequence_name) = &self.current_sequence {
            if let Some(sequence) = self.sequences.get(sequence_name) {
                if !sequence.frames.is_empty() {
                    self.frame_timer += dt * self.playback_speed;

                    let current_frame_duration = sequence.frames[self.current_frame].duration;
                    if self.frame_timer >= current_frame_duration {
                        self.frame_timer -= current_frame_duration;
                        self.current_frame += 1;

                        if self.current_frame >= sequence.frames.len() {
                            if sequence.loop_ {
                                self.current_frame = 0;
                            } else {
                                self.current_frame = sequence.frames.len() - 1;
                                // Could trigger animation end event here
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_current_sprite_index(&self) -> usize {
        if let Some(sequence_name) = &self.current_sequence {
            if let Some(sequence) = self.sequences.get(sequence_name) {
                if let Some(frame) = sequence.frames.get(self.current_frame) {
                    return frame.sprite_index;
                }
            }
        }
        0
    }

    fn set_facing(&mut self, right: bool) {
        self.facing_right = right;
    }

    fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.1);
    }
}

#[derive(Clone, Debug)]
struct MotionController {
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    max_speed: Vec2,
    friction: Vec2,
    gravity: f32,
    on_ground: bool,
    jump_power: f32,
    can_jump: bool,
}

impl MotionController {
    fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            max_speed: Vec2::new(200.0, 400.0),
            friction: Vec2::new(0.8, 1.0),
            gravity: 600.0,
            on_ground: false,
            jump_power: 250.0,
            can_jump: true,
        }
    }

    fn apply_force(&mut self, force: Vec2) {
        self.acceleration += force;
    }

    fn jump(&mut self) {
        if self.can_jump && self.on_ground {
            self.velocity.y = -self.jump_power;
            self.on_ground = false;
            self.can_jump = false;
        }
    }

    fn update(&mut self, dt: f32) {
        // Apply gravity
        if !self.on_ground {
            self.velocity.y += self.gravity * dt;
        }

        // Apply acceleration to velocity
        self.velocity += self.acceleration * dt;

        // Clamp velocity
        self.velocity.x = self.velocity.x.clamp(-self.max_speed.x, self.max_speed.x);
        self.velocity.y = self.velocity.y.clamp(-self.max_speed.y, self.max_speed.y);

        // Apply friction
        self.velocity.x *= self.friction.x;

        // Update position
        self.position += self.velocity * dt;

        // Reset acceleration
        self.acceleration = Vec2::ZERO;
    }

    fn check_ground_collision(&mut self, ground_y: f32) {
        if self.position.y >= ground_y {
            self.position.y = ground_y;
            self.velocity.y = 0.0;
            self.on_ground = true;
            self.can_jump = true;
        } else {
            self.on_ground = false;
        }
    }
}

#[derive(Clone, Debug)]
struct Character {
    animation_controller: AnimationController,
    motion_controller: MotionController,
    size: Vec2,
    state: AnimationState,
    last_state: AnimationState,
}

impl Character {
    fn new(position: Vec2) -> Self {
        let mut animation_controller = AnimationController::new();

        // Create animation sequences
        animation_controller.add_sequence(AnimationSequence::new(
            "idle".to_string(),
            vec![
                AnimationFrame { sprite_index: 0, duration: 0.5 },
                AnimationFrame { sprite_index: 1, duration: 0.5 },
                AnimationFrame { sprite_index: 2, duration: 0.5 },
            ],
            true,
        ));

        animation_controller.add_sequence(AnimationSequence::new(
            "walk".to_string(),
            vec![
                AnimationFrame { sprite_index: 3, duration: 0.15 },
                AnimationFrame { sprite_index: 4, duration: 0.15 },
                AnimationFrame { sprite_index: 5, duration: 0.15 },
                AnimationFrame { sprite_index: 6, duration: 0.15 },
            ],
            true,
        ));

        animation_controller.add_sequence(AnimationSequence::new(
            "run".to_string(),
            vec![
                AnimationFrame { sprite_index: 7, duration: 0.1 },
                AnimationFrame { sprite_index: 8, duration: 0.1 },
                AnimationFrame { sprite_index: 9, duration: 0.1 },
            ],
            true,
        ));

        animation_controller.add_sequence(AnimationSequence::new(
            "jump".to_string(),
            vec![
                AnimationFrame { sprite_index: 10, duration: 0.2 },
            ],
            false,
        ));

        animation_controller.add_sequence(AnimationSequence::new(
            "fall".to_string(),
            vec![
                AnimationFrame { sprite_index: 11, duration: 0.1 },
            ],
            false,
        ));

        animation_controller.add_sequence(AnimationSequence::new(
            "attack".to_string(),
            vec![
                AnimationFrame { sprite_index: 12, duration: 0.1 },
                AnimationFrame { sprite_index: 13, duration: 0.1 },
                AnimationFrame { sprite_index: 14, duration: 0.15 },
            ],
            false,
        ));

        let mut motion_controller = MotionController::new();
        motion_controller.position = position;

        Self {
            animation_controller,
            motion_controller,
            size: Vec2::new(32.0, 32.0),
            state: AnimationState::Idle,
            last_state: AnimationState::Idle,
        }
    }

    fn update(&mut self, dt: f32, input: &CharacterInput) {
        // Update motion
        if input.move_left {
            self.motion_controller.apply_force(Vec2::new(-400.0, 0.0));
            self.animation_controller.set_facing(false);
        }
        if input.move_right {
            self.motion_controller.apply_force(Vec2::new(400.0, 0.0));
            self.animation_controller.set_facing(true);
        }
        if input.jump {
            self.motion_controller.jump();
        }

        self.motion_controller.update(dt);
        self.motion_controller.check_ground_collision(200.0);

        // Determine animation state
        self.last_state = self.state.clone();
        self.state = self.determine_animation_state(input);

        // Update animation if state changed
        if self.state != self.last_state {
            self.play_animation_for_state(&self.state);
        }

        // Adjust animation speed based on movement
        let speed_factor = (self.motion_controller.velocity.x.abs() / 100.0).max(0.5);
        self.animation_controller.set_speed(speed_factor);

        self.animation_controller.update(dt);
    }

    fn determine_animation_state(&self, input: &CharacterInput) -> AnimationState {
        if !self.motion_controller.on_ground {
            if self.motion_controller.velocity.y < 0.0 {
                AnimationState::Jumping
            } else {
                AnimationState::Falling
            }
        } else if input.attack {
            AnimationState::Attacking
        } else {
            let speed = self.motion_controller.velocity.x.abs();
            if speed > 150.0 {
                AnimationState::Running
            } else if speed > 10.0 {
                AnimationState::Walking
            } else {
                AnimationState::Idle
            }
        }
    }

    fn play_animation_for_state(&mut self, state: &AnimationState) {
        let sequence_name = match state {
            AnimationState::Idle => "idle",
            AnimationState::Walking => "walk",
            AnimationState::Running => "run",
            AnimationState::Jumping => "jump",
            AnimationState::Falling => "fall",
            AnimationState::Attacking => "attack",
            AnimationState::Hurt => "hurt",
            AnimationState::Dead => "dead",
        };

        self.animation_controller.play(sequence_name, true);
    }

    fn get_sprite_transform(&self) -> Transform {
        let mut transform = Transform::from_xyz(
            self.motion_controller.position.x,
            self.motion_controller.position.y,
            0.0,
        );

        if !self.animation_controller.facing_right {
            transform.scale.x = -1.0;
        }

        transform
    }

    fn get_current_sprite_index(&self) -> usize {
        self.animation_controller.get_current_sprite_index()
    }
}

#[derive(Clone, Debug)]
struct CharacterInput {
    move_left: bool,
    move_right: bool,
    jump: bool,
    attack: bool,
}

impl Default for CharacterInput {
    fn default() -> Self {
        Self {
            move_left: false,
            move_right: false,
            jump: false,
            attack: false,
        }
    }
}

#[derive(Resource)]
struct AnimationDemo {
    characters: Vec<Character>,
    sprite_sheets: Vec<Handle<Image>>,
    current_time: f64,
}

impl AnimationDemo {
    fn new() -> Self {
        let mut characters = Vec::new();

        // Create multiple characters at different positions
        characters.push(Character::new(Vec2::new(200.0, 200.0)));
        characters.push(Character::new(Vec2::new(400.0, 200.0)));
        characters.push(Character::new(Vec2::new(600.0, 200.0)));

        Self {
            characters,
            sprite_sheets: Vec::new(),
            current_time: 0.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Back in Motion - Animation & Motion Systems".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(AnimationDemo::new())
        .add_systems(Startup, setup_animation_demo)
        .add_systems(Update, (
            handle_animation_input,
            update_animation_demo,
            render_animation_demo,
            display_animation_info,
        ).chain())
        .run();
}

fn setup_animation_demo(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // In a real game, you'd load actual sprite sheets
    // For demo purposes, we'll use colored rectangles
}

fn handle_animation_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<AnimationDemo>,
) {
    demo.update(1.0 / 60.0, &CharacterInput::default());

    // Control first character manually
    if let Some(character) = demo.characters.get_mut(0) {
        let mut input = CharacterInput::default();

        input.move_left = keyboard_input.pressed(KeyCode::KeyA);
        input.move_right = keyboard_input.pressed(KeyCode::KeyD);
        input.jump = keyboard_input.just_pressed(KeyCode::Space);
        input.attack = keyboard_input.just_pressed(KeyCode::KeyF);

        character.update(1.0 / 60.0, &input);
    }

    // Make other characters move automatically for demo
    for i in 1..demo.characters.len() {
        if let Some(character) = demo.characters.get_mut(i) {
            let mut input = CharacterInput::default();

            // Simple AI: move back and forth
            let time = demo.current_time;
            let should_move_right = (time * 2.0).sin() > 0.0;
            input.move_right = should_move_right;
            input.move_left = !should_move_right;

            // Random jumping
            input.jump = (time * 3.0).sin() > 0.8;

            character.update(1.0 / 60.0, &input);
        }
    }
}

fn update_animation_demo(time: Res<Time>, mut demo: ResMut<AnimationDemo>) {
    demo.current_time += time.delta_seconds() as f64;
}

fn render_animation_demo(
    mut commands: Commands,
    mut character_entities: Local<Vec<Entity>>,
    demo: Res<AnimationDemo>,
) {
    // Clear previous frame
    for entity in character_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render ground
    let ground_entity = commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.4, 0.4, 0.4),
            custom_size: Some(Vec2::new(800.0, 20.0)),
            ..default()
        },
        transform: Transform::from_xyz(400.0, 210.0, -1.0),
        ..default()
    }).id();
    character_entities.push(ground_entity);

    // Render characters
    for (i, character) in demo.characters.iter().enumerate() {
        let color = match i {
            0 => Color::rgb(0.2, 0.8, 0.2), // Player character - green
            1 => Color::rgb(0.2, 0.4, 0.8), // AI character 1 - blue
            2 => Color::rgb(0.8, 0.2, 0.8), // AI character 2 - purple
            _ => Color::rgb(0.5, 0.5, 0.5),
        };

        let sprite_index = character.get_current_sprite_index();

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(character.size),
                    ..default()
                },
                transform: character.get_sprite_transform(),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    format!("Frame: {}", sprite_index),
                    TextStyle {
                        font_size: 12.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 20.0, 1.0),
                ..default()
            },
        )).id();
        character_entities.push(entity);
    }
}

fn display_animation_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<AnimationDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        println!("\n=== Back in Motion Demo ===");
        println!("Characters: {}", demo.characters.len());

        for (i, character) in demo.characters.iter().enumerate() {
            println!("\nCharacter {}:", i + 1);
            println!("  Position: ({:.1}, {:.1})", character.motion_controller.position.x, character.motion_controller.position.y);
            println!("  Velocity: ({:.1}, {:.1})", character.motion_controller.velocity.x, character.motion_controller.velocity.y);
            println!("  State: {:?}", character.state);
            println!("  On Ground: {}", character.motion_controller.on_ground);
            println!("  Current Animation: {:?}", character.animation_controller.current_sequence);
            println!("  Current Frame: {}", character.animation_controller.current_frame);
            println!("  Facing Right: {}", character.animation_controller.facing_right);
        }

        println!("\nControls:");
        println!("  A/D: Move left/right (character 1)");
        println!("  Space: Jump (character 1)");
        println!("  F: Attack (character 1)");
        println!("  I: Show this info");
        println!("\nAnimation States:");
        println!("  Green: Player controlled");
        println!("  Blue/Purple: AI controlled");
        println!("======================\n");
    }
}
