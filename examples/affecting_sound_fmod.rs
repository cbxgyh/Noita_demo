// Example: Affecting Sound (with FMOD)
// Based on "Affecting Sound (with FMOD)" blog post
// https://www.slowrush.dev/news/affecting-sound-with-fmod

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Advanced audio using FMOD-like features
// Demonstrates dynamic audio manipulation, DSP effects, and adaptive audio

#[derive(Clone, Debug)]
struct FMODEvent {
    name: String,
    parameters: std::collections::HashMap<String, FMODParameter>,
    position: Vec2,
    volume: f32,
    pitch: f32,
    is_3d: bool,
    min_distance: f32,
    max_distance: f32,
}

impl FMODEvent {
    fn new(name: String) -> Self {
        Self {
            name,
            parameters: std::collections::HashMap::new(),
            position: Vec2::ZERO,
            volume: 1.0,
            pitch: 1.0,
            is_3d: true,
            min_distance: 10.0,
            max_distance: 100.0,
        }
    }

    fn set_parameter(&mut self, name: &str, value: FMODParameter) {
        self.parameters.insert(name.to_string(), value);
    }

    fn get_parameter(&self, name: &str) -> Option<&FMODParameter> {
        self.parameters.get(name)
    }
}

#[derive(Clone, Debug)]
enum FMODParameter {
    Float(f32),
    Int(i32),
    Bool(bool),
    Label(String),
}

#[derive(Clone, Debug)]
struct FMODInstance {
    event_name: String,
    instance_id: u32,
    playback_state: PlaybackState,
    start_time: f64,
    position: Vec2,
    parameters: std::collections::HashMap<String, FMODParameter>,
    volume: f32,
    pitch: f32,
    dsp_effects: Vec<DSPEffect>,
}

#[derive(Clone, Debug, PartialEq)]
enum PlaybackState {
    Stopped,
    Starting,
    Playing,
    Sustaining,
    Stopping,
    StoppedComplete,
}

#[derive(Clone, Debug)]
enum DSPEffect {
    LowPass { cutoff: f32 },
    HighPass { cutoff: f32 },
    Reverb { decay: f32, wet: f32 },
    Distortion { level: f32 },
    Chorus { rate: f32, depth: f32 },
    Echo { delay: f32, feedback: f32 },
    PitchShift { pitch: f32 },
}

