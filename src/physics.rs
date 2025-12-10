use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::atoms::{AtomWorldResource, AtomType};

// Physics bridge as described in "Bridging Physics Worlds" blog post
#[derive(Component)]
pub struct RigidBodyObject {
    pub atom_type: AtomType,
    pub collider_points: Vec<Vec2>,
}

// System to create colliders from atom terrain
pub fn create_terrain_colliders(
    mut commands: Commands,
    world: Res<AtomWorldResource>,
) {
    // Group solid atoms into collision shapes
    let mut solid_regions = find_solid_regions(&world.0);

    for region in solid_regions {
        if region.len() < 3 { continue; }

        // Create simplified collider from region points
        let collider_shape = create_convex_hull(&region);

        if Collider::convex_hull(&collider_shape).is_some() {
            commands.spawn((
                RigidBody::Fixed,
                Collider::convex_hull(&collider_shape).unwrap(),
                Transform::default(),
            ));
        }

    }
}

// Find contiguous regions of solid atoms
fn find_solid_regions(world: &crate::atoms::AtomWorld) -> Vec<Vec<Vec2>> {
    let mut visited = vec![false; world.width * world.height];
    let mut regions = Vec::new();

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            if visited[idx] { continue; }

            if let Some(atom) = world.get_atom(x as i32, y as i32) {
                if is_solid(atom.atom_type) {
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

fn is_solid(atom_type: AtomType) -> bool {
    matches!(atom_type, AtomType::Stone | AtomType::Sand)
}

fn flood_fill(
    world: &crate::atoms::AtomWorld,
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

        if let Some(atom) = world.get_atom(x as i32, y as i32) {
            if is_solid(atom.atom_type) {
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

// Create convex hull using gift wrapping algorithm
fn create_convex_hull(points: &[Vec2]) -> Vec<Vec2> {
    if points.len() <= 3 {
        return points.to_vec();
    }

    // Find leftmost point
    let mut leftmost = 0;
    for i in 1..points.len() {
        if points[i].x < points[leftmost].x {
            leftmost = i;
        }
    }

    let mut hull = Vec::new();
    let mut p = leftmost;
    let mut q;

    loop {
        hull.push(points[p]);

        q = (p + 1) % points.len();

        for i in 0..points.len() {
            if orientation(points[p], points[i], points[q]) < 0.0 {
                q = i;
            }
        }

        p = q;

        if p == leftmost {
            break;
        }
    }

    hull
}

fn orientation(p: Vec2, q: Vec2, r: Vec2) -> f32 {
    (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y)
}

// Optimize colliders using ear clipping as mentioned in "Optimizing the Physics Bridge"
pub fn optimize_colliders_with_ear_clipping(
    mut commands: Commands,
    world: Res<AtomWorldResource>,
) {
    // This would implement the ear clipping triangulation
    // For performance as described in the blog post
    // Simplified version for now
    create_terrain_colliders(commands, world);
}

// Atoms interacting with rigid bodies
pub fn atoms_push_rigid_bodies(
    mut rigid_bodies: Query<(&mut Velocity, &Transform), With<RigidBody>>,
    world: Res<AtomWorldResource>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for (mut velocity, transform) in rigid_bodies.iter_mut() {
        let pos = transform.translation.truncate();
        let size = Vec2::new(1.0, 1.0); // Assuming 1x1 objects for now

        // Calculate force from surrounding atoms
        let force = calculate_atom_force(&world.0, pos, size);
        velocity.linvel += force * dt;
    }
}

fn calculate_atom_force(world: &crate::atoms::AtomWorld, pos: Vec2, size: Vec2) -> Vec2 {
    let mut force = Vec2::ZERO;
    let half_size = size / 2.0;

    // Sample atoms around the object
    let samples = 8;
    for i in 0..samples {
        let angle = (i as f32 / samples as f32) * std::f32::consts::TAU;
        let sample_pos = pos + Vec2::new(angle.cos(), angle.sin()) * half_size.x;

        let x = sample_pos.x.round() as i32;
        let y = sample_pos.y.round() as i32;

        if let Some(atom) = world.get_atom(x, y) {
            if atom.atom_type != AtomType::Empty {
                // Push away from atoms
                let atom_pos = Vec2::new(x as f32, y as f32);
                let dir = (pos - atom_pos).normalize();
                let strength = atom.atom_type.density();
                force += dir * strength * 0.1;
            }
        }
    }

    force
}

// Moving bodies displace atoms
pub fn rigid_bodies_displace_atoms(
    rigid_bodies: Query<(&Transform, &Velocity), With<RigidBody>>,
    mut world: ResMut<AtomWorldResource>,
) {
    for (transform, velocity) in rigid_bodies.iter() {
        let pos = transform.translation.truncate();
        let vel = velocity.linvel;

        if vel.length_squared() > 0.1 {
            // Object is moving, displace atoms
            displace_atoms_around_point(&mut world.0, pos, vel);
        }
    }
}

fn displace_atoms_around_point(world: &mut crate::atoms::AtomWorld, pos: Vec2, force: Vec2) {
    let radius = 2.0;
    let force_strength = force.length();

    for dx in -(radius as i32)..=(radius as i32) {
        for dy in -(radius as i32)..=(radius as i32) {
            let dist = Vec2::new(dx as f32, dy as f32).length();
            if dist <= radius {
                let x = (pos.x + dx as f32) as i32;
                let y = (pos.y + dy as f32) as i32;

                if let Some(atom) = world.get_atom_mut(x, y) {
                    if atom.atom_type != AtomType::Empty && !atom.atom_type.is_gas() {
                        // Move atom in direction of force
                        let push_dir = Vec2::new(dx as f32, dy as f32).normalize();
                        let push_x = x + (push_dir.x * force_strength * 0.5) as i32;
                        let push_y = y + (push_dir.y * force_strength * 0.5) as i32;

                        if world.is_empty(push_x, push_y) {
                            world.swap_atoms(x, y, push_x, push_y);
                        }
                    }
                }
            }
        }
    }
}
