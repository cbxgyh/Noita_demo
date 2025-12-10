// Example: Let Sleeping Atoms Lie
// Based on "Let Sleeping Atoms Lie" blog post
// https://www.slowrush.dev/news/let-sleeping-atoms-lie

use bevy::prelude::*;
use std::collections::HashSet;

// Atom sleeping system - don't update atoms that haven't moved recently
// Performance optimization for large atom worlds

#[derive(Clone, Debug)]
enum AtomType {
    Empty,
    Sand,
    Water,
    Stone,
}

impl AtomType {
    fn color(&self) -> Color {
        match self {
            AtomType::Empty => Color::rgba(0.0, 0.0, 0.0, 0.0),
            AtomType::Sand => Color::rgb(0.8, 0.7, 0.5),
            AtomType::Water => Color::rgba(0.2, 0.4, 0.8, 0.8),
            AtomType::Stone => Color::rgb(0.4, 0.4, 0.4),
        }
    }

    fn can_sleep(&self) -> bool {
        match self {
            AtomType::Empty => false,
            AtomType::Stone => true, // Stone rarely moves
            AtomType::Sand => false, // Sand moves frequently
            AtomType::Water => false, // Water moves frequently
        }
    }
}

#[derive(Clone, Debug)]
struct SleepingAtom {
    atom_type: AtomType,
    position: Vec2,
    velocity: Vec2,
    sleep_timer: f32,
    is_sleeping: bool,
    last_position: Vec2,
}

impl SleepingAtom {
    fn new(atom_type: AtomType, position: Vec2) -> Self {
        Self {
            atom_type,
            position,
            velocity: Vec2::ZERO,
            sleep_timer: 0.0,
            is_sleeping: false,
            last_position: position,
        }
    }

    fn update(&mut self, dt: f32, world_bounds: Vec2) -> bool {
        if self.atom_type == AtomType::Empty {
            return false;
        }

        let was_moved = false;

        if !self.is_sleeping {
            // Normal physics update
            self.velocity.y -= 30.0 * dt;

            let new_position = self.position + self.velocity * dt;

            // Simple bounds checking
            if new_position.x < 0.0 {
                self.position.x = 0.0;
                self.velocity.x *= -0.5;
            } else if new_position.x >= world_bounds.x {
                self.position.x = world_bounds.x - 1.0;
                self.velocity.x *= -0.5;
            }

            if new_position.y < 0.0 {
                self.position.y = 0.0;
                self.velocity.y *= -0.5;
            } else if new_position.y >= world_bounds.y {
                self.position.y = world_bounds.y - 1.0;
                self.velocity.y *= -0.5;
            } else {
                self.position = new_position;
            }

            // Apply friction
            self.velocity *= 0.98;

            // Check if atom should fall asleep
            let moved = (self.position - self.last_position).length_squared() > 0.001;
            self.last_position = self.position;

            if !moved && self.atom_type.can_sleep() {
                self.sleep_timer += dt;
                if self.sleep_timer > 1.0 { // Sleep after 1 second of inactivity
                    self.is_sleeping = true;
                    self.sleep_timer = 0.0;
                }
            } else {
                self.sleep_timer = 0.0;
            }
        } else {
            // Check if sleeping atom should wake up
            // Wake up if disturbed by neighboring atoms or external forces
        }

        was_moved
    }

    fn wake_up(&mut self) {
        self.is_sleeping = false;
        self.sleep_timer = 0.0;
    }

    fn disturb_neighbors(&mut self, world: &mut SleepingWorld, x: usize, y: usize) {
        // Wake up neighboring atoms when this atom moves
        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];

        for (nx, ny) in neighbors {
            if let Some(atom) = world.get_atom_mut(nx, ny) {
                if atom.is_sleeping {
                    atom.wake_up();
                }
            }
        }
    }
}

struct SleepingWorld {
    width: usize,
    height: usize,
    atoms: Vec<SleepingAtom>,
    bounds: Vec2,
    active_atoms: HashSet<(usize, usize)>,
}