impl DSPEffect {
    fn process_sample(&self, sample: f32, time: f64) -> f32 {
        match self {
            DSPEffect::LowPass { cutoff } => {
                // Simple low-pass filter simulation
                sample * (*cutoff / 20000.0).min(1.0)
            }
            DSPEffect::HighPass { cutoff } => {
                // Simple high-pass filter simulation
                sample * (1.0 - *cutoff / 20000.0).max(0.0)
            }
            DSPEffect::Reverb { decay, wet } => {
                // Simple reverb simulation
                sample * (1.0 - wet) + sample * wet * (*decay as f64 * time).sin() as f32 * 0.3
            }
            DSPEffect::Distortion { level } => {
                // Simple distortion
                (sample * (1.0 + level)).tanh()
            }
            DSPEffect::Chorus { rate, depth } => {
                // Simple chorus
                let modulation = (time * *rate as f64 * 2.0 * std::f64::consts::PI).sin() as f32;
                sample + sample * modulation * depth
            }
            DSPEffect::Echo { delay, feedback } => {
                // Simple echo (would need proper delay buffer in real implementation)
                sample * (1.0 - feedback) + sample * feedback * 0.5
            }
            DSPEffect::PitchShift { pitch } => {
                // Pitch shifting (simplified)
                sample * pitch.min(2.0).max(0.5)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct FMODStudio {
    events: std::collections::HashMap<String, FMODEvent>,
    instances: Vec<FMODInstance>,
    global_parameters: std::collections::HashMap<String, FMODParameter>,
    buses: Vec<FMODBus>,
    vca: Vec<FMODVCA>,
    listener_position: Vec2,
    current_time: f64,
    next_instance_id: u32,
}

#[derive(Clone, Debug)]
struct FMODBus {
    name: String,
    volume: f32,
    muted: bool,
    dsp_effects: Vec<DSPEffect>,
}

#[derive(Clone, Debug)]
struct FMODVCA {
    name: String,
    volume: f32,
    buses: Vec<String>,
}

impl FMODStudio {
    fn new() -> Self {
        let mut events = std::collections::HashMap::new();

        // Initialize game events
        let mut footsteps = FMODEvent::new("event:/character/footsteps".to_string());
        footsteps.set_parameter("surface", FMODParameter::Label("stone".to_string()));
        footsteps.set_parameter("speed", FMODParameter::Float(1.0));
        events.insert(footsteps.name.clone(), footsteps);

        let mut weapon_fire = FMODEvent::new("event:/weapons/fire".to_string());
        weapon_fire.set_parameter("weapon_type", FMODParameter::Label("bow".to_string()));
        weapon_fire.set_parameter("power", FMODParameter::Float(1.0));
        events.insert(weapon_fire.name.clone(), weapon_fire);

        let mut ambient_wind = FMODEvent::new("event:/ambient/wind".to_string());
        ambient_wind.set_parameter("intensity", FMODParameter::Float(0.5));
        ambient_wind.is_3d = false; // 2D ambient sound
        events.insert(ambient_wind.name.clone(), ambient_wind);

        let mut music_combat = FMODEvent::new("event:/music/combat".to_string());
        music_combat.set_parameter("intensity", FMODParameter::Float(0.0));
        music_combat.set_parameter("tempo", FMODParameter::Float(120.0));
        music_combat.is_3d = false;
        events.insert(music_combat.name.clone(), music_combat);

        let mut explosion = FMODEvent::new("event:/effects/explosion".to_string());
        explosion.set_parameter("size", FMODParameter::Label("large".to_string()));
        explosion.set_parameter("distance", FMODParameter::Float(0.0));
        events.insert(explosion.name.clone(), explosion);

        Self {
            events,
            instances: Vec::new(),
            global_parameters: std::collections::HashMap::new(),
            buses: vec![
                FMODBus {
                    name: "SFX".to_string(),
                    volume: 1.0,
                    muted: false,
                    dsp_effects: vec![DSPEffect::Reverb { decay: 0.8, wet: 0.3 }],
                },
                FMODBus {
                    name: "Music".to_string(),
                    volume: 0.7,
                    muted: false,
                    dsp_effects: vec![DSPEffect::LowPass { cutoff: 8000.0 }],
                },
                FMODBus {
                    name: "Ambient".to_string(),
                    volume: 0.5,
                    muted: false,
                    dsp_effects: vec![DSPEffect::HighPass { cutoff: 200.0 }],
                },
            ],
            vca: vec![
                FMODVCA {
                    name: "Master".to_string(),
                    volume: 1.0,
                    buses: vec!["SFX".to_string(), "Music".to_string(), "Ambient".to_string()],
                },
            ],
            listener_position: Vec2::ZERO,
            current_time: 0.0,
            next_instance_id: 0,
        }
    }

    fn play_event(&mut self, event_name: &str, position: Vec2) -> Option<u32> {
        if let Some(event) = self.events.get(event_name) {
            let instance_id = self.next_instance_id;
            self.next_instance_id += 1;

            let mut instance = FMODInstance {
                event_name: event_name.to_string(),
                instance_id,
                playback_state: PlaybackState::Starting,
                start_time: self.current_time,
                position,
                parameters: event.parameters.clone(),
                volume: event.volume,
                pitch: event.pitch,
                dsp_effects: Vec::new(),
            };

            // Set initial playback state
            instance.playback_state = PlaybackState::Playing;

            self.instances.push(instance);

            println!("🎵 FMOD Event started: {} (ID: {})", event_name, instance_id);
            Some(instance_id)
        } else {
            println!("⚠️  FMOD Event not found: {}", event_name);
            None
        }
    }

    fn stop_event(&mut self, instance_id: u32, fade_out: bool) {
        if let Some(instance) = self.instances.iter_mut().find(|i| i.instance_id == instance_id) {
            if fade_out {
                instance.playback_state = PlaybackState::Stopping;
                // Add fade out DSP effect
                instance.dsp_effects.push(DSPEffect::LowPass { cutoff: 1000.0 });
            } else {
                instance.playback_state = PlaybackState::Stopped;
            }
        }
    }

    fn set_event_parameter(&mut self, instance_id: u32, parameter: &str, value: FMODParameter) {
        if let Some(instance) = self.instances.iter_mut().find(|i| i.instance_id == instance_id) {
            instance.parameters.insert(parameter.to_string(), value.clone());

            // Apply parameter-based DSP effects
            match (parameter, &value) {
                ("intensity", FMODParameter::Float(intensity)) => {
                    if *intensity > 0.7 {
                        instance.dsp_effects.push(DSPEffect::Distortion { level: *intensity - 0.7 });
                    }
                }
                ("underwater", FMODParameter::Bool(true)) => {
                    instance.dsp_effects.push(DSPEffect::LowPass { cutoff: 2000.0 });
                }
                ("tempo", FMODParameter::Float(tempo)) => {
                    instance.pitch = tempo / 120.0; // Normalize to 120 BPM
                }
                _ => {}
            }
        }
    }

    fn set_global_parameter(&mut self, parameter: &str, value: FMODParameter) {
        self.global_parameters.insert(parameter.to_string(), value.clone());

        // Apply global effects
        match (parameter, &value) {
            ("pause", FMODParameter::Bool(true)) => {
                // Pause all instances
                for instance in &mut self.instances {
                    instance.volume = 0.0;
                }
            }
            ("master_volume", FMODParameter::Float(volume)) => {
                for vca in &mut self.vca {
                    if vca.name == "Master" {
                        vca.volume = *volume;
                    }
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;

        // Update 3D audio positioning
        for instance in &mut self.instances {
            if let Some(event) = self.events.get(&instance.event_name) {
                if event.is_3d {
                    let distance = self.listener_position.distance(instance.position);
                    let min_dist = event.min_distance;
                    let max_dist = event.max_distance;

                    if distance <= min_dist {
                        instance.volume = event.volume;
                    } else if distance >= max_dist {
                        instance.volume = 0.0;
                    } else {
                        let attenuation = 1.0 - ((distance - min_dist) / (max_dist - min_dist));
                        instance.volume = event.volume * attenuation;
                    }

                    // Distance-based panning
                    let direction = instance.position - self.listener_position;
                    if direction.x.abs() > 0.1 {
                        // Simple stereo panning (would need proper HRTF in real FMOD)
                        let pan = (direction.x / distance).clamp(-1.0, 1.0);
                        // Apply pan to DSP chain
                    }
                }
            }
        }

        // Process DSP effects
        for instance in &mut self.instances {
            // Apply bus effects
            if instance.event_name.contains("/music/") {
                if let Some(music_bus) = self.buses.iter().find(|b| b.name == "Music") {
                    for effect in &music_bus.dsp_effects {
                        // Apply bus DSP
                    }
                }
            }
        }

        // Remove stopped instances
        self.instances.retain(|instance| {
            let should_remove = matches!(instance.playback_state, PlaybackState::Stopped | PlaybackState::StoppedComplete);
            if should_remove {
                println!("🔇 FMOD Event stopped: {} (ID: {})", instance.event_name, instance.instance_id);
            }
            !should_remove
        });
    }

    fn get_active_instances(&self) -> Vec<&FMODInstance> {
        self.instances.iter()
            .filter(|i| i.playback_state == PlaybackState::Playing || i.playback_state == PlaybackState::Stopping)
            .collect()
    }
}

#[derive(Clone, Debug)]
struct GameCharacter {
    name: String,
    position: Vec2,
    velocity: Vec2,
    health: f32,
    is_moving: bool,
    is_in_combat: bool,
    last_footstep_time: f64,
    footsteps_instance: Option<u32>,
}

impl GameCharacter {
    fn new(name: String, position: Vec2) -> Self {
        Self {
            name,
            position,
            velocity: Vec2::ZERO,
            health: 100.0,
            is_moving: false,
            is_in_combat: false,
            last_footstep_time: 0.0,
            footsteps_instance: None,
        }
    }

    fn update(&mut self, dt: f64, fmod: &mut FMODStudio) {
        // Apply movement
        self.position += self.velocity * dt as f32;

        // Update movement state
        self.is_moving = self.velocity.length() > 10.0;

        // Handle footsteps
        if self.is_moving && fmod.current_time - self.last_footstep_time > 0.3 {
            if self.footsteps_instance.is_none() {
                self.footsteps_instance = fmod.play_event("event:/character/footsteps", self.position);
            }

            // Update footstep parameters
            if let Some(instance_id) = self.footsteps_instance {
                let speed = (self.velocity.length() / 100.0).clamp(0.5, 2.0);
                fmod.set_event_parameter(instance_id, "speed", FMODParameter::Float(speed));
            }

            self.last_footstep_time = fmod.current_time;
        } else if !self.is_moving {
            if let Some(instance_id) = self.footsteps_instance {
                fmod.stop_event(instance_id, true);
                self.footsteps_instance = None;
            }
        }

        // Update FMOD listener position
        fmod.listener_position = self.position;
    }

    fn attack(&mut self, fmod: &mut FMODStudio) {
        fmod.play_event("event:/weapons/fire", self.position);
        self.is_in_combat = true;
    }

    fn take_damage(&mut self, damage: f32, fmod: &mut FMODStudio) {
        self.health -= damage;
        if self.health <= 0.0 {
            fmod.play_event("event:/effects/explosion", self.position);
        }
    }
}

#[derive(Resource)]
struct FMODDemo {
    fmod: FMODStudio,
    player: GameCharacter,
    enemies: Vec<GameCharacter>,
    music_instance: Option<u32>,
    ambient_instance: Option<u32>,
    current_time: f64,
}

impl FMODDemo {
    fn new() -> Self {
        let mut fmod = FMODStudio::new();
        let player = GameCharacter::new("Player".to_string(), Vec2::new(200.0, 250.0));

        let mut enemies = Vec::new();
        enemies.push(GameCharacter::new("Enemy1".to_string(), Vec2::new(500.0, 250.0)));
        enemies.push(GameCharacter::new("Enemy2".to_string(), Vec2::new(600.0, 250.0)));

        // Start ambient sound
        let ambient = fmod.play_event("event:/ambient/wind", Vec2::new(400.0, 300.0));

        Self {
            fmod,
            player,
            enemies,
            music_instance: None,
            ambient_instance: ambient,
            current_time: 0.0,
        }
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;
        self.fmod.update(dt);

        // Update characters
        self.player.update(dt, &mut self.fmod);

        for enemy in &mut self.enemies {
            // Simple enemy AI
            let target_pos = self.player.position;
            let direction = (target_pos - enemy.position).normalize();
            enemy.velocity = direction * 50.0;
            enemy.update(dt, &mut self.fmod);
        }

        // Update music intensity based on combat
        let in_combat = self.player.is_in_combat || self.enemies.iter().any(|e| e.is_in_combat);
        if in_combat && self.music_instance.is_none() {
            self.music_instance = self.fmod.play_event("event:/music/combat", Vec2::ZERO);
        }

        if let Some(music_id) = self.music_instance {
            let intensity = if in_combat { 1.0 } else { 0.0 };
            self.fmod.set_event_parameter(music_id, "intensity", FMODParameter::Float(intensity));
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Affecting Sound with FMOD - Advanced Audio & DSP".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(FMODDemo::new())
        .add_systems(Startup, setup_fmod_demo)
        .add_systems(Update, (
            handle_fmod_input,
            update_fmod_demo,
            render_fmod_demo,
            display_fmod_info,
        ).chain())
        .run();
}

fn setup_fmod_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_fmod_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<FMODDemo>,
) {
    demo.update(1.0 / 60.0);

    // Player movement
    if keyboard_input.pressed(KeyCode::KeyA) {
        demo.player.velocity.x = -100.0;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        demo.player.velocity.x = 100.0;
    } else {
        demo.player.velocity.x *= 0.8;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        demo.player.velocity.y = 150.0;
    }
    // if keyboard_input.just_pressed(KeyCode::KeyF) {
    // demo.player.attack(&mut demo.fmod); ## 双 mut
    //  1. 使用unsafe
    //  2. 分开
    // // Use unsafe to split borrows - safe because player and fmod are different fields
    // let player = &mut demo.player as *mut GameCharacter;
    // let fmod = &mut demo.fmod;
    // unsafe {
    //     (*player).attack(fmod);
    // }
    // }
    // FMOD events
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        let pos = demo.player.position;
        demo.fmod.play_event("event:/weapons/fire", pos);
        demo.player.is_in_combat = true;
    }

    if keyboard_input.just_pressed(KeyCode::KeyE) {
        let player_pos = demo.player.position;
        demo.fmod.play_event("event:/effects/explosion", player_pos);
    }

    // Parameter control
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        demo.fmod.set_global_parameter("underwater", FMODParameter::Bool(true));
        println!("Underwater effect enabled");
    }

    if keyboard_input.just_pressed(KeyCode::KeyG) {
        demo.fmod.set_global_parameter("underwater", FMODParameter::Bool(false));
        println!("Underwater effect disabled");
    }

    // Volume control
    if keyboard_input.just_pressed(KeyCode::Equal) {
        demo.fmod.set_global_parameter("master_volume", FMODParameter::Float(1.0));
    }

    if keyboard_input.just_pressed(KeyCode::Minus) {
        demo.fmod.set_global_parameter("master_volume", FMODParameter::Float(0.5));
    }
}

fn update_fmod_demo(time: Res<Time>, mut demo: ResMut<FMODDemo>) {
    // Updates are handled in input system
}

fn render_fmod_demo(
    mut commands: Commands,
    mut entity_entities: Local<Vec<Entity>>,
    mut fmod_entities: Local<Vec<Entity>>,
    demo: Res<FMODDemo>,
) {
    // Clear previous frame
    for entity in entity_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in fmod_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render ground
    let ground_entity = commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.4, 0.4, 0.4),
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
                color: Color::srgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(30.0, 40.0)),
                ..default()
            },
            transform: Transform::from_xyz(demo.player.position.x, demo.player.position.y, 1.0),
            ..default()
        }
    )).id();
    entity_entities.push(player_entity);

