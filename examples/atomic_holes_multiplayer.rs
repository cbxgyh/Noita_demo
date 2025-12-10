// Example: Atomic Holes and Multiplayer
// Based on "Atomic Holes and Multiplayer" blog post
// https://www.slowrush.dev/news/atomic-holes-and-multiplayer

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Atomic holes (terrain with gaps) and basic multiplayer support
// Demonstrates how moving bodies can have holes without glitching

#[derive(Clone, Debug,Eq,PartialEq)]
enum AtomType {
    Empty,
    Stone,
    Sand,
    Water,
}

impl AtomType {
    fn color(&self) -> Color {
        match self {
            AtomType::Empty => Color::srgba(0.0, 0.0, 0.0, 0.0),
            AtomType::Stone => Color::srgb(0.4, 0.4, 0.4),
            AtomType::Sand => Color::srgb(0.8, 0.7, 0.5),
            AtomType::Water => Color::srgba(0.2, 0.4, 0.8, 0.8),
        }
    }

    fn is_solid(&self) -> bool {
        matches!(self, AtomType::Stone | AtomType::Sand)
    }
}

#[derive(Clone, Debug)]
struct TerrainAtom {
    atom_type: AtomType,
    position: Vec2,
    is_hole: bool, // Marks atoms that are part of holes/gaps
}

#[derive(Clone, Debug)]
struct Player {
    id: u32,
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
    color: Color,
    input: PlayerInput,
}

#[derive(Clone, Debug)]
struct PlayerInput {
    move_left: bool,
    move_right: bool,
    jump: bool,
    grounded: bool,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            move_left: false,
            move_right: false,
            jump: false,
            grounded: false,
        }
    }
}

struct MultiplayerWorld {
    width: usize,
    height: usize,
    atoms: Vec<TerrainAtom>,
    players: Vec<Player>,
    local_player_id: u32,
}

impl MultiplayerWorld {
    fn new(width: usize, height: usize) -> Self {
        let mut atoms = Vec::with_capacity(width * height);

        // Create terrain with holes
        for y in 0..height {
            for x in 0..width {
                let mut atom_type = AtomType::Empty;
                let mut is_hole = false;

                // Create solid ground with holes
                if y < 10 {
                    atom_type = AtomType::Stone;
                } else if y < 15 && x > width / 2 - 20 && x < width / 2 + 20 {
                    // Create a hole in the terrain
                    is_hole = true;
                } else if y < 15 {
                    atom_type = AtomType::Stone;
                }

                // Add some floating platforms with holes
                if y > 30 && y < 35 && x > 20 && x < 80 {
                    if !(x > 35 && x < 65 && y > 32) { // Hole in platform
                        atom_type = AtomType::Stone;
                    } else {
                        is_hole = true;
                    }
                }

                atoms.push(TerrainAtom {
                    atom_type,
                    position: Vec2::new(x as f32, y as f32),
                    is_hole,
                });
            }
        }

        let mut players = Vec::new();

        // Add local player
        players.push(Player {
            id: 0,
            position: Vec2::new(50.0, 50.0),
            velocity: Vec2::ZERO,
            size: Vec2::new(16.0, 24.0),
            color: Color::srgb(0.2, 0.8, 0.2),
            input: PlayerInput::default(),
        });

        // Add remote player (simulated)
        players.push(Player {
            id: 1,
            position: Vec2::new(150.0, 50.0),
            velocity: Vec2::ZERO,
            size: Vec2::new(16.0, 24.0),
            color: Color::srgb(0.8, 0.2, 0.2),
            input: PlayerInput::default(),
        });

        Self {
            width,
            height,
            atoms,
            players,
            local_player_id: 0,
        }
    }

    fn update(&mut self, dt: f32) {
        // Update players
        // Get atoms reference first to avoid borrowing conflicts
        let atoms = &self.atoms;
        for player in &mut self.players {
             // self.update_player(player, dt);
            Self::update_player_with_atoms(player, atoms, dt);
        }

        // Simulate network sync (in a real implementation, this would sync with remote players)
        self.simulate_network_sync();
    }

    fn update_player_with_atoms(player: &mut Player, atoms: &[TerrainAtom], dt: f32) {
        // Apply input
        let mut acceleration = Vec2::ZERO;

        if player.input.move_left {
            acceleration.x -= 200.0;
        }
        if player.input.move_right {
            acceleration.x += 200.0;
        }

        // Apply gravity
        acceleration.y -= 300.0;

        // Update velocity
        player.velocity += acceleration * dt;

        // Apply friction
        player.velocity.x *= 0.8;

        // Update position
        let mut new_position = player.position + player.velocity * dt;

        // Collision detection with terrain (including holes)
        player.input.grounded = false;

        // Check collision with atoms
        let player_bounds = Rect::from_center_size(new_position, player.size);

        for atom in atoms {
            if atom.atom_type.is_solid() && !atom.is_hole {
                let atom_bounds = Rect::from_center_size(atom.position, Vec2::new(1.0, 1.0));

                let intersection = player_bounds.intersect(atom_bounds);
                if intersection.width() > 0.0 && intersection.height() > 0.0 {
                    // Resolve collision
                    let overlap = Self::get_overlap(player_bounds, atom_bounds);

                    if overlap.x.abs() < overlap.y.abs() {
                        // Horizontal collision
                        if overlap.x > 0.0 {
                            new_position.x = atom.position.x - atom_bounds.width() / 2.0 - player.size.x / 2.0;
                        } else {
                            new_position.x = atom.position.x + atom_bounds.width() / 2.0 + player.size.x / 2.0;
                        }
                        player.velocity.x = 0.0;
                    } else {
                        // Vertical collision
                        if overlap.y > 0.0 {
                            new_position.y = atom.position.y - atom_bounds.height() / 2.0 - player.size.y / 2.0;
                            player.input.grounded = true;
                        } else {
                            new_position.y = atom.position.y + atom_bounds.height() / 2.0 + player.size.y / 2.0;
                        }
                        player.velocity.y = 0.0;
                    }
                }
            }
        }

        // Check if player falls into hole
        let mut in_hole = false;
        for atom in atoms {
            if atom.is_hole {
                let atom_bounds = Rect::from_center_size(atom.position, Vec2::new(1.0, 1.0));
                let intersection = player_bounds.intersect(atom_bounds);
                if intersection.width() > 0.0 && intersection.height() > 0.0 {
                    in_hole = true;
                    break;
                }
            }
        }

        if in_hole && player.input.grounded {
            // Player fell into hole - respawn
            player.position = Vec2::new(player.position.x, player.position.y + 50.0);
            player.velocity = Vec2::ZERO;
        } else {
            player.position = new_position;
        }

        // Handle jumping
        if player.input.jump && player.input.grounded {
            player.velocity.y = 150.0;
            player.input.jump = false;
        }
    }

