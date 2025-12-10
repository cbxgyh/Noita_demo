// Example: Optimizing the Physics Bridge
// Based on "Optimizing the Physics Bridge" blog post
// https://www.slowrush.dev/news/optimizing-the-physics-bridge

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Ear clipping triangulation for optimized collision meshes
// This demonstrates the performance optimization described in the blog

#[derive(Clone, Debug)]
struct Point2D {
    x: f32,
    y: f32,
}

impl Point2D {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

// Ear clipping triangulation algorithm
fn ear_clipping_triangulation(points: &[Point2D]) -> Vec<[Vec2; 3]> {
    if points.len() < 3 {
        return Vec::new();
    }

    let mut remaining_points: Vec<Point2D> = points.to_vec();
    let mut triangles = Vec::new();

    // Ensure counter-clockwise winding
    if !is_counter_clockwise(&remaining_points) {
        remaining_points.reverse();
    }

    while remaining_points.len() >= 3 {
        let mut ear_found = false;

        for i in 0..remaining_points.len() {
            if is_ear(&remaining_points, i) {
                // Create triangle
                let prev = if i == 0 { remaining_points.len() - 1 } else { i - 1 };
                let next = (i + 1) % remaining_points.len();

                let triangle = [
                    Vec2::new(remaining_points[prev].x, remaining_points[prev].y),
                    Vec2::new(remaining_points[i].x, remaining_points[i].y),
                    Vec2::new(remaining_points[next].x, remaining_points[next].y),
                ];

                triangles.push(triangle);

                // Remove the ear
                remaining_points.remove(i);
                ear_found = true;
                break;
            }
        }

        if !ear_found {
            // No ear found, break to avoid infinite loop
            break;
        }
    }

    triangles
}

fn is_counter_clockwise(points: &[Point2D]) -> bool {
    let mut sum = 0.0;
    for i in 0..points.len() {
        let p1 = &points[i];
        let p2 = &points[(i + 1) % points.len()];
        sum += (p2.x - p1.x) * (p2.y + p1.y);
    }
    sum > 0.0
}

fn is_ear(points: &[Point2D], index: usize) -> bool {
    let len = points.len();
    let prev = if index == 0 { len - 1 } else { index - 1 };
    let next = (index + 1) % len;

    let p1 = &points[prev];
    let p2 = &points[index];
    let p3 = &points[next];

    // Check if triangle is convex
    if !is_convex(p1, p2, p3, &points[(next + 1) % len]) {
        return false;
    }

    // Check if no other points are inside the triangle
    for i in 0..len {
        if i != prev && i != index && i != next {
            if point_in_triangle(&points[i], p1, p2, p3) {
                return false;
            }
        }
    }

    true
}

fn is_convex(p1: &Point2D, p2: &Point2D, p3: &Point2D, p4: &Point2D) -> bool {
    let cross1 = (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x);
    let cross2 = (p3.x - p2.x) * (p4.y - p2.y) - (p3.y - p2.y) * (p4.x - p2.x);
    cross1 > 0.0 && cross2 > 0.0
}

fn point_in_triangle(p: &Point2D, a: &Point2D, b: &Point2D, c: &Point2D) -> bool {
    let area = 0.5 * ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs();
    let area1 = 0.5 * ((a.x - p.x) * (b.y - p.y) - (b.x - p.x) * (a.y - p.y)).abs();
    let area2 = 0.5 * ((b.x - p.x) * (c.y - p.y) - (c.x - p.x) * (b.y - p.y)).abs();
    let area3 = 0.5 * ((c.x - p.x) * (a.y - p.y) - (a.x - p.x) * (c.y - p.y)).abs();

    (area1 + area2 + area3 - area).abs() < 0.001
}

// Optimized collision mesh generation
#[derive(Component)]
struct OptimizedCollider {
    triangles: Vec<[Vec2; 3]>,
    aabb: Rect,
}

impl OptimizedCollider {
    fn new(points: &[Point2D]) -> Self {
        let triangles = ear_clipping_triangulation(points);
        let aabb = calculate_aabb(points);

        Self { triangles, aabb }
    }
}

fn calculate_aabb(points: &[Point2D]) -> Rect {
    if points.is_empty() {
        return Rect::new(0.0, 0.0, 0.0, 0.0);
    }

    let mut min_x = points[0].x;
    let mut max_x = points[0].x;
    let mut min_y = points[0].y;
    let mut max_y = points[0].y;

    for point in points {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }

    Rect::new(min_x, min_y, max_x, max_y)
}

// Performance comparison system
#[derive(Resource)]
struct PerformanceStats {
    frame_count: u32,
    total_triangles: u32,
    average_triangles_per_frame: f32,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            frame_count: 0,
            total_triangles: 0,
            average_triangles_per_frame: 0.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Optimizing Physics Bridge - Ear Clipping Triangulation".to_string(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(PerformanceStats::default())
        .add_systems(Startup, setup_optimized_physics)
        .add_systems(Update, (
            update_performance_stats,
            demonstrate_triangulation,
            render_triangulation_debug,
        ).chain())
        .run();
}

fn setup_optimized_physics(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Create various shapes to demonstrate triangulation optimization
    create_demo_shapes(commands);
}

fn create_demo_shapes(mut commands: Commands) {
    // Complex concave shape that benefits from triangulation
    let complex_shape = vec![
        Point2D::new(0.0, 0.0),
        Point2D::new(100.0, 0.0),
        Point2D::new(80.0, 30.0),
        Point2D::new(120.0, 60.0),
        Point2D::new(60.0, 80.0),
        Point2D::new(20.0, 60.0),
        Point2D::new(40.0, 40.0),
    ];

    let optimized_collider = OptimizedCollider::new(&complex_shape);

    // Create entity with optimized collider
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.7, 0.9),
                custom_size: Some(Vec2::new(120.0, 80.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        create_rapier_collider(&optimized_collider),
        optimized_collider,
    ));

