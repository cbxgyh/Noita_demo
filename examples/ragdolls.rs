// Example: Ragdolls
// Based on "Ragdolls" blog post
// https://www.slowrush.dev/news/ragdolls

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Ragdoll physics system with connected rigid bodies
// Demonstrates how to create realistic character physics

#[derive(Component)]
struct RagdollLimb {
    bone_type: BoneType,
}

#[derive(Clone, Debug)]
enum BoneType {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftForearm,
    RightForearm,
    LeftLeg,
    RightLeg,
    LeftShin,
    RightShin,
}

impl BoneType {
    fn size(&self) -> Vec2 {
        match self {
            BoneType::Head => Vec2::new(12.0, 12.0),
            BoneType::Torso => Vec2::new(16.0, 24.0),
            BoneType::LeftArm | BoneType::RightArm => Vec2::new(6.0, 12.0),
            BoneType::LeftForearm | BoneType::RightForearm => Vec2::new(6.0, 12.0),
            BoneType::LeftLeg | BoneType::RightLeg => Vec2::new(8.0, 16.0),
            BoneType::LeftShin | BoneType::RightShin => Vec2::new(8.0, 16.0),
        }
    }

    fn mass(&self) -> f32 {
        match self {
            BoneType::Head => 5.0,
            BoneType::Torso => 20.0,
            BoneType::LeftArm | BoneType::RightArm => 3.0,
            BoneType::LeftForearm | BoneType::RightForearm => 2.0,
            BoneType::LeftLeg | BoneType::RightLeg => 5.0,
            BoneType::LeftShin | BoneType::RightShin => 4.0,
        }
    }

    fn color(&self) -> Color {
        match self {
            BoneType::Head => Color::rgb(0.9, 0.7, 0.5), // Skin color
            BoneType::Torso => Color::rgb(0.3, 0.6, 0.8), // Shirt blue
            BoneType::LeftArm | BoneType::RightArm => Color::rgb(0.9, 0.7, 0.5),
            BoneType::LeftForearm | BoneType::RightForearm => Color::rgb(0.9, 0.7, 0.5),
            BoneType::LeftLeg | BoneType::RightLeg => Color::rgb(0.4, 0.4, 0.4), // Pants gray
            BoneType::LeftShin | BoneType::RightShin => Color::rgb(0.4, 0.4, 0.4),
        }
    }
}

#[derive(Component)]
struct RagdollJoint {
    bone_a: Entity,
    bone_b: Entity,
    local_anchor_a: Vec2,
    local_anchor_b: Vec2,
    joint_type: JointType,
}

#[derive(Clone, Debug)]
enum JointType {
    Revolute,
    Fixed,
    Prismatic,
}

#[derive(Component)]
struct Ragdoll {
    root_entity: Entity,
    limbs: Vec<Entity>,
    joints: Vec<Entity>,
    is_ragdoll: bool, // Whether it's in ragdoll mode
}

impl Ragdoll {
    fn new() -> Self {
        Self {
            root_entity: Entity::PLACEHOLDER,
            limbs: Vec::new(),
            joints: Vec::new(),
            is_ragdoll: false,
        }
    }

    fn activate_ragdoll(&mut self, commands: &mut Commands) {
        self.is_ragdoll = true;

        // Disconnect joints to allow free movement
        for &joint_entity in &self.joints {
            commands.entity(joint_entity).remove::<ImpulseJoint>();
        }
    }

    fn deactivate_ragdoll(&mut self, commands: &mut Commands) {
        self.is_ragdoll = false;

        // Reconnect joints
        // In a real implementation, this would restore the original joint constraints
        // For demo purposes, we just set the flag
    }
}

