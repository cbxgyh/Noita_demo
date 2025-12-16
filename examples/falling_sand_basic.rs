// Example: Basic Falling Sand Simulation
// Based on "The Game So Far" blog post
// https://www.slowrush.dev/news/the-game-so-far

use bevy::prelude::*;
use rand::Rng;

// Simplified atom world for demonstration
#[derive(Clone, Copy, Debug,Eq, PartialEq)]
enum AtomType {
    Empty,
    Sand,
    Water,
}

impl AtomType {
    fn color(&self) -> Color {
        match self {
            AtomType::Empty => Color::rgba(0.0, 0.0, 0.0, 0.0),
            AtomType::Sand => Color::rgb(0.8, 0.7, 0.5),
            AtomType::Water => Color::rgba(0.2, 0.4, 0.8, 0.8),
        }
    }
}

#[derive(Clone)]
struct Atom {
    atom_type: AtomType,
}

struct AtomWorld {
    width: usize,
    height: usize,
    atoms: Vec<Atom>,
}

impl AtomWorld {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            atoms: vec![Atom { atom_type: AtomType::Empty }; width * height],
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<&Atom> {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            Some(&self.atoms[(y as usize) * self.width + (x as usize)])
        } else {
            None
        }
    }

    fn set(&mut self, x: i32, y: i32, atom: Atom) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.atoms[(y as usize) * self.width + (x as usize)] = atom;
        }
    }

    fn swap(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        if let (Some(atom1), Some(atom2)) = (self.get(x1, y1).cloned(), self.get(x2, y2).cloned()) {
            self.set(x1, y1, atom2);
            self.set(x2, y2, atom1);
        }
    }
}

#[derive(Resource)]
struct WorldResource(AtomWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Basic Falling Sand - The Game So Far".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(WorldResource(AtomWorld::new(200, 150)))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_physics, render_atoms))
        .run();
}

fn setup(mut commands: Commands, mut world: ResMut<WorldResource>) {
    commands.spawn(Camera2dBundle::default());

    // Create initial sand pile as shown in the blog
    for x in 80..120 {
        for y in 50..80 {
            if rand::random::<f32>() < 0.7 {
                world.0.set(x, y, Atom { atom_type: AtomType::Sand });
            }
        }
    }

    // Add some water
    for x in 140..160 {
        for y in 60..75 {
            if rand::random::<f32>() < 0.8 {
                world.0.set(x, y, Atom { atom_type: AtomType::Water });
            }
        }
    }
}

fn update_physics(mut world: ResMut<WorldResource>) {
    // Update from bottom to top for gravity
    for y in (0..world.0.height).rev() {
        for x in 0..world.0.width {
            update_atom(&mut world.0, x as i32, y as i32);
        }
    }
}

fn update_atom(world: &mut AtomWorld, x: i32, y: i32) {
    let atom = match world.get(x, y) {
        Some(atom) => atom.clone(),
        None => return,
    };

    match atom.atom_type {
        AtomType::Sand => {
            // Sand falls down
            if matches!(world.get(x, y + 1), Some(a) if a.atom_type == AtomType::Empty) {
                world.swap(x, y, x, y + 1);
            } else if matches!(world.get(x - 1, y + 1), Some(a) if a.atom_type == AtomType::Empty) {
                world.swap(x, y, x - 1, y + 1);
            } else if matches!(world.get(x + 1, y + 1), Some(a) if a.atom_type == AtomType::Empty) {
                world.swap(x, y, x + 1, y + 1);
            }
        }
        AtomType::Water => {
            // Water falls or flows sideways
            if matches!(world.get(x, y + 1), Some(a) if a.atom_type == AtomType::Empty) {
                world.swap(x, y, x, y + 1);
            } else {
                let mut rng = rand::thread_rng();
                let dir = if rng.gen_bool(0.5) { -1 } else { 1 };

                if matches!(world.get(x + dir, y), Some(a) if a.atom_type == AtomType::Empty) {
                    world.swap(x, y, x + dir, y);
                } else if matches!(world.get(x - dir, y), Some(a) if a.atom_type == AtomType::Empty) {
                    world.swap(x, y, x - dir, y);
                }
            }
        }
        AtomType::Empty => {}
    }
}

fn render_atoms(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    world: Res<WorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render current atoms
    for y in 0..world.0.height {
        for x in 0..world.0.width {
            if let Some(atom) = world.0.get(x as i32, y as i32) {
                if !matches!(atom.atom_type, AtomType::Empty) {
                    let entity = commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: atom.atom_type.color(),
                            custom_size: Some(Vec2::new(1.0, 1.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            x as f32 - (world.0.width as f32 / 2.0),
                            (world.0.height as f32 / 2.0) - y as f32,
                            0.0,
                        ),
                        ..default()
                    }).id();
                    atom_entities.push(entity);
                }
            }
        }
    }
}
