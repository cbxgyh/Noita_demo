// Example: Effecting Sound
// Based on "Effecting Sound" blog post
// https://www.slowrush.dev/news/effecting-sound

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Sound effects implementation
// Demonstrates audio integration, procedural sound generation, and sound triggering

#[derive(Clone, Debug)]
struct SoundEffect {
    id: String,
    volume: f32,
    pitch: f32,
    pan: f32, // -1.0 (left) to 1.0 (right)
    duration: f32,
    loop_: bool,
    fade_in: f32,
    fade_out: f32,
}

impl SoundEffect {
    fn new(id: String) -> Self {
        Self {
            id,
            volume: 1.0,
            pitch: 1.0,
            pan: 0.0,
            duration: 1.0,
            loop_: false,
            fade_in: 0.0,
            fade_out: 0.0,
        }
    }

    fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }

    fn looping(mut self, loop_: bool) -> Self {
        self.loop_ = loop_;
        self
    }

    fn with_fade(mut self, fade_in: f32, fade_out: f32) -> Self {
        self.fade_in = fade_in;
        self.fade_out = fade_out;
        self
    }
}

#[derive(Clone, Debug)]
struct PlayingSound {
    sound_id: String,
    instance_id: u32,
    start_time: f64,
    position: Vec2,
    volume: f32,
    pitch: f32,
    pan: f32,
    is_looping: bool,
    fade_in_duration: f32,
    fade_out_duration: f32,
    fade_out_start: Option<f64>,
}

impl PlayingSound {
    fn new(sound: SoundEffect, instance_id: u32, start_time: f64, position: Vec2) -> Self {
        Self {
            sound_id: sound.id,
            instance_id,
            start_time,
            position,
            volume: sound.volume,
            pitch: sound.pitch,
            pan: sound.pan,
            is_looping: sound.loop_,
            fade_in_duration: sound.fade_in,
            fade_out_duration: sound.fade_out,
            fade_out_start: None,
        }
    }

    fn get_current_volume(&self, current_time: f64) -> f32 {
        let elapsed = current_time - self.start_time;

        // Fade in
        if self.fade_in_duration > 0.0 && elapsed < self.fade_in_duration as f64 {
            let fade_progress = (elapsed / self.fade_in_duration as f64) as f32;
            return self.volume * fade_progress;
        }

        // Fade out
        if let Some(fade_start) = self.fade_out_start {
            let fade_elapsed = current_time - fade_start;
            if fade_elapsed < self.fade_out_duration as f64 {
                let fade_progress = 1.0 - (fade_elapsed / self.fade_out_duration as f64) as f32;
                return self.volume * fade_progress;
            } else {
                return 0.0; // Fully faded out
            }
        }

        self.volume
    }

    fn start_fade_out(&mut self, current_time: f64) {
        self.fade_out_start = Some(current_time);
    }

    fn is_finished(&self, current_time: f64) -> bool {
        if let Some(fade_start) = self.fade_out_start {
            let fade_elapsed = current_time - fade_start;
            fade_elapsed >= self.fade_out_duration as f64
        } else {
            false // Not fading out
        }
    }
}

#[derive(Clone, Debug)]
struct AudioEngine {
    sound_library: std::collections::HashMap<String, SoundEffect>,
    playing_sounds: Vec<PlayingSound>,
    master_volume: f32,
    next_instance_id: u32,
    listener_position: Vec2,
    current_time: f64,
}

