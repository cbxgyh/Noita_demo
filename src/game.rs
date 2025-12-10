use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::atoms::{AtomWorldResource, AtomType};
use crate::rendering;
use crate::physics;
use crate::magic;
use crate::level_generation;
use crate::level_editor;
use crate::sound;
use crate::touchscreen;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AtomWorldResource(crate::atoms::AtomWorld::new(200, 150)))
            .insert_resource(level_generation::LevelManager::default())
            .insert_resource(level_editor::LevelEditor::default())
            .insert_resource(level_editor::EditorHistory::default())
            .insert_resource(sound::SoundManager::default())
            .insert_resource(touchscreen::TouchControls::default())
            .insert_resource(touchscreen::TouchGestureRecognizer::default())
            .add_systems(Startup, (setup_game, level_editor::setup_level_editor, sound::setup_audio_buses, touchscreen::setup_touch_controls))
            .add_event::<sound::SpellCastEvent>()
            .add_systems(Update, (
            crate::atoms::update_atoms,
            crate::atoms::process_reactions,
            physics::atoms_push_rigid_bodies,
            physics::rigid_bodies_displace_atoms,
            player_input,
            update_player,
            rendering::render_atoms,
            rendering::camera_follow_player,
            brush_tool,
            spawn_demo_atoms,
            magic::update_magic_users,
            magic::cast_spell,
            magic::update_spell_instances,
            level_generation::level_transition_system,

            ))
            .add_systems(Update, (
                level_editor::toggle_level_editor,
              level_editor::update_editor_cursor,
              level_editor::editor_input,
              level_editor::editor_undo_redo,
              sound::monitor_atomic_reactions,
              sound::toggle_sound_system,
              sound::adjust_volume,
              touchscreen::process_touch_input,
              touchscreen::touch_to_game_input,
              touchscreen::toggle_touchscreen,
              touchscreen::render_touch_controls)
            )
            .add_systems(FixedUpdate, physics::create_terrain_colliders);
    }
}

// Player component
#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub jump_force: f32,
    pub is_grounded: bool,
    pub coyote_time: f32,
    pub coyote_timer: f32,
    pub jump_buffer_time: f32,
    pub jump_buffer_timer: f32,
}

// Brush tool for painting atoms
#[derive(Resource)]
pub struct BrushTool {
    pub atom_type: AtomType,
    pub size: i32,
    pub is_active: bool,
}

impl Default for BrushTool {
    fn default() -> Self {
        Self {
            atom_type: AtomType::Sand,
            size: 3,
            is_active: false,
        }
    }
}

fn setup_game(
    mut commands: Commands,
    mut world: ResMut<AtomWorldResource>,
) {
    // Setup pixel-perfect camera
    commands.spawn((
        Camera2dBundle::default(),
        rendering::PixelCamera {
            pixels_per_unit: 32.0,
        },
    ));

    // Create some initial terrain
    create_demo_terrain(&mut world.0);

    // Spawn player
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1.0, 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 10.0, 1.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 1.0),
        Velocity::zero(),
        LockedAxes::ROTATION_LOCKED,
        Player {
            speed: 5.0,
            jump_force: 10.0,
            is_grounded: false,
            coyote_time: 0.1,
            coyote_timer: 0.0,
            jump_buffer_time: 0.1,
            jump_buffer_timer: 0.0,
        },
        magic::MagicUser::default(),
    ));

    // Initialize brush tool
    commands.insert_resource(BrushTool::default());

    // Setup physics - gravity is configured via the RapierPhysicsPlugin in main.rs
    // The default gravity should work, but if we need to modify it, we can do so here
    // Note: In Bevy Rapier 0.27, gravity is typically set during plugin initialization
}

