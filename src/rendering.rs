use bevy::prelude::*;
use crate::atoms::{AtomWorldResource, AtomType};

// Pixel-perfect camera setup as described in "Pixel Perfect Rendering" blog post
pub fn setup_pixel_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1000.0),
        ..default()
    });
}

// System to render atoms as sprites
pub fn render_atoms(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    world: Res<AtomWorldResource>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Clear previous atom entities
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Spawn new sprites for each atom
    for y in 0..world.0.height {
        for x in 0..world.0.width {
            if let Some(atom) = world.0.get_atom(x as i32, y as i32) {
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

// Pixel-perfect camera controller
#[derive(Component)]
pub struct PixelCamera {
    pub pixels_per_unit: f32,
}

pub fn pixel_perfect_camera(
    mut query: Query<(&mut Transform, &PixelCamera)>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let (mut transform, camera) = query.single_mut();

    // Ensure camera position is pixel-aligned
    let pixel_size = 1.0 / camera.pixels_per_unit;
    transform.translation.x = (transform.translation.x / pixel_size).round() * pixel_size;
    transform.translation.y = (transform.translation.y / pixel_size).round() * pixel_size;

    // Adjust projection to be pixel-perfect
    // This ensures sprites align to pixel boundaries
}

// Smoothing filter for pixel art as mentioned in the blog
pub fn apply_pixel_art_filter(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // This would implement the pixel art smoothing filter
    // For now, we'll keep it simple
}

// Camera controller for following player
pub fn camera_follow_player(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, (With<crate::game::Player>, Without<Camera>)>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) = (camera_query.get_single_mut(), player_query.get_single()) {
        let target = player_transform.translation;
        let current = camera_transform.translation;

        // Smooth camera follow
        let lerp_factor = 0.1;
        camera_transform.translation = current.lerp(target, lerp_factor);
    }
}
