// Example: Timing Animations
// Based on "Timing Animations" blog post
// https://www.slowrush.dev/news/timing-animations

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashMap;

// Animation timing and synchronization
// Demonstrates precise animation timing, event synchronization, and animation curves

#[derive(Clone, Debug)]
struct AnimationEvent {
    name: String,
    timestamp: f64,
    data: AnimationEventData,
}

#[derive(Clone, Debug)]
enum AnimationEventData {
    FrameChange { frame_index: usize },
    SoundTrigger { sound_id: String },
    ParticleEffect { effect_type: String, position: Vec2 },
    Damage { amount: f32, target: String },
    ProjectileSpawn { projectile_type: String, position: Vec2, velocity: Vec2 },
}

#[derive(Clone, Debug)]
struct TimedAnimation {
    name: String,
    duration: f64,
    events: Vec<AnimationEvent>,
    easing_curve: EasingCurve,
    loop_count: i32, // -1 for infinite
}

impl TimedAnimation {
    fn new(name: String, duration: f64, events: Vec<AnimationEvent>, easing_curve: EasingCurve) -> Self {
        Self {
            name,
            duration,
            events,
            easing_curve,
            loop_count: 1,
        }
    }

    fn get_events_at_time(&self, time: f64) -> Vec<&AnimationEvent> {
        let normalized_time = time / self.duration;
        let looped_time = if self.loop_count != 0 {
            time % self.duration
        } else {
            time.min(self.duration)
        };

        self.events.iter()
            .filter(|event| (event.timestamp - looped_time).abs() < 0.016) // Within one frame
            .collect()
    }

    fn get_progress(&self, time: f64) -> f32 {
        let normalized_time = time / self.duration;
        let looped_time = if self.loop_count != 0 {
            (time % self.duration) / self.duration
        } else {
            (time.min(self.duration)) / self.duration
        };

        self.easing_curve.apply(looped_time as f32)
    }

    fn is_finished(&self, time: f64) -> bool {
        if self.loop_count == -1 {
            false // Infinite loop
        } else if self.loop_count == 0 {
            time >= self.duration
        } else {
            time >= self.duration * self.loop_count as f64
        }
    }
}

#[derive(Clone, Debug)]
enum EasingCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Sine,
}

impl EasingCurve {
    fn apply(&self, t: f32) -> f32 {
        match self {
            EasingCurve::Linear => t,
            EasingCurve::EaseIn => t * t,
            EasingCurve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingCurve::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EasingCurve::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
            EasingCurve::Elastic => {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -(2.0f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
                }
            }
            EasingCurve::Sine => -(t * std::f32::consts::PI / 2.0).cos() / 2.0 + 0.5,
        }
    }
}

#[derive(Clone, Debug)]
struct AnimationTimeline {
    animations: HashMap<String, TimedAnimation>,
    active_animations: Vec<ActiveAnimation>,
    global_time: f64,
}

impl AnimationTimeline {
    fn new() -> Self {
        Self {
            animations: HashMap::new(),
            active_animations: Vec::new(),
            global_time: 0.0,
        }
    }

    fn add_animation(&mut self, animation: TimedAnimation) {
        self.animations.insert(animation.name.clone(), animation);
    }

    fn play_animation(&mut self, animation_name: &str, start_time: f64) -> Option<usize> {
        if let Some(animation) = self.animations.get(animation_name) {
            let active = ActiveAnimation {
                animation_name: animation_name.to_string(),
                start_time,
                speed: 1.0,
                id: self.active_animations.len(),
            };
            self.active_animations.push(active);
            Some(active.id)
        } else {
            None
        }
    }

    fn update(&mut self, dt: f64) -> Vec<AnimationEvent> {
        self.global_time += dt;
        let mut triggered_events = Vec::new();

        self.active_animations.retain_mut(|active| {
            if let Some(animation) = self.animations.get(&active.animation_name) {
                let animation_time = (self.global_time - active.start_time) * active.speed as f64;

                if animation.is_finished(animation_time) {
                    false // Remove finished animation
                } else {
                    // Check for events at current time
                    let events = animation.get_events_at_time(animation_time);
                    for event in events {
                        triggered_events.push(event.clone());
                    }
                    true // Keep animation
                }
            } else {
                false // Animation not found, remove
            }
        });

        triggered_events
    }

