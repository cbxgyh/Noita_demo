use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use crate::atoms::{AtomWorld, Atom, AtomType, AtomWorldResource};

// Procedural level generation using noise functions
// Based on "Legit Levels" blog post

pub struct LevelGenerator {
    seed: u32,
    perlin: Perlin,
}

impl LevelGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            perlin: Perlin::new(seed),
        }
    }

    pub fn generate_level(&self, width: usize, height: usize, level_type: LevelType) -> AtomWorld {
        let mut world = AtomWorld::new(width, height);

        match level_type {
            LevelType::Cave => self.generate_cave_level(&mut world),
            LevelType::Island => self.generate_island_level(&mut world),
            LevelType::Mountain => self.generate_mountain_level(&mut world),
            LevelType::Volcano => self.generate_volcano_level(&mut world),
            LevelType::Laboratory => self.generate_laboratory_level(&mut world),
        }

        world
    }

    fn generate_cave_level(&self, world: &mut AtomWorld) {
        // Generate cave system using 3D noise for natural cave shapes
        for y in 0..world.height {
            for x in 0..world.width {
                let cave_noise = self.perlin.get([
                    x as f64 * 0.05,
                    y as f64 * 0.05,
                    0.0
                ]);

                let tunnel_noise = self.perlin.get([
                    x as f64 * 0.02,
                    y as f64 * 0.02,
                    1.0
                ]);

                // Create cave walls and floors
                if cave_noise > 0.1 || tunnel_noise > 0.3 {
                    let atom_type = if cave_noise > 0.6 {
                        AtomType::Stone
                    } else {
                        AtomType::Sand
                    };

                    world.set_atom(x as i32, y as i32, Atom {
                        atom_type,
                        velocity: Vec2::ZERO,
                        mass: atom_type.mass(),
                        lifetime: None,
                        temperature: 20.0,
                    });
                }

                // Add ore deposits
                if cave_noise > 0.7 && rand::random::<f32>() < 0.1 {
                    // Special ore (could be different atom types)
                }
            }
        }

        // Add water lakes in caves
        self.add_water_features(world, 0.1);
    }

    fn generate_island_level(&self, world: &mut AtomWorld) {
        // Generate floating island using radial falloff
        let center_x = world.width as f32 / 2.0;
        let center_y = world.height as f32 / 2.0;

        for y in 0..world.height {
            for x in 0..world.width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                let max_distance = center_x.min(center_y);

                let height_factor = 1.0 - (distance / max_distance);
                let noise = self.perlin.get([
                    x as f64 * 0.1,
                    y as f64 * 0.1,
                    0.0
                ]) as f32;

                let threshold = 0.3 + height_factor * 0.4 + noise * 0.2;

                if rand::random::<f32>() < threshold {
                    let atom_type = if height_factor > 0.7 {
                        AtomType::Stone
                    } else {
                        AtomType::Sand
                    };

                    world.set_atom(x as i32, y as i32, Atom {
                        atom_type,
                        velocity: Vec2::ZERO,
                        mass: atom_type.mass(),
                        lifetime: None,
                        temperature: 20.0,
                    });
                }
            }
        }

        // Add water around the island
        self.add_water_features(world, 0.8);
    }

    fn generate_mountain_level(&self, world: &mut AtomWorld) {
        // Generate mountain ranges
        for y in 0..world.height {
            for x in 0..world.width {
                let ridge_noise = self.perlin.get([
                    x as f64 * 0.03,
                    0.0,
                    0.0
                ]);

                let detail_noise = self.perlin.get([
                    x as f64 * 0.1,
                    y as f64 * 0.1,
                    1.0
                ]);

                let height = (ridge_noise as f32 + 1.0) / 2.0;
                let threshold = 0.4 + height * 0.4 + detail_noise as f32 * 0.1;

                if y as f32 > world.height as f32 * threshold {
                    let atom_type = if height > 0.7 {
                        AtomType::Stone
                    } else {
                        AtomType::Sand
                    };

                    world.set_atom(x as i32, y as i32, Atom {
                        atom_type,
                        velocity: Vec2::ZERO,
                        mass: atom_type.mass(),
                        lifetime: None,
                        temperature: 20.0,
                    });
                }
            }
        }
    }

    fn generate_volcano_level(&self, world: &mut AtomWorld) {
        // Generate volcano with lava
        let center_x = world.width / 2;
        let center_y = world.height / 2;

        // Create volcanic cone
        for y in 0..world.height {
            for x in 0..world.width {
                let dx = (x as i32 - center_x as i32) as f32;
                let dy = (y as i32 - center_y as i32) as f32;
                let distance = (dx * dx + dy * dy).sqrt();

                let cone_height = 50.0 - distance * 0.5;
                let noise = self.perlin.get([
                    x as f64 * 0.08,
                    y as f64 * 0.08,
                    0.0
                ]) as f32 * 10.0;

                if y as f32 > world.height as f32 - cone_height + noise {
                    world.set_atom(x as i32, y as i32, Atom {
                        atom_type: AtomType::Stone,
                        velocity: Vec2::ZERO,
                        mass: AtomType::Stone.mass(),
                        lifetime: None,
                        temperature: 20.0,
                    });
                }
                // Add lava inside volcano
                else if distance < 20.0 && y as f32 > world.height as f32 - cone_height + 20.0 {
                    world.set_atom(x as i32, y as i32, Atom {
                        atom_type: AtomType::Fire,
                        velocity: Vec2::ZERO,
                        mass: AtomType::Fire.mass(),
                        lifetime: Some(30.0),
                        temperature: 1000.0,
                    });
                }
            }
        }
    }

    fn generate_laboratory_level(&self, world: &mut AtomWorld) {
        // Generate laboratory with containment areas
        let room_width = 40;
        let room_height = 30;

        for room_y in 0..(world.height / room_height) {
            for room_x in 0..(world.width / room_width) {
                let start_x = room_x * room_width;
                let start_y = room_y * room_height;

                // Create room walls
                for x in start_x..start_x + room_width {
                    for y in start_y..start_y + room_height {
                        let is_wall = x == start_x || x == start_x + room_width - 1 ||
                                     y == start_y || y == start_y + room_height - 1;

                        if is_wall {
                            world.set_atom(x as i32, y as i32, Atom {
                                atom_type: AtomType::Stone,
                                velocity: Vec2::ZERO,
                                mass: AtomType::Stone.mass(),
                                lifetime: None,
                                temperature: 20.0,
                            });
                        }
                    }
                }

                // Add experimental materials based on room
                let experiment_type = (room_x + room_y) % 4;
                match experiment_type {
                    0 => self.add_acid_experiment(world, start_x + 5, start_y + 5, 10),
                    1 => self.add_fire_experiment(world, start_x + 5, start_y + 5, 10),
                    2 => self.add_water_experiment(world, start_x + 5, start_y + 5, 10),
                    3 => self.add_mixed_experiment(world, start_x + 5, start_y + 5, 10),
                    _ => {}
                }
            }
        }
    }

    fn add_water_features(&self, world: &mut AtomWorld, probability: f32) {
        for y in 0..world.height {
            for x in 0..world.width {
                if world.get_atom(x as i32, y as i32).map_or(true, |a| a.atom_type == AtomType::Empty) {
                    if rand::random::<f32>() < probability {
                        world.set_atom(x as i32, y as i32, Atom {
                            atom_type: AtomType::Water,
                            velocity: Vec2::ZERO,
                            mass: AtomType::Water.mass(),
                            lifetime: None,
                            temperature: 20.0,
                        });
                    }
                }
            }
        }
    }

    fn add_acid_experiment(&self, world: &mut AtomWorld, start_x: usize, start_y: usize, size: usize) {
        for x in start_x..start_x + size {
            for y in start_y..start_y + size {
                if rand::random::<f32>() < 0.6 {
                    world.set_atom(x as i32, y as i32, Atom {
                        atom_type: AtomType::Acid,
                        velocity: Vec2::ZERO,
                        mass: AtomType::Acid.mass(),
                        lifetime: None,
                        temperature: 20.0,
                    });
                }
            }
        }
    }

    fn add_fire_experiment(&self, world: &mut AtomWorld, start_x: usize, start_y: usize, size: usize) {
        for x in start_x..start_x + size {
            for y in start_y..start_y + size {
                if rand::random::<f32>() < 0.4 {
                    world.set_atom(x as i32, y as i32, Atom {
                        atom_type: AtomType::Fire,
                        velocity: Vec2::ZERO,
                        mass: AtomType::Fire.mass(),
                        lifetime: Some(10.0),
                        temperature: 800.0,
                    });
                }
            }
        }
    }

    fn add_water_experiment(&self, world: &mut AtomWorld, start_x: usize, start_y: usize, size: usize) {
        for x in start_x..start_x + size {
            for y in start_y..start_y + size {
                if rand::random::<f32>() < 0.7 {
                    world.set_atom(x as i32, y as i32, Atom {
                        atom_type: AtomType::Water,
                        velocity: Vec2::ZERO,
                        mass: AtomType::Water.mass(),
                        lifetime: None,
                        temperature: 20.0,
                    });
                }
            }
        }
    }

    fn add_mixed_experiment(&self, world: &mut AtomWorld, start_x: usize, start_y: usize, size: usize) {
        // Mix of different atoms for experimentation
        for x in start_x..start_x + size {
            for y in start_y..start_y + size {
                let atom_type = match rand::random::<f32>() {
                    r if r < 0.25 => AtomType::Water,
                    r if r < 0.5 => AtomType::Acid,
                    r if r < 0.75 => AtomType::Fire,
                    _ => AtomType::Sand,
                };

                world.set_atom(x as i32, y as i32, Atom {
                    atom_type,
                    velocity: Vec2::ZERO,
                    mass: atom_type.mass(),
                    lifetime: if atom_type == AtomType::Fire { Some(8.0) } else { None },
                    temperature: if atom_type == AtomType::Fire { 700.0 } else { 20.0 },
                });
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LevelType {
    Cave,
    Island,
    Mountain,
    Volcano,
    Laboratory,
}

// Level manager for switching between levels
#[derive(Resource)]
pub struct LevelManager {
    pub current_level: usize,
    pub generator: LevelGenerator,
    pub level_types: Vec<LevelType>,
}

impl Default for LevelManager {
    fn default() -> Self {
        Self {
            current_level: 0,
            generator: LevelGenerator::new(12345), // Fixed seed for now
            level_types: vec![
                LevelType::Cave,
                LevelType::Island,
                LevelType::Mountain,
                LevelType::Volcano,
                LevelType::Laboratory,
            ],
        }
    }
}

impl LevelManager {
    pub fn next_level(&mut self) -> LevelType {
        self.current_level = (self.current_level + 1) % self.level_types.len();
        self.level_types[self.current_level]
    }

    pub fn previous_level(&mut self) -> LevelType {
        if self.current_level == 0 {
            self.current_level = self.level_types.len() - 1;
        } else {
            self.current_level -= 1;
        }
        self.level_types[self.current_level]
    }

    pub fn get_current_level_type(&self) -> LevelType {
        self.level_types[self.current_level]
    }

    pub fn generate_current_level(&self, width: usize, height: usize) -> AtomWorld {
        self.generator.generate_level(width, height, self.get_current_level_type())
    }
}

// Level loading system
pub fn load_level(
    mut commands: Commands,
    mut world: ResMut<AtomWorldResource>,
    level_manager: Res<LevelManager>,
) {
    *world = AtomWorldResource(level_manager.generate_current_level(200, 150));
}

// Level transition system
pub fn level_transition_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut level_manager: ResMut<LevelManager>,
    mut world: ResMut<AtomWorldResource>,
) {
    if keyboard_input.just_pressed(KeyCode::BracketRight) { // ] key
        let level_type = level_manager.next_level();
        *world = AtomWorldResource(level_manager.generator.generate_level(200, 150, level_type));
        println!("Loaded level: {:?}", level_type);
    }

    if keyboard_input.just_pressed(KeyCode::BracketLeft) { // [ key
        let level_type = level_manager.previous_level();
        *world = AtomWorldResource(level_manager.generator.generate_level(200, 150, level_type));
        println!("Loaded level: {:?}", level_type);
    }
}