impl AudioEngine {
    fn new() -> Self {
        let mut sound_library = std::collections::HashMap::new();

        // Initialize sound library with game sounds
        sound_library.insert("bow_draw".to_string(), SoundEffect::new("bow_draw".to_string())
            .with_volume(0.7)
            .with_pitch(1.0));

        sound_library.insert("bow_release".to_string(), SoundEffect::new("bow_release".to_string())
            .with_volume(0.8)
            .with_pitch(0.9));

        sound_library.insert("arrow_impact".to_string(), SoundEffect::new("arrow_impact".to_string())
            .with_volume(0.6)
            .with_pitch(1.2));

        sound_library.insert("footstep".to_string(), SoundEffect::new("footstep".to_string())
            .with_volume(0.4)
            .with_pitch(1.0));

        sound_library.insert("jump".to_string(), SoundEffect::new("jump".to_string())
            .with_volume(0.5)
            .with_pitch(1.1));

        sound_library.insert("land".to_string(), SoundEffect::new("land".to_string())
            .with_volume(0.6)
            .with_pitch(0.8));

        sound_library.insert("explosion".to_string(), SoundEffect::new("explosion".to_string())
            .with_volume(1.0)
            .with_pitch(0.5));

        sound_library.insert("background_music".to_string(), SoundEffect::new("background_music".to_string())
            .with_volume(0.3)
            .with_pitch(1.0)
            .looping(true)
            .with_fade(2.0, 2.0));

        sound_library.insert("ui_click".to_string(), SoundEffect::new("ui_click".to_string())
            .with_volume(0.8)
            .with_pitch(1.0));

        sound_library.insert("pickup".to_string(), SoundEffect::new("pickup".to_string())
            .with_volume(0.7)
            .with_pitch(1.3));

        Self {
            sound_library,
            playing_sounds: Vec::new(),
            master_volume: 1.0,
            next_instance_id: 0,
            listener_position: Vec2::ZERO,
            current_time: 0.0,
        }
    }

    fn play_sound(&mut self, sound_id: &str, position: Vec2) -> Option<u32> {
        if let Some(sound) = self.sound_library.get(sound_id) {
            let instance_id = self.next_instance_id;
            self.next_instance_id += 1;

            let playing_sound = PlayingSound::new(sound.clone(), instance_id, self.current_time, position);
            self.playing_sounds.push(playing_sound);

            // Simulate audio playback
            println!("🔊 Playing sound: {} (ID: {}) at ({:.1}, {:.1})",
                    sound_id, instance_id, position.x, position.y);

            Some(instance_id)
        } else {
            println!("⚠️  Sound not found: {}", sound_id);
            None
        }
    }

    fn stop_sound(&mut self, instance_id: u32) {
        if let Some(sound) = self.playing_sounds.iter_mut().find(|s| s.instance_id == instance_id) {
            sound.start_fade_out(self.current_time);
        }
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;

        // Update 3D audio positioning
        for sound in &mut self.playing_sounds {
            let distance = self.listener_position.distance(sound.position);
            let max_distance = 300.0; // Max audible distance

            if distance > max_distance {
                sound.volume = 0.0;
            } else {
                // Distance-based volume attenuation
                let attenuation = 1.0 - (distance / max_distance);
                sound.volume = sound.get_current_volume(self.current_time) * attenuation;

                // Distance-based panning
                let direction = sound.position - self.listener_position;
                if direction.x.abs() > 0.1 {
                    sound.pan = (direction.x / distance).clamp(-1.0, 1.0);
                }
            }
        }

        // Remove finished sounds
        self.playing_sounds.retain(|sound| {
            let finished = sound.is_finished(self.current_time);
            if finished {
                println!("🔇 Sound finished: {} (ID: {})", sound.sound_id, sound.instance_id);
            }
            !finished
        });
    }

    fn set_listener_position(&mut self, position: Vec2) {
        self.listener_position = position;
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    fn get_active_sounds(&self) -> Vec<&PlayingSound> {
        self.playing_sounds.iter().filter(|s| s.get_current_volume(self.current_time) > 0.0).collect()
    }
}

#[derive(Clone, Debug)]
struct GameEntity {
    name: String,
    position: Vec2,
    velocity: Vec2,
    on_ground: bool,
    last_footstep_time: f64,
    footstep_interval: f64,
}

impl GameEntity {
    fn new(name: String, position: Vec2) -> Self {
        Self {
            name,
            position,
            velocity: Vec2::ZERO,
            on_ground: true,
            last_footstep_time: 0.0,
            footstep_interval: 0.3,
        }
    }

    fn update(&mut self, dt: f64, audio_engine: &mut AudioEngine) {
        // Apply gravity
        if !self.on_ground {
            self.velocity.y -= 300.0 * dt as f32;
        }

        // Update position
        self.position += self.velocity * dt as f32;

        // Ground collision
        if self.position.y <= 50.0 {
            self.position.y = 50.0;
            self.velocity.y = 0.0;
            self.on_ground = true;

            // Play landing sound if falling fast
            if self.velocity.y < -50.0 {
                audio_engine.play_sound("land", self.position);
            }
        }

        // Footstep sounds
        if self.on_ground && self.velocity.x.abs() > 10.0 {
            if audio_engine.current_time - self.last_footstep_time > self.footstep_interval {
                audio_engine.play_sound("footstep", self.position);
                self.last_footstep_time = audio_engine.current_time;
            }
        }
    }

