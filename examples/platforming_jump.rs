// Example: Platforming with Jump Buffering and Coyote Time
// Based on "Making Platforming Feel Good" blog post
// https://www.slowrush.dev/news/making-platforming-feel-good

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
struct Player {
    speed: f32,
    jump_force: f32,
    is_grounded: bool,
    // Coyote time: extra time after leaving ground where you can still jump
    coyote_time: f32,
    coyote_timer: f32,
    // Jump buffering: press jump before landing and jump when you land
    jump_buffer_time: f32,
    jump_buffer_timer: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Platforming Jump - Making Platforming Feel Good".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .add_plugins(RapierDebugRenderPlugin::default())

        .add_systems(Startup, setup)
        .add_systems(Update, (player_input, update_player))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Spawn player
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(16.0, 32.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 100.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(8.0, 16.0),
        Velocity::zero(),
        LockedAxes::ROTATION_LOCKED,
        Player {
            speed: 150.0,
            jump_force: 300.0,
            is_grounded: false,
            coyote_time: 0.1, // 100ms coyote time
            coyote_timer: 0.0,
            jump_buffer_time: 0.1, // 100ms jump buffer
            jump_buffer_timer: 0.0,
        },
    ));

    // Spawn platforms
    spawn_platforms(commands);
}

fn spawn_platforms(mut commands: Commands) {
    let platforms = vec![
        // Ground
        (Vec2::new(0.0, -100.0), Vec2::new(400.0, 20.0)),
        // Floating platforms
        (Vec2::new(-150.0, 0.0), Vec2::new(100.0, 20.0)),
        (Vec2::new(150.0, 50.0), Vec2::new(100.0, 20.0)),
        (Vec2::new(0.0, 150.0), Vec2::new(100.0, 20.0)),
        // Moving platform
        (Vec2::new(0.0, -50.0), Vec2::new(80.0, 20.0)),
    ];

    for (pos, size) in platforms {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.5, 0.5),
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(0.0)),
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(size.x / 2.0, size.y / 2.0),
        ));
    }
}

fn player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Player>,
) {
    for mut player in query.iter_mut() {
        // Jump buffering: record jump input
        if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::ArrowUp) {
            player.jump_buffer_timer = player.jump_buffer_time;
        }
    }
}

fn update_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Velocity, &Transform)>,
    rapier_context: Res<RapierContext>,
) {
    let dt = time.delta_seconds();

    for (mut player, mut velocity, transform) in query.iter_mut() {
        // Horizontal movement
        let mut movement = 0.0;
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            movement -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            movement += 1.0;
        }

        velocity.linvel.x = movement * player.speed;

        // Ground check using raycast (more accurate than intersection)
        let ray_pos = transform.translation.truncate() + Vec2::new(0.0, -17.0);
        let ray_dir = Vec2::new(0.0, -5.0);
        let max_toi = 4.0;
        let solid = true;
        let filter = QueryFilter::default();

        let ground_hit = rapier_context.cast_ray(
            ray_pos, ray_dir, max_toi, solid, filter,
        );

        player.is_grounded = ground_hit.is_some();

        // Update coyote time
        if player.is_grounded {
            player.coyote_timer = player.coyote_time;
        } else {
            player.coyote_timer = (player.coyote_timer - dt).max(0.0);
        }

        // Update jump buffer
        player.jump_buffer_timer = (player.jump_buffer_timer - dt).max(0.0);

        // Jump if buffered and within coyote time
        if player.jump_buffer_timer > 0.0 && player.coyote_timer > 0.0 {
            velocity.linvel.y = player.jump_force;
            player.jump_buffer_timer = 0.0;
            player.coyote_timer = 0.0;
        }

        // Variable jump height (optional)
        if !keyboard_input.pressed(KeyCode::Space) && !keyboard_input.pressed(KeyCode::ArrowUp) && velocity.linvel.y > 0.0 {
            velocity.linvel.y *= 0.5; // Cut jump short if button released
        }
    }
}

// Bonus: Moving platform example
#[derive(Component)]
struct MovingPlatform {
    speed: f32,
    distance: f32,
    start_pos: Vec2,
    direction: Vec2,
}

fn add_moving_platform(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.7, 0.3),
                custom_size: Some(Vec2::new(80.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -50.0, 0.0),
            ..default()
        },
        RigidBody::KinematicVelocityBased,
        Collider::cuboid(40.0, 10.0),
        Velocity::zero(),
        MovingPlatform {
            speed: 50.0,
            distance: 100.0,
            start_pos: Vec2::new(0.0, -50.0),
            direction: Vec2::new(1.0, 0.0),
        },
    ));
}

fn update_moving_platforms(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut MovingPlatform)>,
) {
    for (mut transform, mut velocity, mut platform) in query.iter_mut() {
        let current_pos = transform.translation.truncate();
        let distance_from_start = (current_pos - platform.start_pos).length();

        if distance_from_start >= platform.distance {
            platform.direction = -platform.direction;
        }

        velocity.linvel = platform.direction * platform.speed;
    }
}