    // Create simple shapes for comparison
    create_comparison_shapes(commands);
}

fn create_rapier_collider(optimized: &OptimizedCollider) -> Collider {
    // Create convex hull from the triangulated mesh for better performance
    let mut points = Vec::new();
    for triangle in &optimized.triangles {
        points.push(triangle[0]);
        points.push(triangle[1]);
        points.push(triangle[2]);
    }

    if points.is_empty() {
        // Fallback to AABB
        let size = optimized.aabb.size();
        Collider::cuboid(size.x / 2.0, size.y / 2.0)
    } else {
        // Create convex hull from all triangle vertices
        Collider::convex_hull(&points).unwrap_or_else(|| {
            let size = optimized.aabb.size();
            Collider::cuboid(size.x / 2.0, size.y / 2.0)
        })
    }
}

fn create_comparison_shapes(mut commands: Commands) {
    // AABB collider (before optimization)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.9, 0.5, 0.5),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            transform: Transform::from_xyz(-200.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(50.0, 50.0),
        Text2dBundle {
            text: Text::from_section(
                "AABB\n(simpler)",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(0.0, 60.0, 1.0),
            ..default()
        },
    ));

    // Optimized collider (after optimization)
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.9, 0.5),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(50.0, 50.0), // Simplified for demo
        Text2dBundle {
            text: Text::from_section(
                "Optimized\n(ear clipping)",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(0.0, 60.0, 1.0),
            ..default()
        },
    ));
}

fn update_performance_stats(
    mut stats: ResMut<PerformanceStats>,
    query: Query<&OptimizedCollider>,
) {
    stats.frame_count += 1;

    let total_triangles: u32 = query.iter().map(|c| c.triangles.len() as u32).sum();
    stats.total_triangles += total_triangles;

    stats.average_triangles_per_frame = stats.total_triangles as f32 / stats.frame_count as f32;
}

fn demonstrate_triangulation(
    mut commands: Commands,
    query: Query<(Entity, &OptimizedCollider)>,
    time: Res<Time>,
) {
    // Periodically update triangulation for dynamic shapes
    static mut LAST_UPDATE: f64 = 0.0;
    unsafe {
        if time.elapsed_seconds_f64() - LAST_UPDATE > 2.0 {
            LAST_UPDATE = time.elapsed_seconds_f64();

            for (entity, collider) in query.iter() {
                // Update collider if shape changed
                // In a real implementation, this would check for shape modifications
            }
        }
    }
}

fn render_triangulation_debug(
    mut commands: Commands,
    query: Query<(Entity, &OptimizedCollider, &Transform)>,
    time: Res<Time>,
) {
    // Clear previous debug visuals
    commands.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)));

    for (entity, collider, transform) in query.iter() {
        // Draw triangulation triangles for debugging
        for triangle in &collider.triangles {
            // Create debug lines for triangles
            create_triangle_debug_lines(&mut commands, triangle, transform, time.elapsed_seconds());
        }

        // Draw AABB
        create_aabb_debug_lines(&mut commands, &collider.aabb, transform);
    }
}

fn create_triangle_debug_lines(
    commands: &mut Commands,
    triangle: &[Vec2; 3],
    transform: &Transform,
    time: f32,
) {
    let color = Color::hsl(time * 50.0 % 360.0, 1.0, 0.5);

    // Draw triangle edges
    for i in 0..3 {
        let start = transform.transform_point((triangle[i] - Vec2::new(50.0, 40.0)).extend(0.0));
        let end = transform.transform_point((triangle[(i + 1) % 3] - Vec2::new(50.0, 40.0)).extend(0.0));

        // In a real implementation, you'd use a line drawing system
        // For now, we'll just log the triangle info
        println!("Triangle edge: ({:.1}, {:.1}) -> ({:.1}, {:.1})",
                start.x, start.y, end.x, end.y);
    }
}

fn create_aabb_debug_lines(
    commands: &mut Commands,
    aabb: &Rect,
    transform: &Transform,
) {
    let center = transform.transform_point(Vec2::ZERO.extend(0.0));
    let size = aabb.size();

    println!("AABB: center=({:.1}, {:.1}), size=({:.1}, {:.1})",
            center.x, center.y, size.x, size.y);
}

// Performance comparison display
fn display_performance_info(
    stats: Res<PerformanceStats>,
    mut commands: Commands,
) {
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            format!(
                "Physics Bridge Optimization Demo\n\
                 Average triangles per frame: {:.1}\n\
                 Frame: {}",
                stats.average_triangles_per_frame,
                stats.frame_count
            ),
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        transform: Transform::from_xyz(-500.0, 300.0, 10.0),
        ..default()
    });
}
