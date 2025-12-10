// Example: Loading Levels
// Based on "Loading Levels" blog post
// https://www.slowrush.dev/news/loading-levels

use bevy::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// Level loading system with serialization and deserialization
// Demonstrates how to save and load levels

#[derive(Clone, Debug, Serialize, Deserialize)]
enum AtomType {
    Empty,
    Sand,
    Water,
    Stone,
    Fire,
    Acid,
}

impl AtomType {
    fn color(&self) -> Color {
        match self {
            AtomType::Empty => Color::rgba(0.0, 0.0, 0.0, 0.0),
            AtomType::Sand => Color::rgb(0.8, 0.7, 0.5),
            AtomType::Water => Color::rgba(0.2, 0.4, 0.8, 0.8),
            AtomType::Stone => Color::rgb(0.4, 0.4, 0.4),
            AtomType::Fire => Color::rgb(1.0, 0.3, 0.0),
            AtomType::Acid => Color::rgb(0.0, 0.8, 0.0),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SerializableAtom {
    x: usize,
    y: usize,
    atom_type: AtomType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlayerSpawn {
    x: f32,
    y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LevelData {
    name: String,
    width: usize,
    height: usize,
    atoms: Vec<SerializableAtom>,
    player_spawn: PlayerSpawn,
    description: String,
    difficulty: u32,
}

impl LevelData {
    fn new(name: String, width: usize, height: usize) -> Self {
        Self {
            name,
            width,
            height,
            atoms: Vec::new(),
            player_spawn: PlayerSpawn { x: 50.0, y: 50.0 },
            description: "A level".to_string(),
            difficulty: 1,
        }
    }

    fn add_atom(&mut self, x: usize, y: usize, atom_type: AtomType) {
        self.atoms.push(SerializableAtom { x, y, atom_type });
    }

    // Serialize to JSON string
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    // Deserialize from JSON string
    fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    // Save to file (in a real implementation)
    fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = self.to_json()?;
        println!("Would save level '{}' to file '{}'", self.name, filename);
        println!("Level data: {}", json);
        Ok(())
    }

    // Load from file (in a real implementation)
    fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Would load level from file '{}'", filename);
        // For demo, return a default level
        Ok(Self::create_demo_level())
    }

    fn create_demo_level() -> Self {
        let mut level = Self::new("Demo Level".to_string(), 100, 80);

        // Create stone floor
        for x in 0..level.width {
            for y in 0..8 {
                level.add_atom(x, y, AtomType::Stone);
            }
        }

        // Create sand pile
        for x in 30..70 {
            for y in 10..30 {
                if rand::random::<f32>() < 0.8 {
                    level.add_atom(x, y, AtomType::Sand);
                }
            }
        }

        // Create water pool
        for x in 20..50 {
            for y in 40..60 {
                if rand::random::<f32>() < 0.9 {
                    level.add_atom(x, y, AtomType::Water);
                }
            }
        }

        level.player_spawn = PlayerSpawn { x: 25.0, y: 70.0 };
        level.description = "A demonstration level with various atoms".to_string();
        level.difficulty = 2;

        level
    }
}

#[derive(Clone, Debug)]
struct LoadedAtom {
    position: Vec2,
    atom_type: AtomType,
}

#[derive(Clone, Debug)]
struct LoadedPlayer {
    position: Vec2,
}

struct LoadedLevel {
    name: String,
    width: usize,
    height: usize,
    atoms: Vec<LoadedAtom>,
    player: LoadedPlayer,
    description: String,
    difficulty: u32,
}

impl LoadedLevel {
    fn from_level_data(data: LevelData) -> Self {
        let atoms = data.atoms.into_iter()
            .map(|atom| LoadedAtom {
                position: Vec2::new(atom.x as f32, atom.y as f32),
                atom_type: atom.atom_type,
            })
            .collect();

        Self {
            name: data.name,
            width: data.width,
            height: data.height,
            atoms,
            player: LoadedPlayer {
                position: Vec2::new(data.player_spawn.x, data.player_spawn.y),
            },
            description: data.description,
            difficulty: data.difficulty,
        }
    }
}

#[derive(Resource)]
struct LevelManager {
    current_level: Option<LoadedLevel>,
    available_levels: HashMap<String, LevelData>,
    level_index: usize,
}

impl Default for LevelManager {
    fn default() -> Self {
        let mut available_levels = HashMap::new();

        // Create some demo levels
        available_levels.insert("tutorial".to_string(), Self::create_tutorial_level());
        available_levels.insert("challenge".to_string(), Self::create_challenge_level());
        available_levels.insert("creative".to_string(), Self::create_creative_level());

        Self {
            current_level: None,
            available_levels,
            level_index: 0,
        }
    }
}

impl LevelManager {
    fn create_tutorial_level() -> LevelData {
        let mut level = LevelData::new("Tutorial".to_string(), 80, 60);
        level.description = "Learn the basics of falling sand physics".to_string();
        level.difficulty = 1;
        level.player_spawn = PlayerSpawn { x: 40.0, y: 50.0 };

        // Simple stone platform
        for x in 20..60 {
            for y in 0..5 {
                level.add_atom(x, y, AtomType::Stone);
            }
        }

        // Small sand pile
        for x in 35..45 {
            for y in 10..20 {
                if rand::random::<f32>() < 0.7 {
                    level.add_atom(x, y, AtomType::Sand);
                }
            }
        }

        level
    }

    fn create_challenge_level() -> LevelData {
        let mut level = LevelData::new("Challenge".to_string(), 120, 90);
        level.description = "Complex level with multiple atom types".to_string();
        level.difficulty = 3;
        level.player_spawn = PlayerSpawn { x: 60.0, y: 80.0 };

        // Complex terrain
        for x in 0..level.width {
            let height = 10 + (x as f32 * 0.1).sin() as usize * 5;
            for y in 0..height {
                level.add_atom(x, y, AtomType::Stone);
            }
        }

        // Multiple sand piles
        for i in 0..3 {
            let center_x = 30 + i * 30;
            for x in (center_x - 10)..(center_x + 10) {
                for y in 20..40 {
                    if rand::random::<f32>() < 0.8 {
                        level.add_atom(x, y, AtomType::Sand);
                    }
                }
            }
        }

        // Acid pits
        for x in 50..70 {
            for y in 15..25 {
                if rand::random::<f32>() < 0.6 {
                    level.add_atom(x, y, AtomType::Acid);
                }
            }
        }

        level
    }

    fn create_creative_level() -> LevelData {
        let mut level = LevelData::new("Creative".to_string(), 100, 75);
        level.description = "Blank canvas for experimentation".to_string();
        level.difficulty = 1;
        level.player_spawn = PlayerSpawn { x: 50.0, y: 60.0 };

        // Just a small platform to stand on
        for x in 45..55 {
            for y in 0..3 {
                level.add_atom(x, y, AtomType::Stone);
            }
        }

        level
    }

    fn load_level(&mut self, level_name: &str) -> Result<(), String> {
        if let Some(level_data) = self.available_levels.get(level_name) {
            self.current_level = Some(LoadedLevel::from_level_data(level_data.clone()));
            println!("Loaded level: {}", level_name);
            Ok(())
        } else {
            Err(format!("Level '{}' not found", level_name))
        }
    }

    fn get_level_names(&self) -> Vec<String> {
        self.available_levels.keys().cloned().collect()
    }

    fn next_level(&mut self) {
        let level_names = self.get_level_names();
        self.level_index = (self.level_index + 1) % level_names.len();
        let next_level_name = &level_names[self.level_index];
        let _ = self.load_level(next_level_name);
    }

    fn previous_level(&mut self) {
        let level_names = self.get_level_names();
        if self.level_index == 0 {
            self.level_index = level_names.len() - 1;
        } else {
            self.level_index -= 1;
        }
        let prev_level_name = &level_names[self.level_index];
        let _ = self.load_level(prev_level_name);
    }
}

#[derive(Component)]
struct LevelAtom;

#[derive(Component)]
struct PlayerMarker;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Loading Levels - Level Management System".to_string(),
                resolution: (1000.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(LevelManager::default())
        .add_systems(Startup, setup_level_loading)
        .add_systems(Update, (
            handle_level_controls,
            render_loaded_level,
            display_level_info,
        ).chain())
        .run();
}

fn setup_level_loading(mut commands: Commands, mut level_manager: ResMut<LevelManager>) {
    commands.spawn(Camera2dBundle::default());

    // Load initial level
    let _ = level_manager.load_level("tutorial");
}

fn handle_level_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut level_manager: ResMut<LevelManager>,
) {
    if keyboard_input.just_pressed(KeyCode::BracketRight) { // ]
        level_manager.next_level();
    }

    if keyboard_input.just_pressed(KeyCode::BracketLeft) { // [
        level_manager.previous_level();
    }

    if keyboard_input.just_pressed(KeyCode::KeyS) {
        // Save current level (demo - would save to file)
        if let Some(level) = &level_manager.current_level {
            let level_data = LevelData {
                name: level.name.clone(),
                width: level.width,
                height: level.height,
                atoms: level.atoms.iter().map(|atom| SerializableAtom {
                    x: atom.position.x as usize,
                    y: atom.position.y as usize,
                    atom_type: atom.atom_type.clone(),
                }).collect(),
                player_spawn: PlayerSpawn {
                    x: level.player.position.x,
                    y: level.player.position.y,
                },
                description: level.description.clone(),
                difficulty: level.difficulty,
            };

            let filename = format!("{}.json", level.name.to_lowercase().replace(" ", "_"));
            let _ = level_data.save_to_file(&filename);
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyL) {
        // Load level from file (demo)
        let _ = LevelData::load_from_file("demo_level.json");
    }
}

fn render_loaded_level(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    mut player_entity: Local<Option<Entity>>,
    level_manager: Res<LevelManager>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }

    if let Some(level) = &level_manager.current_level {
        // Render atoms
        for atom in &level.atoms {
            let entity = commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: atom.atom_type.color(),
                        custom_size: Some(Vec2::new(1.0, 1.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        atom.position.x - level.width as f32 / 2.0,
                        atom.position.y - level.height as f32 / 2.0,
                        0.0,
                    ),
                    ..default()
                },
                LevelAtom,
            )).id();
            atom_entities.push(entity);
        }

        // Render player spawn point
        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.2, 0.8, 0.2),
                    custom_size: Some(Vec2::new(10.0, 15.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    level.player.position.x - level.width as f32 / 2.0,
                    level.player.position.y - level.height as f32 / 2.0,
                    1.0,
                ),
                ..default()
            },
            PlayerMarker,
        )).id();
        *player_entity = Some(entity);
    }
}

fn display_level_info(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    level_manager: Res<LevelManager>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        if let Some(level) = &level_manager.current_level {
            println!("\n=== Level Information ===");
            println!("Name: {}", level.name);
            println!("Size: {}x{}", level.width, level.height);
            println!("Description: {}", level.description);
            println!("Difficulty: {}", level.difficulty);
            println!("Atoms: {}", level.atoms.len());
            println!("Player spawn: ({:.1}, {:.1})", level.player.position.x, level.player.position.y);
        }

        println!("\n=== Available Levels ===");
        for level_name in level_manager.get_level_names() {
            let current_marker = if Some(&level_name) == level_manager.current_level.as_ref().map(|l| &l.name) { " ←" } else { "" };
            println!("- {}{}", level_name, current_marker);
        }

        println!("\nControls:");
        println!("[ ]: Switch levels");
        println!("S: Save current level");
        println!("L: Load level");
        println!("H: Show this info");
        println!("====================\n");
    }
}