    fn jump(&mut self, audio_engine: &mut AudioEngine) {
        if self.on_ground {
            self.velocity.y = 200.0;
            self.on_ground = false;
            audio_engine.play_sound("jump", self.position);
        }
    }

    fn attack(&mut self, target_pos: Vec2, audio_engine: &mut AudioEngine) {
        // Simulate bow attack
        audio_engine.play_sound("bow_draw", self.position);

        // Arrow release after delay (would be handled by animation system)
        let arrow_release_time = audio_engine.current_time + 0.5;
        // In real implementation, this would be scheduled

        // Simulate arrow impact
        let impact_time = audio_engine.current_time + 1.0;
        let impact_pos = target_pos;
        // In real implementation, this would be scheduled
    }
}

#[derive(Resource)]
struct SoundDemo {
    audio_engine: AudioEngine,
    player: GameEntity,
    enemies: Vec<GameEntity>,
    background_music_id: Option<u32>,
    current_time: f64,
}

impl SoundDemo {
    fn new() -> Self {
        let mut audio_engine = AudioEngine::new();

        let player = GameEntity::new("Player".to_string(), Vec2::new(200.0, 200.0));
        let mut enemies = Vec::new();

        enemies.push(GameEntity::new("Enemy1".to_string(), Vec2::new(500.0, 200.0)));
        enemies.push(GameEntity::new("Enemy2".to_string(), Vec2::new(600.0, 200.0)));

        // Start background music
        let music_id = audio_engine.play_sound("background_music", Vec2::new(400.0, 300.0));

        Self {
            audio_engine,
            player,
            enemies,
            background_music_id: music_id,
            current_time: 0.0,
        }
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;

        // Update audio engine
        self.audio_engine.update(dt);
        self.audio_engine.set_listener_position(self.player.position);

        // Update player
        self.player.update(dt, &mut self.audio_engine);

        // Update enemies (simple AI)
        for enemy in &mut self.enemies {
            // Simple patrol AI
            let patrol_center = Vec2::new(550.0, 200.0);
            let direction = (patrol_center - enemy.position).normalize();
            enemy.velocity.x = direction.x * 50.0;

            enemy.update(dt, &mut self.audio_engine);
        }
    }

    fn trigger_sound_event(&mut self, event_type: &str) {
        match event_type {
            "explosion" => {
                self.audio_engine.play_sound("explosion", Vec2::new(400.0, 300.0));
                // Create screen shake effect (visual only in this demo)
            }
            "ui_click" => {
                self.audio_engine.play_sound("ui_click", self.player.position);
            }
            "pickup" => {
                self.audio_engine.play_sound("pickup", self.player.position);
            }
            "attack" => {
                self.player.attack(Vec2::new(500.0, 200.0), &mut self.audio_engine);
            }
            _ => {}
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Effecting Sound - Audio Integration & Sound Effects".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(SoundDemo::new())
        .add_systems(Startup, setup_sound_demo)
        .add_systems(Update, (
            handle_sound_input,
            update_sound_demo,
            render_sound_demo,
            display_sound_info,
        ).chain())
        .run();
}

fn setup_sound_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_sound_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<SoundDemo>,
) {
    demo.update(1.0 / 60.0);

    // Player movement
    if keyboard_input.pressed(KeyCode::KeyA) {
        demo.player.velocity.x = -100.0;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        demo.player.velocity.x = 100.0;
    } else {
        demo.player.velocity.x *= 0.8; // Friction
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        demo.player.jump(&mut demo.audio_engine);
    }

    // Sound event triggers
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        demo.trigger_sound_event("explosion");
    }

    if keyboard_input.just_pressed(KeyCode::KeyF) {
        demo.trigger_sound_event("attack");
    }

    if keyboard_input.just_pressed(KeyCode::KeyP) {
        demo.trigger_sound_event("pickup");
    }

    if keyboard_input.just_pressed(KeyCode::KeyU) {
        demo.trigger_sound_event("ui_click");
    }