impl SleepingWorld {
    fn new(width: usize, height: usize) -> Self {
        let mut atoms = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let mut atom_type = AtomType::Empty;

                // Create terrain
                if y < 10 {
                    atom_type = AtomType::Stone;
                } else if y < 25 && x > width / 2 - 20 && x < width / 2 + 20 {
                    atom_type = AtomType::Sand;
                }

                atoms.push(SleepingAtom::new(atom_type, Vec2::new(x as f32, y as f32)));
            }
        }

        Self {
            width,
            height,
            atoms,
            bounds: Vec2::new(width as f32, height as f32),
            active_atoms: HashSet::new(),
        }
    }

    fn update(&mut self, dt: f32) {
        self.active_atoms.clear();

        // Update non-sleeping atoms
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if !self.atoms[idx].is_sleeping {
                    let was_moved = self.atoms[idx].update(dt, self.bounds);

                    if was_moved {
                        self.active_atoms.insert((x, y));
                        // Wake up neighbors
                        self.atoms[idx].disturb_neighbors(self, x, y);
                    }
                }
            }
        }

        // Handle collisions between active atoms
        self.handle_active_collisions();
    }

    fn handle_active_collisions(&mut self) {
        let active_list: Vec<(usize, usize)> = self.active_atoms.iter().cloned().collect();

        for &(x, y) in &active_list {
            if let Some(atom) = self.get_atom_mut(x, y) {
                if atom.atom_type == AtomType::Sand {
                    // Check if sand should fall
                    if self.is_empty(x, y + 1) {
                        self.swap_atoms(x, y, x, y + 1);
                    } else if self.is_empty(x - 1, y + 1) {
                        self.swap_atoms(x, y, x - 1, y + 1);
                    } else if self.is_empty(x + 1, y + 1) {
                        self.swap_atoms(x, y, x + 1, y + 1);
                    }
                } else if atom.atom_type == AtomType::Water {
                    // Water flows
                    if self.is_empty(x, y + 1) {
                        self.swap_atoms(x, y, x, y + 1);
                    } else {
                        let dir = if rand::random::<bool>() { -1 } else { 1 };
                        if self.is_empty(x + dir, y) {
                            self.swap_atoms(x, y, x + dir, y);
                        }
                    }
                }
            }
        }
    }

    fn get_atom(&self, x: usize, y: usize) -> Option<&SleepingAtom> {
        if x < self.width && y < self.height {
            Some(&self.atoms[y * self.width + x])
        } else {
            None
        }
    }

    fn get_atom_mut(&mut self, x: usize, y: usize) -> Option<&mut SleepingAtom> {
        if x < self.width && y < self.height {
            Some(&mut self.atoms[y * self.width + x])
        } else {
            None
        }
    }

    fn is_empty(&self, x: usize, y: usize) -> bool {
        self.get_atom(x, y).map_or(true, |a| matches!(a.atom_type, AtomType::Empty))
    }

    fn swap_atoms(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        if x1 < self.width && y1 < self.height && x2 < self.width && y2 < self.height {
            let idx1 = y1 * self.width + x1;
            let idx2 = y2 * self.width + x2;

            self.atoms.swap(idx1, idx2);

            // Mark both positions as active
            self.active_atoms.insert((x1, y1));
            self.active_atoms.insert((x2, y2));

            // Wake up the moved atoms
            self.atoms[idx1].wake_up();
            self.atoms[idx2].wake_up();
        }
    }

    fn add_force_at(&mut self, position: Vec2, force: Vec2, radius: f32) {
        let center_x = position.x as usize;
        let center_y = position.y as usize;
        let radius_int = radius.ceil() as usize;

        for dy in -(radius_int as i32)..=(radius_int as i32) {
            for dx in -(radius_int as i32)..=(radius_int as i32) {
                let x = center_x as i32 + dx;
                let y = center_y as i32 + dy;

                if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                    let distance = Vec2::new(dx as f32, dy as f32).length();
                    if distance <= radius {
                        if let Some(atom) = self.get_atom_mut(x as usize, y as usize) {
                            if atom.atom_type != AtomType::Empty {
                                let falloff = 1.0 - (distance / radius);
                                atom.velocity += force * falloff;
                                atom.wake_up();
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_sleeping_stats(&self) -> (usize, usize, f32) {
        let total_atoms = self.width * self.height;
        let sleeping_atoms = self.atoms.iter().filter(|a| a.is_sleeping).count();
        let sleeping_percentage = (sleeping_atoms as f32 / total_atoms as f32) * 100.0;

        (total_atoms, sleeping_atoms, sleeping_percentage)
    }
}

#[derive(Resource)]
struct SleepingWorldResource(SleepingWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Let Sleeping Atoms Lie - Performance Optimization".to_string(),
                resolution: (1000.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(SleepingWorldResource(SleepingWorld::new(200, 150)))
        .add_systems(Startup, setup_sleeping_demo)
        .add_systems(Update, (
            update_sleeping_world,
            render_sleeping_atoms,
            handle_sleeping_input,
            display_sleeping_stats,
        ).chain())
        .run();
}

fn setup_sleeping_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_sleeping_world(mut world: ResMut<SleepingWorldResource>, time: Res<Time>) {
    let dt = time.delta_seconds().min(1.0 / 30.0);
    world.0.update(dt);
}

fn render_sleeping_atoms(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    world: Res<SleepingWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render atoms
    for atom in &world.0.atoms {
        if atom.atom_type != AtomType::Empty {
            let mut color = atom.atom_type.color();

            // Highlight sleeping atoms with different brightness
            if atom.is_sleeping {
                color = color.with_r(color.r() * 0.7)
                           .with_g(color.g() * 0.7)
                           .with_b(color.b() * 0.7);
            } else {
                // Highlight active atoms
                color = color.with_r((color.r() + 0.3).min(1.0))
                           .with_g((color.g() + 0.3).min(1.0))
                           .with_b((color.b() + 0.3).min(1.0));
            }

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
}

fn handle_sleeping_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<SleepingWorldResource>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        let atom_x = (world_pos.origin.x + world.0.width as f32 / 2.0) as f32;
                        let atom_y = (world_pos.origin.y + world.0.height as f32 / 2.0) as f32;
                        let position = Vec2::new(atom_x, atom_y);

                        // Apply force to wake up atoms
                        world.0.add_force_at(position, Vec2::new(0.0, 50.0), 15.0);
                    }
                }
            }
        }
    }
}

fn display_sleeping_stats(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    world: Res<SleepingWorldResource>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        let (total, sleeping, percentage) = world.0.get_sleeping_stats();

        println!("\n=== Sleeping Atoms Performance Stats ===");
        println!("Total atoms: {}", total);
        println!("Sleeping atoms: {} ({:.1}%)", sleeping, percentage);
        println!("Active atoms: {}", total - sleeping);
        println!("Performance boost: {:.1}x", total as f32 / (total - sleeping) as f32);
        println!("\nControls:");
        println!("Left click: Apply force (wakes up atoms)");
        println!("H: Show this stats");
        println!("======================================\n");
    }
}