    fn get_overlap(rect1: Rect, rect2: Rect) -> Vec2 {
        let center1 = rect1.center();
        let center2 = rect2.center();

        let overlap_x = (rect1.width() + rect2.width()) / 2.0 - (center1.x - center2.x).abs();
        let overlap_y = (rect1.height() + rect2.height()) / 2.0 - (center1.y - center2.y).abs();

        Vec2::new(
            if center1.x > center2.x { overlap_x } else { -overlap_x },
            if center1.y > center2.y { overlap_y } else { -overlap_y },
        )
    }

    fn simulate_network_sync(&mut self) {
        // Simulate receiving input from remote player
        // In a real implementation, this would come from network packets

        static mut REMOTE_MOVE_COUNTER: f32 = 0.0;
        unsafe {
            REMOTE_MOVE_COUNTER += 0.016; // Assume 60fps

            if REMOTE_MOVE_COUNTER > 2.0 { // Every 2 seconds
                REMOTE_MOVE_COUNTER = 0.0;

                // Make remote player move randomly
                if let Some(remote_player) = self.players.get_mut(1) {
                    remote_player.input.move_left = rand::random::<bool>();
                    remote_player.input.move_right = !remote_player.input.move_left;
                    remote_player.input.jump = rand::random::<f32>() < 0.3;
                }
            }
        }
    }

    fn set_local_player_input(&mut self, input: PlayerInput) {
        if let Some(player) = self.players.get_mut(self.local_player_id as usize) {
            player.input = input;
        }
    }
}

#[derive(Resource)]
struct MultiplayerWorldResource(MultiplayerWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Atomic Holes and Multiplayer - Terrain Gaps & Multiplayer".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(MultiplayerWorldResource(MultiplayerWorld::new(200, 100)))
        .add_systems(Startup, setup_multiplayer_demo)
        .add_systems(Update, (
            update_multiplayer_world,
            render_multiplayer_world,
            handle_multiplayer_input,
            demonstrate_holes_multiplayer,
        ).chain())
        .run();
}

fn setup_multiplayer_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_multiplayer_world(mut world: ResMut<MultiplayerWorldResource>, time: Res<Time>) {
    let dt = time.delta_seconds().min(1.0 / 30.0); // Cap dt for stability
    world.0.update(dt);
}

fn render_multiplayer_world(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    mut player_entities: Local<Vec<Entity>>,
    world: Res<MultiplayerWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in player_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render atoms
    for atom in &world.0.atoms {
        if atom.atom_type != AtomType::Empty {
            let alpha = if atom.is_hole { 0.3 } else { 1.0 };
            let base_color = atom.atom_type.color();
            let [r, g, b, _] = base_color.to_srgba().to_f32_array();
            let color = Color::srgba(r, g, b, alpha);

            let entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    atom.position.x - world.0.width as f32 / 2.0,
                    atom.position.y - world.0.height as f32 / 2.0,
                    0.0,
                ),
                ..default()
            }).id();
            atom_entities.push(entity);
        }
    }

    // Render players
    for player in &world.0.players {
        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: player.color,
                    custom_size: Some(player.size),
                    ..default()
                },
                transform: Transform::from_xyz(
                    player.position.x - world.0.width as f32 / 2.0,
                    player.position.y - world.0.height as f32 / 2.0,
                    1.0,
                ),
                ..default()
            }
        )).id();
        player_entities.push(entity);
    }
}

fn handle_multiplayer_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<MultiplayerWorldResource>,
) {
    let mut input = PlayerInput::default();

    // Local player input
    input.move_left = keyboard_input.pressed(KeyCode::KeyA);
    input.move_right = keyboard_input.pressed(KeyCode::KeyD);
    input.jump = keyboard_input.just_pressed(KeyCode::Space);

    world.0.set_local_player_input(input);
}

fn demonstrate_holes_multiplayer(keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Atomic Holes & Multiplayer Demo ===");
        println!("Features:");
        println!("- Terrain with holes (transparent areas)");
        println!("- Players can fall through holes");
        println!("- Two players: Local (Green) and Remote (Red)");
        println!("- Remote player moves automatically (simulated)");
        println!("- Collision detection works around holes");
        println!("\nControls:");
        println!("A/D: Move left/right");
        println!("Space: Jump");
        println!("H: Show this help");
        println!("=====================================\n");
    }
}
