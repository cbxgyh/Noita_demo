use bevy::prelude::*;
use crate::atoms::{AtomWorldResource, Atom, AtomType};
use crate::level_generation::{LevelManager, LevelType};

// Custom Level Editor as described in "A Custom Level Editor" blog post
// Pixel-based level editor that supports placing enemies and hazards

#[derive(Resource)]
pub struct LevelEditor {
    pub is_active: bool,
    pub selected_atom_type: AtomType,
    pub brush_size: i32,
    pub mode: EditorMode,
    pub current_level_data: Vec<(i32, i32, AtomType)>, // For undo/redo
}

#[derive(Debug, Clone, Copy)]
pub enum EditorMode {
    Draw,      // Draw atoms
    Erase,     // Erase atoms
    Fill,      // Fill area
    Rectangle, // Draw rectangles
    Circle,    // Draw circles
    Line,      // Draw lines
    Entity,    // Place entities (enemies, hazards, etc.)
}

impl Default for LevelEditor {
    fn default() -> Self {
        Self {
            is_active: false,
            selected_atom_type: AtomType::Stone,
            brush_size: 3,
            mode: EditorMode::Draw,
            current_level_data: Vec::new(),
        }
    }
}

#[derive(Component)]
pub struct EditorCursor;

#[derive(Component)]
pub struct LevelEntity {
    pub entity_type: EntityType,
    pub position: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum EntityType {
    Enemy,
    Hazard,
    Collectible,
    SpawnPoint,
    Exit,
}

// Editor UI state
#[derive(Resource)]
pub struct EditorUI {
    pub show_grid: bool,
    pub show_entities: bool,
    pub snap_to_grid: bool,
    pub grid_size: f32,
}

impl Default for EditorUI {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_entities: true,
            snap_to_grid: true,
            grid_size: 1.0,
        }
    }
}

// Systems for level editor
pub fn setup_level_editor(
    mut commands: Commands,
    mut editor: ResMut<LevelEditor>,
) {
    // Create editor cursor
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..default()
        },
        EditorCursor,
    ));

    // Initially hide editor
    editor.is_active = false;
}

pub fn toggle_level_editor(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<LevelEditor>,
    mut cursor_query: Query<&mut Visibility, With<EditorCursor>>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        editor.is_active = !editor.is_active;

        if let Ok(mut visibility) = cursor_query.get_single_mut() {
            *visibility = if editor.is_active {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn update_editor_cursor(
    editor: Res<LevelEditor>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut cursor_query: Query<&mut Transform, With<EditorCursor>>,
) {
    if !editor.is_active {
        return;
    }

    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    let mut cursor_transform = cursor_query.single_mut();

                    // Snap to grid if enabled
                    let pos = if true { // snap_to_grid
                        let grid_size = 1.0;
                        Vec2::new(
                            (world_pos.origin.x / grid_size).round() * grid_size,
                            (world_pos.origin.y / grid_size).round() * grid_size,
                        )
                    } else {
                        world_pos.origin.truncate()
                    };

                    cursor_transform.translation = pos.extend(10.0);

                    // Update cursor size based on brush size
                    // (Would update sprite size here)
                }
            }
        }
    }
}

pub fn editor_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<LevelEditor>,
    mut world: ResMut<AtomWorldResource>,
    cursor_query: Query<&Transform, With<EditorCursor>>,
) {
    if !editor.is_active {
        return;
    }

    // Change brush size
    if keyboard_input.just_pressed(KeyCode::Equal) { // +
        editor.brush_size = (editor.brush_size + 1).min(20);
    }
    if keyboard_input.just_pressed(KeyCode::Minus) { // -
        editor.brush_size = (editor.brush_size - 1).max(1);
    }

    // Change atom type
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        editor.selected_atom_type = AtomType::Sand;
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        editor.selected_atom_type = AtomType::Water;
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        editor.selected_atom_type = AtomType::Stone;
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        editor.selected_atom_type = AtomType::Acid;
    }
    if keyboard_input.just_pressed(KeyCode::Digit5) {
        editor.selected_atom_type = AtomType::Fire;
    }

    // Change mode
    if keyboard_input.just_pressed(KeyCode::KeyD) {
        editor.mode = EditorMode::Draw;
    }
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        editor.mode = EditorMode::Erase;
    }
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        editor.mode = EditorMode::Fill;
    }

    // Apply editor action
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok(cursor_transform) = cursor_query.get_single() {
            let pos = cursor_transform.translation.truncate();

            match editor.mode {
                EditorMode::Draw => {
                    draw_with_brush(&mut world.0, pos, editor.selected_atom_type, editor.brush_size);
                }
                EditorMode::Erase => {
                    draw_with_brush(&mut world.0, pos, AtomType::Empty, editor.brush_size);
                }
                EditorMode::Fill => {
                    // Implement flood fill
                    flood_fill_area(&mut world.0, pos, editor.selected_atom_type);
                }
                _ => {}
            }
        }
    }
}