fn create_demo_terrain(world: &mut crate::atoms::AtomWorld) {
    // Create a simple terrain floor
    for x in 0..world.width {
        for y in 0..10 {
            world.set_atom(x as i32, y as i32, crate::atoms::Atom {
                atom_type: AtomType::Stone,
                velocity: Vec2::ZERO,
                mass: AtomType::Stone.mass(),
                lifetime: None,
                temperature: 20.0,
            });
        }
    }

    // Add some sand piles
    for x in 50..80 {
        for y in 10..25 {
            if rand::random::<f32>() < 0.7 {
                world.set_atom(x as i32, y as i32, crate::atoms::Atom {
                    atom_type: AtomType::Sand,
                    velocity: Vec2::ZERO,
                    mass: AtomType::Sand.mass(),
                    lifetime: None,
                    temperature: 20.0,
                });
            }
        }
    }

    // Add water
    for x in 120..140 {
        for y in 10..20 {
            if rand::random::<f32>() < 0.8 {
                world.set_atom(x as i32, y as i32, crate::atoms::Atom {
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

fn player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut brush: ResMut<BrushTool>,
) {
    // Brush tool controls
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        brush.atom_type = AtomType::Sand;
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        brush.atom_type = AtomType::Water;
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        brush.atom_type = AtomType::Stone;
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        brush.atom_type = AtomType::Acid;
    }
    if keyboard_input.just_pressed(KeyCode::Digit5) {
        brush.atom_type = AtomType::Fire;
    }

    brush.is_active = keyboard_input.pressed(KeyCode::Space);
}

fn update_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Velocity, &Transform)>,
    rapier_context: Res<RapierContext>,
) {
    for (mut player, mut velocity, transform) in query.iter_mut() {
        // Movement
        let mut movement = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            movement.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            movement.x += 1.0;
        }

        velocity.linvel.x = movement.x * player.speed;

        // Ground check
        let shape = Collider::cuboid(0.4, 0.1);
        let shape_pos = transform.translation.truncate() + Vec2::new(0.0, -1.1);
        let filter = QueryFilter::default();

        player.is_grounded = rapier_context.intersection_with_shape(
            shape_pos,
            0.0,
            &shape,
            filter,
        ).is_some();

        // Coyote time and jump buffering
        let dt = time.delta_seconds();

        if player.is_grounded {
            player.coyote_timer = player.coyote_time;
        } else {
            player.coyote_timer = (player.coyote_timer - dt).max(0.0);
        }

        if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
            player.jump_buffer_timer = player.jump_buffer_time;
        } else {
            player.jump_buffer_timer = (player.jump_buffer_timer - dt).max(0.0);
        }

        if player.jump_buffer_timer > 0.0 && player.coyote_timer > 0.0 {
            velocity.linvel.y = player.jump_force;
            player.jump_buffer_timer = 0.0;
            player.coyote_timer = 0.0;
        }
    }
}

fn brush_tool(
    brush: Res<BrushTool>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<AtomWorldResource>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if !brush.is_active && !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera_query.single();
    let window = windows.single();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let atom_x = (world_pos.origin.x + (world.0.width as f32 / 2.0)).round() as i32;
            let atom_y = (world_pos.origin.y + (world.0.height as f32 / 2.0)).round() as i32;

            // Paint atoms in a circle around the cursor
            for dx in -brush.size..=brush.size {
                for dy in -brush.size..=brush.size {
                    let dist = (dx * dx + dy * dy) as f32;
                    if dist <= (brush.size * brush.size) as f32 {
                        let x = atom_x + dx;
                        let y = atom_y + dy;

                        if brush.atom_type == AtomType::Empty {
                            // Erase
                            world.0.set_atom(x, y, crate::atoms::Atom::default());
                        } else {
                            world.0.set_atom(x, y, crate::atoms::Atom {
                                atom_type: brush.atom_type,
                                velocity: Vec2::ZERO,
                                mass: brush.atom_type.mass(),
                                lifetime: None,
                                temperature: 20.0,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn spawn_demo_atoms(
    mut world: ResMut<AtomWorldResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // Spawn different atoms for testing
    static mut LAST_SPAWN: f32 = 0.0;

    unsafe {
        LAST_SPAWN += time.delta_seconds();
        if LAST_SPAWN > 0.5 { // Spawn every half second
            LAST_SPAWN = 0.0;

            if keyboard_input.pressed(KeyCode::KeyQ) {
                // Spawn sand
                world.0.set_atom(100, 50, crate::atoms::Atom {
                    atom_type: AtomType::Sand,
                    velocity: Vec2::ZERO,
                    mass: AtomType::Sand.mass(),
                    lifetime: None,
                    temperature: 20.0,
                });
            }

            if keyboard_input.pressed(KeyCode::KeyW) {
                // Spawn water
                world.0.set_atom(105, 50, crate::atoms::Atom {
                    atom_type: AtomType::Water,
                    velocity: Vec2::ZERO,
                    mass: AtomType::Water.mass(),
                    lifetime: None,
                    temperature: 20.0,
                });
            }

            if keyboard_input.pressed(KeyCode::KeyE) {
                // Spawn fire
                world.0.set_atom(110, 50, crate::atoms::Atom {
                    atom_type: AtomType::Fire,
                    velocity: Vec2::ZERO,
                    mass: AtomType::Fire.mass(),
                    lifetime: Some(5.0),
                    temperature: 800.0, // Hot fire
                });
            }

            if keyboard_input.pressed(KeyCode::KeyR) {
                // Spawn acid
                world.0.set_atom(115, 50, crate::atoms::Atom {
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
