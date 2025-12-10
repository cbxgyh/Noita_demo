// Example: Bridging Physics Worlds
// Based on "Bridging Physics Worlds" blog post
// https://www.slowrush.dev/news/bridging-physics-worlds

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Simplified atom system for physics bridge demo
#[derive(Clone, Copy, Debug, PartialEq)]
enum AtomType {
    Empty,
    Sand,
    Stone,
}

impl AtomType {
    fn color(&self) -> Color {
        match self {
            AtomType::Empty => Color::rgba(0.0, 0.0, 0.0, 0.0),
            AtomType::Sand => Color::rgb(0.8, 0.7, 0.5),
            AtomType::Stone => Color::rgb(0.4, 0.4, 0.4),
        }
    }

    fn is_solid(&self) -> bool {
        matches!(self, AtomType::Sand | AtomType::Stone)
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

    fn is_empty(&self, x: i32, y: i32) -> bool {
        self.get(x, y).map_or(true, |a| a.atom_type == AtomType::Empty)
    }
}

#[derive(Resource)]
struct WorldResource(AtomWorld);

#[derive(Component)]
struct RigidBodyObject;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Physics Bridge - Bridging Physics Worlds".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -100.0),
            ..default()
        })
        .insert_resource(WorldResource(AtomWorld::new(100, 75)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_atom_physics,
            create_terrain_colliders,
            atoms_interact_with_rigid_bodies,
            render_atoms,
        ).chain())
        .run();
}

fn setup(mut commands: Commands, mut world: ResMut<WorldResource>) {
    commands.spawn(Camera2dBundle::default());

    // Create terrain from atoms
    create_atom_terrain(&mut world.0);

    // Spawn a box that should interact with the terrain
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 200.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(10.0, 10.0),
        Velocity::zero(),
        RigidBodyObject,
    ));

    // Spawn a ball
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 1.0, 0.0),
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
            transform: Transform::from_xyz(50.0, 250.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(8.0),
        Velocity::zero(),
        RigidBodyObject,
    ));
}

fn create_atom_terrain(world: &mut AtomWorld) {
    // Create a hill of sand
    for x in 0..world.width {
        let height = 20 + (x as f32 * 0.1).sin() as usize * 10;
        for y in 0..height {
            let atom_type = if y < height - 5 {
                AtomType::Stone
            } else {
                AtomType::Sand
            };
            world.set(x as i32, y as i32, Atom { atom_type });
        }
    }

    // Create a platform
    for x in 60..80 {
        for y in 40..45 {
            world.set(x as i32, y as i32, Atom { atom_type: AtomType::Stone });
        }
    }
}

fn update_atom_physics(mut world: ResMut<WorldResource>) {
    // Simple sand physics
    for y in (0..world.0.height).rev() {
        for x in 0..world.0.width {
            update_sand_atom(&mut world.0, x as i32, y as i32);
        }
    }
}

fn update_sand_atom(world: &mut AtomWorld, x: i32, y: i32) {
    let atom = match world.get(x, y) {
        Some(atom) if atom.atom_type == AtomType::Sand => atom.clone(),
        _ => return,
    };

    // Sand falls down if empty below
    if world.is_empty(x, y + 1) {
        world.set(x, y, Atom { atom_type: AtomType::Empty });
        world.set(x, y + 1, atom);
    }
    // Or diagonally
    else if world.is_empty(x - 1, y + 1) {
        world.set(x, y, Atom { atom_type: AtomType::Empty });
        world.set(x - 1, y + 1, atom);
    }
    else if world.is_empty(x + 1, y + 1) {
        world.set(x, y, Atom { atom_type: AtomType::Empty });
        world.set(x + 1, y + 1, atom);
    }
}