fn create_ragdoll(
    commands: &mut Commands,
    position: Vec2,
) -> Ragdoll {
    let mut ragdoll = Ragdoll::new();
    let mut limb_entities = Vec::new();

    // Create limbs
    let bones = vec![
        (BoneType::Torso, Vec2::new(0.0, 0.0)),
        (BoneType::Head, Vec2::new(0.0, 15.0)),
        (BoneType::LeftArm, Vec2::new(-10.0, 5.0)),
        (BoneType::RightArm, Vec2::new(10.0, 5.0)),
        (BoneType::LeftForearm, Vec2::new(-10.0, -5.0)),
        (BoneType::RightForearm, Vec2::new(10.0, -5.0)),
        (BoneType::LeftLeg, Vec2::new(-5.0, -15.0)),
        (BoneType::RightLeg, Vec2::new(5.0, -15.0)),
        (BoneType::LeftShin, Vec2::new(-5.0, -25.0)),
        (BoneType::RightShin, Vec2::new(5.0, -25.0)),
    ];

    for (bone_type, offset) in bones {
        let world_pos = position + offset;
        let size = bone_type.size();

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: bone_type.color(),
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_translation(world_pos.extend(0.0)),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(size.x / 2.0, size.y / 2.0),
            RagdollLimb { bone_type },
            Velocity::zero(),
        )).id();

        limb_entities.push(entity);
    }

    ragdoll.limbs = limb_entities.clone();
    ragdoll.root_entity = limb_entities[0]; // Torso is root

    // Create joints
    let joints = vec![
        // Head to torso
        (0, 1, Vec2::new(0.0, 12.0), Vec2::new(0.0, -6.0), JointType::Revolute),
        // Left arm to torso
        (0, 2, Vec2::new(-8.0, 8.0), Vec2::new(0.0, 6.0), JointType::Revolute),
        // Right arm to torso
        (0, 3, Vec2::new(8.0, 8.0), Vec2::new(0.0, 6.0), JointType::Revolute),
        // Left forearm to left arm
        (2, 4, Vec2::new(0.0, -6.0), Vec2::new(0.0, 6.0), JointType::Revolute),
        // Right forearm to right arm
        (3, 5, Vec2::new(0.0, -6.0), Vec2::new(0.0, 6.0), JointType::Revolute),
        // Left leg to torso
        (0, 6, Vec2::new(-4.0, -12.0), Vec2::new(0.0, 8.0), JointType::Revolute),
        // Right leg to torso
        (0, 7, Vec2::new(4.0, -12.0), Vec2::new(0.0, 8.0), JointType::Revolute),
        // Left shin to left leg
        (6, 8, Vec2::new(0.0, -8.0), Vec2::new(0.0, 8.0), JointType::Revolute),
        // Right shin to right leg
        (7, 9, Vec2::new(0.0, -8.0), Vec2::new(0.0, 8.0), JointType::Revolute),
    ];

    for (bone_a_idx, bone_b_idx, anchor_a, anchor_b, joint_type) in joints {
        let bone_a = limb_entities[bone_a_idx];
        let bone_b = limb_entities[bone_b_idx];

        let joint = match joint_type {
            JointType::Revolute => {
                RevoluteJointBuilder::new()
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b)
            }
            JointType::Fixed => {
                FixedJointBuilder::new()
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b)
            }
            JointType::Prismatic => {
                PrismaticJointBuilder::new(Vec2::Y)
                    .local_anchor1(anchor_a)
                    .local_anchor2(anchor_b)
                    .limits([-10.0, 10.0])
            }
        };

        let joint_entity = commands.spawn((
            ImpulseJoint::new(bone_a, joint),
            RagdollJoint {
                bone_a,
                bone_b,
                local_anchor_a: anchor_a,
                local_anchor_b: anchor_b,
                joint_type,
            },
        )).id();

        ragdoll.joints.push(joint_entity);
    }

    commands.spawn(ragdoll.clone());

    ragdoll
}

fn setup_ragdoll_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Create ground
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.3, 0.3, 0.3),
                custom_size: Some(Vec2::new(400.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -150.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(200.0, 10.0),
    ));

    // Create ragdoll
    let _ragdoll = create_ragdoll(&mut commands, Vec2::new(0.0, 50.0));
}

fn handle_ragdoll_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ragdolls: Query<&mut Ragdoll>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut ragdoll in ragdolls.iter_mut() {
            if ragdoll.is_ragdoll {
                ragdoll.deactivate_ragdoll(&mut commands);
            } else {
                ragdoll.activate_ragdoll(&mut commands);
            }
        }
    }

    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset ragdoll position
        for ragdoll in ragdolls.iter() {
            if let Ok(mut transform) = commands.get_entity(ragdoll.root_entity).map(|e| e.get::<Transform>()) {
                if let Some(mut transform) = transform {
                    transform.translation = Vec3::new(0.0, 50.0, 0.0);
                }
            }
        }
    }
}

fn update_ragdoll_physics(
    time: Res<Time>,
    mut ragdolls: Query<&mut Ragdoll>,
    mut velocities: Query<&mut Velocity>,
) {
    let dt = time.delta_seconds();

    for ragdoll in ragdolls.iter() {
        if !ragdoll.is_ragdoll {
            // Apply some simple animation when not in ragdoll mode
            // In a real game, this would be proper character animation
            for &limb_entity in &ragdoll.limbs {
                if let Ok(mut velocity) = velocities.get_mut(limb_entity) {
                    // Apply damping to keep limbs together
                    velocity.linvel *= 0.95;
                    velocity.angvel *= 0.9;
                }
            }
        }
    }
}

fn apply_ragdoll_forces(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ragdolls: Query<&mut Ragdoll>,
    mut commands: Commands,
    mut external_forces: Query<&mut ExternalForce>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        // Apply force to torso to demonstrate ragdoll physics
        for ragdoll in ragdolls.iter() {
            if let Ok(mut force) = external_forces.get_mut(ragdoll.root_entity) {
                force.force = Vec2::new(500.0, 200.0);
            }
        }
    }
}

fn display_ragdoll_info(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ragdolls: Query<&Ragdoll>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        let ragdoll_count = ragdolls.iter().count();
        let ragdoll_mode = ragdolls.iter().next().map(|r| r.is_ragdoll).unwrap_or(false);

        println!("\n=== Ragdoll Physics Demo ===");
        println!("Ragdolls: {}", ragdoll_count);
        println!("Mode: {}", if ragdoll_mode { "Ragdoll (limp)" } else { "Animated" });
        println!("Limbs per ragdoll: 10 (head, torso, arms, legs)");
        println!("Joints per ragdoll: 9 (revolute joints)");
        println!("\nControls:");
        println!("Space: Toggle ragdoll mode");
        println!("F: Apply force to demonstrate physics");
        println!("R: Reset position");
        println!("H: Show this info");
        println!("=======================\n");
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ragdolls - Physics-Based Character Animation".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_ragdoll_demo)
        .add_systems(Update, (
            handle_ragdoll_input,
            update_ragdoll_physics,
            apply_ragdoll_forces,
            display_ragdoll_info,
        ).chain())
        .run();
}