    // Render enemies
    for enemy in &demo.enemies {
        let enemy_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.8, 0.2, 0.2),
                    custom_size: Some(Vec2::new(25.0, 35.0)),
                    ..default()
                },
                transform: Transform::from_xyz(enemy.position.x, enemy.position.y, 1.0),
                ..default()
            },
        )).id();
        entity_entities.push(enemy_entity);
    }

    // Render active FMOD instances
    let active_instances = demo.fmod.get_active_instances();
    for (i, instance) in active_instances.iter().enumerate() {
        let volume_indicator = instance.volume;
        let color = if instance.event_name.contains("music") {
            Color::srgb(0.8, 0.2, 0.8) // Purple for music
        } else if instance.event_name.contains("ambient") {
            Color::srgb(0.2, 0.8, 0.8) // Cyan for ambient
        } else {
            Color::srgb(0.8, 0.8, 0.2) // Yellow for SFX
        };
        let [r,g,b,a] =color.to_srgba().to_f32_array();
        let fmod_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(r, g, b, volume_indicator * 0.5),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(instance.position.x, instance.position.y + 60.0 + i as f32 * 20.0, 2.0),
                ..default()
            },
        )).id();
        fmod_entities.push(fmod_entity);
    }
}

fn display_fmod_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<FMODDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        let active_instances = demo.fmod.get_active_instances();

        println!("\n=== FMOD Studio Demo ===");
        println!("Active Instances: {}", active_instances.len());
        println!("Total Events: {}", demo.fmod.events.len());
        println!("Buses: {}", demo.fmod.buses.len());
        println!("VCAs: {}", demo.fmod.vca.len());
        println!("Listener Position: ({:.1}, {:.1})", demo.fmod.listener_position.x, demo.fmod.listener_position.y);

        println!("\nActive FMOD Events:");
        for instance in &active_instances {
            println!("  {} (ID:{}): vol={:.2}, pitch={:.1}, DSP effects: {}",
                    instance.event_name, instance.instance_id, instance.volume,
                    instance.pitch, instance.dsp_effects.len());

            for (param_name, param_value) in &instance.parameters {
                match param_value {
                    FMODParameter::Float(val) => println!("    {}: {:.2}", param_name, val),
                    FMODParameter::Int(val) => println!("    {}: {}", param_name, val),
                    FMODParameter::Bool(val) => println!("    {}: {}", param_name, val),
                    FMODParameter::Label(val) => println!("    {}: {}", param_name, val),
                }
            }
        }

        println!("\nFMOD Buses:");
        for bus in &demo.fmod.buses {
            println!("  {}: vol={:.1}, muted={}, DSP: {}", bus.name, bus.volume, bus.muted, bus.dsp_effects.len());
        }

        println!("\nGlobal Parameters:");
        for (param_name, param_value) in &demo.fmod.global_parameters {
            match param_value {
                FMODParameter::Float(val) => println!("  {}: {:.2}", param_name, val),
                FMODParameter::Bool(val) => println!("  {}: {}", param_name, val),
                _ => println!("  {}: {:?}", param_name, param_value),
            }
        }

        println!("\nControls:");
        println!("  A/D: Move | Space: Jump | F: Attack | E: Explosion");
        println!("  U: Underwater effect | G: Disable underwater");
        println!("  +/-: Master volume");
        println!("  I: Show this info");
        println!("\nNote: FMOD features simulated - check console for audio events!");
        println!("======================\n");
    }
}