    // Volume control
    if keyboard_input.just_pressed(KeyCode::Equal) {
        let new_volume = (demo.audio_engine.master_volume + 0.1).min(1.0);
        demo.audio_engine.set_master_volume(new_volume);
        println!("Master volume: {:.1}", new_volume);
    }

    if keyboard_input.just_pressed(KeyCode::Minus) {
        let new_volume = (demo.audio_engine.master_volume - 0.1).max(0.0);
        demo.audio_engine.set_master_volume(new_volume);
        println!("Master volume: {:.1}", new_volume);
    }

    // Stop background music
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        if let Some(music_id) = demo.background_music_id {
            demo.audio_engine.stop_sound(music_id);
            demo.background_music_id = None;
            println!("Background music stopped");
        }
    }
}

fn update_sound_demo(time: Res<Time>, mut demo: ResMut<SoundDemo>) {
    // Updates are handled in input system
}

fn render_sound_demo(
    mut commands: Commands,
    mut entity_entities: Local<Vec<Entity>>,
    mut sound_entities: Local<Vec<Entity>>,
    demo: Res<SoundDemo>,
) {
    // Clear previous frame
    for entity in entity_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in sound_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render ground
    let ground_entity = commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.4, 0.4, 0.4),
            custom_size: Some(Vec2::new(800.0, 20.0)),
            ..default()
        },
        transform: Transform::from_xyz(400.0, 30.0, 0.0),
        ..default()
    }).id();
    entity_entities.push(ground_entity);

    // Render player
    let player_entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(30.0, 40.0)),
                ..default()
            },
            transform: Transform::from_xyz(demo.player.position.x, demo.player.position.y, 1.0),
            ..default()
        },
        Text2dBundle {
            text: Text::from_section(
                "Player",
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
    entity_entities.push(player_entity);

    // Render enemies
    for enemy in &demo.enemies {
        let enemy_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.2, 0.2),
                    custom_size: Some(Vec2::new(25.0, 35.0)),
                    ..default()
                },
                transform: Transform::from_xyz(enemy.position.x, enemy.position.y, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    &enemy.name,
                    TextStyle {
                        font_size: 10.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 20.0, 2.0),
                    ..default()
            },
        )).id();
        entity_entities.push(enemy_entity);
    }

    // Render active sounds
    let active_sounds = demo.audio_engine.get_active_sounds();
    for (i, sound) in active_sounds.iter().enumerate() {
        let volume = sound.get_current_volume(demo.current_time);
        let size = 10.0 + volume * 20.0;

        let sound_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.8, 0.8, 0.2, volume * 0.5),
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                transform: Transform::from_xyz(sound.position.x, sound.position.y + 50.0 + i as f32 * 15.0, 2.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    format!("{} ({:.1})", sound.sound_id, volume),
                    TextStyle {
                        font_size: 8.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ..default()
            },
        )).id();
        sound_entities.push(sound_entity);
    }
}

fn display_sound_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<SoundDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        let active_sounds = demo.audio_engine.get_active_sounds();

        println!("\n=== Effecting Sound Demo ===");
        println!("Active Sounds: {}/{}", active_sounds.len(), demo.audio_engine.playing_sounds.len());
        println!("Master Volume: {:.1}", demo.audio_engine.master_volume);
        println!("Listener Position: ({:.1}, {:.1})", demo.audio_engine.listener_position.x, demo.audio_engine.listener_position.y);

        println!("\nSound Library: {} sounds", demo.audio_engine.sound_library.len());
        for (sound_id, sound) in &demo.audio_engine.sound_library {
            println!("  {}: vol={:.1}, pitch={:.1}, loop={}",
                    sound_id, sound.volume, sound.pitch, sound.loop_);
        }

        println!("\nCurrently Playing:");
        for sound in &active_sounds {
            let volume = sound.get_current_volume(demo.current_time);
            println!("  {} (ID:{}): vol={:.2}, pan={:.1}, pos=({:.1}, {:.1})",
                    sound.sound_id, sound.instance_id, volume, sound.pan,
                    sound.position.x, sound.position.y);
        }

        println!("\nControls:");
        println!("  A/D: Move | Space: Jump");
        println!("  E: Explosion | F: Attack | P: Pickup | U: UI Click");
        println!("  +/-: Volume | M: Stop music");
        println!("  I: Show this info");
        println!("\nNote: Sounds are simulated - check console for audio events!");
        println!("======================\n");
    }
}