fn create_terrain_colliders(
    mut commands: Commands,
    world: Res<WorldResource>,
    mut existing_colliders: Local<Vec<Entity>>,
) {
    // Remove old colliders
    for entity in existing_colliders.drain(..) {
        commands.entity(entity).despawn();
    }

    // Find solid regions and create colliders
    let solid_regions = find_solid_regions(&world.0);

    for region in solid_regions {
        if region.len() >= 3 {
            // Create a simplified bounding box collider for each region
            let (min_x, max_x, min_y, max_y) = find_bounds(&region);

            let width = max_x - min_x;
            let height = max_y - min_y;
            let center_x = min_x + width / 2.0;
            let center_y = min_y + height / 2.0;

            let entity = commands.spawn((
                RigidBody::Fixed,
                Collider::cuboid(width / 2.0, height / 2.0),
                Transform::from_xyz(center_x, center_y, 0.0),
            )).id();

            existing_colliders.push(entity);
        }
    }
}

fn find_solid_regions(world: &AtomWorld) -> Vec<Vec<Vec2>> {
    let mut visited = vec![false; world.width * world.height];
    let mut regions = Vec::new();

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            if visited[idx] { continue; }

            if let Some(atom) = world.get(x as i32, y as i32) {
                if atom.atom_type.is_solid() {
                    let mut region = Vec::new();
                    flood_fill(world, x, y, &mut visited, &mut region);
                    if region.len() >= 3 {
                        regions.push(region);
                    }
                }
            }
        }
    }

    regions
}

fn flood_fill(
    world: &AtomWorld,
    start_x: usize,
    start_y: usize,
    visited: &mut Vec<bool>,
    region: &mut Vec<Vec2>,
) {
    let mut stack = vec![(start_x, start_y)];

    while let Some((x, y)) = stack.pop() {
        let idx = y * world.width + x;
        if visited[idx] { continue; }

        visited[idx] = true;

        if let Some(atom) = world.get(x as i32, y as i32) {
            if atom.atom_type.is_solid() {
                region.push(Vec2::new(x as f32, y as f32));

                // Check neighbors
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < world.width as i32 && ny >= 0 && ny < world.height as i32 {
                            let nidx = ny as usize * world.width + nx as usize;
                            if !visited[nidx] {
                                stack.push((nx as usize, ny as usize));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn find_bounds(points: &[Vec2]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for point in points {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }

    (min_x, max_x, min_y, max_y)
}

fn atoms_interact_with_rigid_bodies(
    mut rigid_bodies: Query<(&Transform, &Velocity), With<RigidBodyObject>>,
    mut world: ResMut<WorldResource>,
) {
    for (transform, velocity) in rigid_bodies.iter() {
        let pos = transform.translation.truncate();
        let vel = velocity.linvel;

        // If moving fast enough, displace atoms
        if vel.length_squared() > 10.0 {
            displace_atoms_around_point(&mut world.0, pos, vel * 0.1);
        }
    }
}

fn displace_atoms_around_point(world: &mut AtomWorld, pos: Vec2, force: Vec2) {
    let radius = 3.0;

    for dx in -(radius as i32)..=(radius as i32) {
        for dy in -(radius as i32)..=(radius as i32) {
            let dist = Vec2::new(dx as f32, dy as f32).length();
            if dist <= radius {
                let x = (pos.x + dx as f32) as i32;
                let y = (pos.y + dy as f32) as i32;

                if let Some(atom) = world.get(x, y) {
                    if atom.atom_type.is_solid() && rand::random::<f32>() < 0.3 {
                        // Move atom in direction of force
                        let new_x = x + (force.x * 0.5) as i32;
                        let new_y = y + (force.y * 0.5) as i32;

                        if world.is_empty(new_x, new_y) {
                            world.set(x, y, Atom { atom_type: AtomType::Empty });
                            world.set(new_x, new_y, atom.clone());
                        }
                    }
                }
            }
        }
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
            if let Some(atom) = world.get(x as i32, y as i32) {
                if atom.atom_type != AtomType::Empty {
                    let entity = commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: atom.atom_type.color(),
                            custom_size: Some(Vec2::new(1.0, 1.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(
                            x as f32 - (world.0.width as f32 / 2.0),
                            y as f32 - (world.0.height as f32 / 2.0),
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