    fn get_animation_progress(&self, animation_id: usize) -> Option<f32> {
        if let Some(active) = self.active_animations.get(animation_id) {
            if let Some(animation) = self.animations.get(&active.animation_name) {
                let animation_time = (self.global_time - active.start_time) * active.speed as f64;
                Some(animation.get_progress(animation_time))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn set_animation_speed(&mut self, animation_id: usize, speed: f32) {
        if let Some(active) = self.active_animations.get_mut(animation_id) {
            active.speed = speed;
        }
    }
}

#[derive(Clone, Debug)]
struct ActiveAnimation {
    animation_name: String,
    start_time: f64,
    speed: f32,
    id: usize,
}

#[derive(Clone, Debug)]
struct CombatEntity {
    name: String,
    position: Vec2,
    health: f32,
    animation_timeline: AnimationTimeline,
    last_attack_time: f64,
    attack_cooldown: f64,
}

impl CombatEntity {
    fn new(name: String, position: Vec2) -> Self {
        let mut animation_timeline = AnimationTimeline::new();

        // Create attack animation with precise timing
        let attack_events = vec![
            AnimationEvent {
                name: "attack_start".to_string(),
                timestamp: 0.1,
                data: AnimationEventData::SoundTrigger { sound_id: "swing".to_string() },
            },
            AnimationEvent {
                name: "projectile_spawn".to_string(),
                timestamp: 0.3,
                data: AnimationEventData::ProjectileSpawn {
                    projectile_type: "arrow".to_string(),
                    position: Vec2::ZERO, // Will be updated
                    velocity: Vec2::new(200.0, 0.0), // Will be updated
                },
            },
            AnimationEvent {
                name: "attack_end".to_string(),
                timestamp: 0.8,
                data: AnimationEventData::SoundTrigger { sound_id: "impact".to_string() },
            },
        ];

        let attack_animation = TimedAnimation::new(
            "attack".to_string(),
            1.0,
            attack_events,
            EasingCurve::EaseOut,
        );

        animation_timeline.add_animation(attack_animation);

        Self {
            name,
            position,
            health: 100.0,
            animation_timeline,
            last_attack_time: 0.0,
            attack_cooldown: 1.5,
        }
    }

    fn attack(&mut self, target_position: Vec2, current_time: f64) -> bool {
        if current_time - self.last_attack_time >= self.attack_cooldown {
            // Start attack animation
            if let Some(animation_id) = self.animation_timeline.play_animation("attack", current_time) {
                self.last_attack_time = current_time;

                // Update projectile spawn position in the event
                if let Some(active) = self.animation_timeline.active_animations.last_mut() {
                    if let Some(animation) = self.animation_timeline.animations.get(&active.animation_name) {
                        for event in &mut animation.events {
                            if let AnimationEventData::ProjectileSpawn { position, velocity, .. } = &mut event.data {
                                *position = self.position;
                                let direction = (target_position - self.position).normalize();
                                *velocity = direction * 200.0;
                            }
                        }
                    }
                }

                return true;
            }
        }
        false
    }

    fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
        if self.health <= 0.0 {
            self.health = 0.0;
            // Could trigger death animation here
        }
    }
}

#[derive(Clone, Debug)]
struct Projectile {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    lifetime: f32,
    max_lifetime: f32,
    damage: f32,
    owner: String,
}

impl Projectile {
    fn new(id: u32, position: Vec2, velocity: Vec2, damage: f32, owner: String) -> Self {
        Self {
            id,
            position,
            velocity,
            lifetime: 3.0,
            max_lifetime: 3.0,
            damage,
            owner,
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        self.position += self.velocity * dt;
        self.lifetime -= dt;
        self.lifetime > 0.0
    }
}

#[derive(Resource)]
struct TimingDemo {
    entities: Vec<CombatEntity>,
    projectiles: Vec<Projectile>,
    timeline: AnimationTimeline,
    current_time: f64,
    next_projectile_id: u32,
    triggered_events: Vec<AnimationEvent>,
}

impl TimingDemo {
    fn new() -> Self {
        let mut entities = Vec::new();

        // Create archer
        let mut archer = CombatEntity::new("Archer".to_string(), Vec2::new(200.0, 250.0));

        // Create more detailed attack animation
        let detailed_attack_events = vec![
            AnimationEvent {
                name: "draw_bow".to_string(),
                timestamp: 0.1,
                data: AnimationEventData::SoundTrigger { sound_id: "bow_draw".to_string() },
            },
            AnimationEvent {
                name: "aim".to_string(),
                timestamp: 0.2,
                data: AnimationEventData::FrameChange { frame_index: 1 },
            },
            AnimationEvent {
                name: "release".to_string(),
                timestamp: 0.4,
                data: AnimationEventData::SoundTrigger { sound_id: "bow_release".to_string() },
            },
            AnimationEvent {
                name: "arrow_spawn".to_string(),
                timestamp: 0.45,
                data: AnimationEventData::ProjectileSpawn {
                    projectile_type: "arrow".to_string(),
                    position: Vec2::ZERO,
                    velocity: Vec2::ZERO,
                },
            },
            AnimationEvent {
                name: "follow_through".to_string(),
                timestamp: 0.6,
                data: AnimationEventData::FrameChange { frame_index: 2 },
            },
            AnimationEvent {
                name: "animation_end".to_string(),
                timestamp: 1.0,
                data: AnimationEventData::FrameChange { frame_index: 0 },
            },
        ];

        let detailed_attack = TimedAnimation::new(
            "detailed_attack".to_string(),
            1.0,
            detailed_attack_events,
            EasingCurve::EaseInOut,
        );

        archer.animation_timeline.add_animation(detailed_attack);

        entities.push(archer);

        // Create hound (target)
        let hound = CombatEntity::new("Hound".to_string(), Vec2::new(500.0, 250.0));
        entities.push(hound);

        let mut timeline = AnimationTimeline::new();

        // Add some global animations
        let global_events = vec![
            AnimationEvent {
                name: "level_start".to_string(),
                timestamp: 0.0,
                data: AnimationEventData::SoundTrigger { sound_id: "level_music".to_string() },
            },
            AnimationEvent {
                name: "combat_start".to_string(),
                timestamp: 2.0,
                data: AnimationEventData::ParticleEffect {
                    effect_type: "dust".to_string(),
                    position: Vec2::new(350.0, 200.0),
                },
            },
        ];

        let level_intro = TimedAnimation::new(
            "level_intro".to_string(),
            3.0,
            global_events,
            EasingCurve::Linear,
        );

        timeline.add_animation(level_intro);

        Self {
            entities,
            projectiles: Vec::new(),
            timeline,
            current_time: 0.0,
            next_projectile_id: 0,
            triggered_events: Vec::new(),
        }
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;

        // Update entity animations
        for entity in &mut self.entities {
            let events = entity.animation_timeline.update(dt);
            self.triggered_events.extend(events);
        }

        // Update global timeline
        let global_events = self.timeline.update(dt);
        self.triggered_events.extend(global_events);

        // Update projectiles
        self.projectiles.retain_mut(|proj| proj.update(dt as f32));

        // Process triggered events
        self.process_events();
    }

    fn process_events(&mut self) {
        for event in &self.triggered_events {
            match &event.data {
                AnimationEventData::ProjectileSpawn { projectile_type, position, velocity } => {
                    let projectile = Projectile::new(
                        self.next_projectile_id,
                        *position,
                        *velocity,
                        25.0,
                        "archer".to_string(),
                    );
                    self.projectiles.push(projectile);
                    self.next_projectile_id += 1;
                    println!("Projectile spawned at ({:.1}, {:.1})", position.x, position.y);
                }
                AnimationEventData::SoundTrigger { sound_id } => {
                    println!("Sound triggered: {}", sound_id);
                }
                AnimationEventData::ParticleEffect { effect_type, position } => {
                    println!("Particle effect: {} at ({:.1}, {:.1})", effect_type, position.x, position.y);
                }
                AnimationEventData::Damage { amount, target } => {
                    if let Some(entity) = self.entities.iter_mut().find(|e| e.name == *target) {
                        entity.take_damage(*amount);
                        println!("{} took {} damage, health: {}", target, amount, entity.health);
                    }
                }
                _ => {}
            }
        }
        self.triggered_events.clear();
    }

    fn start_combat(&mut self) {
        // Start level intro animation
        self.timeline.play_animation("level_intro", self.current_time);

        // Start archer attack after a delay
        if let Some(archer) = self.entities.first_mut() {
            archer.attack(Vec2::new(500.0, 250.0), self.current_time + 2.5);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Timing Animations - Precise Animation Timing".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(TimingDemo::new())
        .add_systems(Startup, setup_timing_demo)
        .add_systems(Update, (
            handle_timing_input,
            update_timing_demo,
            render_timing_demo,
            display_timing_info,
        ).chain())
        .run();
}

fn setup_timing_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_timing_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<TimingDemo>,
) {
    demo.update(1.0 / 60.0);

    if keyboard_input.just_pressed(KeyCode::Space) {
        demo.start_combat();
    }

    if keyboard_input.just_pressed(KeyCode::KeyA) {
        if let Some(archer) = demo.entities.first_mut() {
            archer.attack(Vec2::new(500.0, 250.0), demo.current_time);
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset demo
        *demo = TimingDemo::new();
    }
}

fn update_timing_demo(time: Res<Time>, mut demo: ResMut<TimingDemo>) {
    // Updates are handled in input system
}

fn render_timing_demo(
    mut commands: Commands,
    mut entity_entities: Local<Vec<Entity>>,
    mut projectile_entities: Local<Vec<Entity>>,
    demo: Res<TimingDemo>,
) {
    // Clear previous frame
    for entity in entity_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in projectile_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render entities
    for entity in &demo.entities {
        let color = if entity.name == "Archer" {
            Color::rgb(0.2, 0.6, 0.8) // Blue for archer
        } else {
            Color::rgb(0.8, 0.4, 0.2) // Orange for hound
        };

        let entity_bundle = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(40.0, 40.0)),
                    ..default()
                },
                transform: Transform::from_xyz(entity.position.x, entity.position.y, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    format!("{}: {:.0} HP", entity.name, entity.health),
                    TextStyle {
                        font_size: 12.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 25.0, 2.0),
                ..default()
            },
        )).id();
        entity_entities.push(entity_bundle);
    }

    // Render projectiles
    for projectile in &demo.projectiles {
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.8, 0.8, 0.2),
                custom_size: Some(Vec2::new(10.0, 5.0)),
                ..default()
            },
            transform: Transform::from_xyz(projectile.position.x, projectile.position.y, 1.0),
            ..default()
        }).id();
        projectile_entities.push(entity);
    }

    // Render active animations
    for active_anim in &demo.timeline.active_animations {
        if let Some(progress) = demo.timeline.get_animation_progress(active_anim.id) {
            let progress_bar = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.8, 0.2),
                    custom_size: Some(Vec2::new(100.0 * progress, 10.0)),
                    ..default()
                },
                transform: Transform::from_xyz(350.0, 550.0, 2.0),
                ..default()
            }).id();
            entity_entities.push(progress_bar);
        }
    }
}

fn display_timing_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<TimingDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        println!("\n=== Timing Animations Demo ===");
        println!("Current Time: {:.2}s", demo.current_time);
        println!("Active Projectiles: {}", demo.projectiles.len());

        println!("\nEntities:");
        for entity in &demo.entities {
            println!("  {} - HP: {:.0}, Position: ({:.1}, {:.1})",
                    entity.name, entity.health, entity.position.x, entity.position.y);
            println!("    Active Animations: {}", entity.animation_timeline.active_animations.len());
        }

        println!("\nGlobal Timeline:");
        println!("  Active Animations: {}", demo.timeline.active_animations.len());
        for active in &demo.timeline.active_animations {
            if let Some(progress) = demo.timeline.get_animation_progress(active.id) {
                println!("    {}: {:.1}% complete", active.animation_name, progress * 100.0);
            }
        }

        println!("\nControls:");
        println!("  Space: Start combat sequence");
        println!("  A: Manual archer attack");
        println!("  R: Reset demo");
        println!("  I: Show this info");
        println!("\nAnimation Events:");
        println!("  Watch console for triggered events!");
        println!("======================\n");
    }
}
