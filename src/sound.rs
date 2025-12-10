use std::collections::HashMap;
use bevy::prelude::*;
use crate::atoms::{AtomType, AtomWorldResource};

// Simple sound system inspired by "Effecting Sound" and "Affecting Sound" blog posts
// Using a simplified FMOD-like approach for atomic reactions

#[derive(Resource)]
pub struct SoundManager {
    pub sounds_enabled: bool,
    pub master_volume: f32,
    pub reaction_sounds: HashMap<ReactionType, SoundEvent>,
    pub ambient_sounds: Vec<AmbientSound>,
}

#[derive(Resource, Default)]
pub struct AudioBuses(pub HashMap<String, AudioBus>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ReactionType {
    FireIgnite,
    AcidCorrode,
    WaterSplash,
    Explosion,
    SteamHiss,
    PoisonDamage,
    MagicCast,
    PlayerJump,
    PlayerLand,
}

#[derive(Clone)]
pub struct SoundEvent {
    pub name: String,
    pub volume: f32,
    pub pitch: f32,
    pub duration: f32,
    pub position: Vec2,
    pub timestamp: f64,
}

#[derive(Clone)]
pub struct AmbientSound {
    pub sound_type: AmbientType,
    pub volume: f32,
    pub position: Vec2,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub enum AmbientType {
    FireCrackle,
    WaterFlow,
    Wind,
    CaveEcho,
}

// Audio bus system (simplified FMOD approach)
#[derive(Clone)]
pub struct AudioBus {
    pub name: String,
    pub volume: f32,
    pub effects: Vec<AudioEffect>,
}

#[derive(Clone)]
pub enum AudioEffect {
    Reverb { room_size: f32, damping: f32 },
    Echo { delay: f32, decay: f32 },
    LowPass { cutoff: f32 },
    HighPass { cutoff: f32 },
    Distortion { amount: f32 },
}

impl Default for SoundManager {
    fn default() -> Self {
        let mut reaction_sounds = HashMap::new();

        // Setup reaction sounds
        reaction_sounds.insert(ReactionType::FireIgnite, SoundEvent {
            name: "fire_ignite".to_string(),
            volume: 0.7,
            pitch: 1.0,
            duration: 0.5,
            position: Vec2::ZERO,
            timestamp: 0.0,
        });

        reaction_sounds.insert(ReactionType::AcidCorrode, SoundEvent {
            name: "acid_corrode".to_string(),
            volume: 0.6,
            pitch: 0.8,
            duration: 1.0,
            position: Vec2::ZERO,
            timestamp: 0.0,
        });

        reaction_sounds.insert(ReactionType::WaterSplash, SoundEvent {
            name: "water_splash".to_string(),
            volume: 0.5,
            pitch: 1.2,
            duration: 0.3,
            position: Vec2::ZERO,
            timestamp: 0.0,
        });

        reaction_sounds.insert(ReactionType::Explosion, SoundEvent {
            name: "explosion".to_string(),
            volume: 1.0,
            pitch: 0.5,
            duration: 2.0,
            position: Vec2::ZERO,
            timestamp: 0.0,
        });

        reaction_sounds.insert(ReactionType::SteamHiss, SoundEvent {
            name: "steam_hiss".to_string(),
            volume: 0.4,
            pitch: 1.5,
            duration: 3.0,
            position: Vec2::ZERO,
            timestamp: 0.0,
        });

        Self {
            sounds_enabled: true,
            master_volume: 0.8,
            reaction_sounds,
            ambient_sounds: Vec::new(),
        }
    }
}

// System to monitor atomic reactions and trigger sounds
pub fn monitor_atomic_reactions(
    world: Res<AtomWorldResource>,
    mut sound_manager: ResMut<SoundManager>,
    time: Res<Time>,
) {
    if !sound_manager.sounds_enabled {
        return;
    }

    let current_time = time.elapsed_seconds_f64();

    // Check for fire reactions
    check_fire_reactions(&world.0, &mut sound_manager, current_time);

    // Check for acid reactions
    check_acid_reactions(&world.0, &mut sound_manager, current_time);

    // Check for water reactions
    check_water_reactions(&world.0, &mut sound_manager, current_time);

    // Check for explosions
    check_explosion_reactions(&world.0, &mut sound_manager, current_time);

    // Update ambient sounds
    update_ambient_sounds(&world.0, &mut sound_manager);
}

fn check_fire_reactions(world: &crate::atoms::AtomWorld, sound_manager: &mut SoundManager, current_time: f64) {
    let mut fire_positions = Vec::new();
    let mut fire_count = 0;

    for y in 0..world.height {
        for x in 0..world.width {
            if let Some(atom) = world.get_atom(x as i32, y as i32) {
                if atom.atom_type == AtomType::Fire {
                    fire_count += 1;
                    if fire_count % 50 == 0 { // Sample every 50 fire atoms
                        fire_positions.push(Vec2::new(x as f32, y as f32));
                    }
                }
            }
        }
    }

    if fire_count > 10 {
        // Trigger fire sound with averaged position
        let avg_pos = fire_positions.iter().fold(Vec2::ZERO, |acc, pos| acc + *pos) /
                     fire_positions.len() as f32;

        trigger_reaction_sound(sound_manager, ReactionType::FireIgnite, avg_pos, current_time);
    }
}

fn check_acid_reactions(world: &crate::atoms::AtomWorld, sound_manager: &mut SoundManager, current_time: f64) {
    let mut acid_positions = Vec::new();
    let mut acid_count = 0;

    for y in 0..world.height {
        for x in 0..world.width {
            if let Some(atom) = world.get_atom(x as i32, y as i32) {
                if atom.atom_type == AtomType::Acid {
                    acid_count += 1;
                    if acid_count % 30 == 0 {
                        acid_positions.push(Vec2::new(x as f32, y as f32));
                    }
                }
            }
        }
    }

    if acid_count > 5 {
        let avg_pos = acid_positions.iter().fold(Vec2::ZERO, |acc, pos| acc + *pos) /
                     acid_positions.len().max(1) as f32;

        trigger_reaction_sound(sound_manager, ReactionType::AcidCorrode, avg_pos, current_time);
    }
}

fn check_water_reactions(world: &crate::atoms::AtomWorld, sound_manager: &mut SoundManager, current_time: f64) {
    let mut water_positions = Vec::new();
    let mut moving_water_count = 0;

    for y in 0..world.height {
        for x in 0..world.width {
            if let Some(atom) = world.get_atom(x as i32, y as i32) {
                if atom.atom_type == AtomType::Water && atom.velocity.length_squared() > 1.0 {
                    moving_water_count += 1;
                    if moving_water_count % 20 == 0 {
                        water_positions.push(Vec2::new(x as f32, y as f32));
                    }
                }
            }
        }
    }

    if moving_water_count > 3 {
        let avg_pos = water_positions.iter().fold(Vec2::ZERO, |acc, pos| acc + *pos) /
                     water_positions.len().max(1) as f32;

        trigger_reaction_sound(sound_manager, ReactionType::WaterSplash, avg_pos, current_time);
    }
}

fn check_explosion_reactions(world: &crate::atoms::AtomWorld, sound_manager: &mut SoundManager, current_time: f64) {
    // Check for rapid expansion of gas/fire atoms (explosion-like behavior)
    // This is a simplified detection
}

fn update_ambient_sounds(world: &crate::atoms::AtomWorld, sound_manager: &mut SoundManager) {
    sound_manager.ambient_sounds.clear();

    // Scan for areas with concentrated atom types
    let scan_step = 20; // Sample every 20 pixels

    for base_y in (0..world.height).step_by(scan_step) {
        for base_x in (0..world.width).step_by(scan_step) {
            let mut atom_counts = std::collections::HashMap::new();

            // Count atoms in this region
            for y in base_y..(base_y + scan_step).min(world.height) {
                for x in base_x..(base_x + scan_step).min(world.width) {
                    if let Some(atom) = world.get_atom(x as i32, y as i32) {
                        *atom_counts.entry(atom.atom_type).or_insert(0) += 1;
                    }
                }
            }

            // Create ambient sounds for concentrated areas
            let region_size = (scan_step * scan_step) as f32;
            let center_pos = Vec2::new(base_x as f32 + scan_step as f32 / 2.0, base_y as f32 + scan_step as f32 / 2.0);

            for (atom_type, count) in atom_counts {
                let concentration = count as f32 / region_size;

                if concentration > 0.3 { // 30% concentration threshold
                    let ambient_type = match atom_type {
                        AtomType::Fire => Some(AmbientType::FireCrackle),
                        AtomType::Water => Some(AmbientType::WaterFlow),
                        _ => None,
                    };

                    if let Some(sound_type) = ambient_type {
                        sound_manager.ambient_sounds.push(AmbientSound {
                            sound_type,
                            volume: (concentration * 0.5).min(1.0),
                            position: center_pos,
                            radius: scan_step as f32,
                        });
                    }
                }
            }
        }
    }
}

fn trigger_reaction_sound(sound_manager: &mut SoundManager, reaction_type: ReactionType, position: Vec2, current_time: f64) {
    if let Some(sound_template) = sound_manager.reaction_sounds.get(&reaction_type) {
        // Check if we recently played this sound (avoid spam)
        if current_time - sound_template.timestamp > sound_template.duration as f64 {
            println!("Playing sound: {} at position {:?}", sound_template.name, position);

            // In a real implementation, this would queue the sound for playback
            // For now, we just print to console
        }
    }
}

// Player sound effects
pub fn player_sound_effects(
    mut player_query: Query<(&crate::game::Player, &mut crate::magic::MagicUser), Changed<crate::game::Player>>,
    mut sound_manager: ResMut<SoundManager>,
    time: Res<Time>,
) {
    for (player, mut magic_user) in player_query.iter_mut() {
        // Jump sound
        if player.is_grounded {
            // Could trigger landing sound here
        }
    }
}

// Magic casting sounds
pub fn magic_casting_sounds(
    mut spell_events: EventReader<SpellCastEvent>,
    mut sound_manager: ResMut<SoundManager>,
) {
    for event in spell_events.read() {
        trigger_reaction_sound(&mut sound_manager, ReactionType::MagicCast, event.position, 0.0);
    }
}

// Event for spell casting (would be emitted by magic system)
#[derive(Event)]
pub struct SpellCastEvent {
    pub position: Vec2,
    pub spell_name: String,
}

// Audio bus management (simplified FMOD approach)
pub fn setup_audio_buses(mut commands: Commands) {
    let mut buses = HashMap::new();

    // Master bus
    buses.insert("master".to_string(), AudioBus {
        name: "master".to_string(),
        volume: 1.0,
        effects: vec![],
    });

    // SFX bus
    buses.insert("sfx".to_string(), AudioBus {
        name: "sfx".to_string(),
        volume: 0.8,
        effects: vec![],
    });

    // Ambient bus
    buses.insert("ambient".to_string(), AudioBus {
        name: "ambient".to_string(),
        volume: 0.6,
        effects: vec![AudioEffect::Reverb { room_size: 0.5, damping: 0.3 }],
    });

    commands.insert_resource(AudioBuses(buses));
}

// Toggle sound system
pub fn toggle_sound_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut sound_manager: ResMut<SoundManager>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        sound_manager.sounds_enabled = !sound_manager.sounds_enabled;
        println!("Sound {}: {}", if sound_manager.sounds_enabled { "enabled" } else { "disabled" }, sound_manager.sounds_enabled);
    }
}

// Volume controls
pub fn adjust_volume(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut sound_manager: ResMut<SoundManager>,
) {
    if keyboard_input.just_pressed(KeyCode::Equal) { // +
        sound_manager.master_volume = (sound_manager.master_volume + 0.1).min(1.0);
        println!("Master volume: {:.1}", sound_manager.master_volume);
    }

    if keyboard_input.just_pressed(KeyCode::Minus) { // -
        sound_manager.master_volume = (sound_manager.master_volume - 0.1).max(0.0);
        println!("Master volume: {:.1}", sound_manager.master_volume);
    }
}