fn draw_with_brush(world: &mut crate::atoms::AtomWorld, center: Vec2, atom_type: AtomType, size: i32) {
    let start_x = (center.x - size as f32 / 2.0) as i32;
    let start_y = (center.y - size as f32 / 2.0) as i32;
    let end_x = start_x + size;
    let end_y = start_y + size;

    for x in start_x..end_x {
        for y in start_y..end_y {
            let dx = x as f32 - center.x;
            let dy = y as f32 - center.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= size as f32 / 2.0 {
                world.set_atom(x, y, Atom {
                    atom_type,
                    velocity: Vec2::ZERO,
                    mass: atom_type.mass(),
                    lifetime: if atom_type == AtomType::Fire { Some(10.0) } else { None },
                    temperature: if atom_type == AtomType::Fire { 700.0 } else { 20.0 },
                });
            }
        }
    }
}

fn flood_fill_area(world: &mut crate::atoms::AtomWorld, start_pos: Vec2, fill_type: AtomType) {
    let start_x = start_pos.x as i32;
    let start_y = start_pos.y as i32;

    if let Some(start_atom) = world.get_atom(start_x, start_y) {
        let target_type = start_atom.atom_type;
        if target_type == fill_type {
            return; // Already the right type
        }

        let mut stack = vec![(start_x, start_y)];
        let mut visited = std::collections::HashSet::new();

        while let Some((x, y)) = stack.pop() {
            if visited.contains(&(x, y)) {
                continue;
            }
            visited.insert((x, y));

            if let Some(atom) = world.get_atom(x, y) {
                if atom.atom_type == target_type {
                    world.set_atom(x, y, Atom {
                        atom_type: fill_type,
                        velocity: Vec2::ZERO,
                        mass: fill_type.mass(),
                        lifetime: if fill_type == AtomType::Fire { Some(10.0) } else { None },
                        temperature: if fill_type == AtomType::Fire { 700.0 } else { 20.0 },
                    });

                    // Add neighbors
                    stack.push((x + 1, y));
                    stack.push((x - 1, y));
                    stack.push((x, y + 1));
                    stack.push((x, y - 1));
                }
            }
        }
    }
}

// Save/load level functionality
pub fn save_level(
    world: Res<AtomWorldResource>,
    level_manager: Res<LevelManager>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F2) {
        // Save current level to file
        // In a real implementation, this would serialize the atom world
        println!("Level saved!");
    }
}

pub fn load_level_editor(
    mut world: ResMut<AtomWorldResource>,
    level_manager: Res<LevelManager>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        // Load level from file
        // In a real implementation, this would deserialize from file
        *world = AtomWorldResource(level_manager.generate_current_level(200, 150));
        println!("Level loaded!");
    }
}

// Undo/Redo system (basic implementation)
#[derive(Resource)]
pub struct EditorHistory {
    pub undo_stack: Vec<Vec<(i32, i32, Atom)>>,
    pub redo_stack: Vec<Vec<(i32, i32, Atom)>>,
    pub max_history: usize,
}

impl Default for EditorHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }
}

pub fn editor_undo_redo(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut history: ResMut<EditorHistory>,
    mut world: ResMut<AtomWorldResource>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyZ) &&
       (keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight)) {

        if let Some(changes) = history.undo_stack.pop() {
            // Store current state for redo
            let current_state = changes.iter().map(|(x, y, _)| {
                (*x, *y, world.0.get_atom(*x, *y).cloned().unwrap_or_default())
            }).collect();

            history.redo_stack.push(current_state);

            // Apply undo changes
            for (x, y, atom) in changes {
                world.0.set_atom(x, y, atom);
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyY) &&
       (keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight)) {

        if let Some(changes) = history.redo_stack.pop() {
            // Store current state for undo
            let current_state = changes.iter().map(|(x, y, _)| {
                (*x, *y, world.0.get_atom(*x, *y).cloned().unwrap_or_default())
            }).collect();

            history.undo_stack.push(current_state);

            // Apply redo changes
            for (x, y, atom) in changes {
                world.0.set_atom(x, y, atom);
            }
        }
    }
}

// Level validation system
pub fn validate_level(world: Res<AtomWorldResource>) {
    // Check for basic level requirements:
    // - Has player spawn point
    // - Has exit
    // - Is playable (not completely blocked)
    // - Has some interactive elements

    let mut has_spawn = false;
    let mut has_exit = false;
    let mut atom_count = 0;

    // This would need to be integrated with entity placement system
    // For now, just count atoms
    for y in 0..world.0.height {
        for x in 0..world.0.width {
            if let Some(atom) = world.0.get_atom(x as i32, y as i32) {
                if atom.atom_type != AtomType::Empty {
                    atom_count += 1;
                }
            }
        }
    }

    println!("Level validation: {} atoms", atom_count);
    if atom_count < 100 {
        println!("Warning: Level seems too empty!");
    }
}
