// Example: Circular Raycasting
// Based on "Circular Raycasting" blog post
// https://www.slowrush.dev/news/circular-raycasting

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Circular raycasting for field of view and collision detection
// Demonstrates radial ray casting for visibility, AI, and collision systems

#[derive(Clone, Debug)]
struct CircularRaycaster {
    center: Vec2,
    radius: f32,
    ray_count: usize,
    angle_offset: f32,
    rays: Vec<RayResult>,
}

impl CircularRaycaster {
    fn new(center: Vec2, radius: f32, ray_count: usize) -> Self {
        Self {
            center,
            radius,
            ray_count,
            angle_offset: 0.0,
            rays: Vec::with_capacity(ray_count),
        }
    }

    fn update(&mut self, obstacles: &[Obstacle], dt: f32) {
        self.angle_offset += dt * 0.5; // Slow rotation
        self.cast_rays(obstacles);
    }

    fn cast_rays(&mut self, obstacles: &[Obstacle]) {
        self.rays.clear();

        for i in 0..self.ray_count {
            let angle = (i as f32 / self.ray_count as f32) * std::f32::consts::PI * 2.0 + self.angle_offset;
            let direction = Vec2::new(angle.cos(), angle.sin());

            let ray = Ray {
                origin: self.center,
                direction,
            };

            let result = self.cast_single_ray(ray, self.radius, obstacles);
            self.rays.push(result);
        }
    }

    fn cast_single_ray(&self, ray: Ray, max_distance: f32, obstacles: &[Obstacle]) -> RayResult {
        let mut closest_hit: Option<RayHit> = None;
        let mut min_distance = max_distance;

        for obstacle in obstacles {
            if let Some(hit) = obstacle.ray_intersection(ray, max_distance) {
                if hit.distance < min_distance {
                    min_distance = hit.distance;
                    closest_hit = Some(hit);
                }
            }
        }

        RayResult {
            ray,
            hit: closest_hit,
            max_distance,
        }
    }

    fn get_visible_points(&self) -> Vec<Vec2> {
        self.rays.iter()
            .filter_map(|result| result.hit.as_ref().map(|hit| hit.point))
            .collect()
    }

    fn get_visibility_polygon(&self) -> Vec<Vec2> {
        let mut points = Vec::new();

        for result in &self.rays {
            if let Some(hit) = &result.hit {
                points.push(hit.point);
            } else {
                // Extend to max distance
                points.push(result.ray.origin + result.ray.direction * result.max_distance);
            }
        }

        points
    }

    fn is_point_visible(&self, point: Vec2) -> bool {
        let direction = (point - self.center).normalize();
        let angle = direction.y.atan2(direction.x);

        // Find the closest ray to this angle
        let normalized_angle = if angle < 0.0 { angle + std::f32::consts::PI * 2.0 } else { angle };
        let angle_step = std::f32::consts::PI * 2.0 / self.ray_count as f32;
        let ray_index = ((normalized_angle / angle_step) as usize) % self.ray_count;

        if let Some(ray_result) = self.rays.get(ray_index) {
            if let Some(hit) = &ray_result.hit {
                let distance_to_point = self.center.distance(point);
                distance_to_point <= hit.distance
            } else {
                true // No obstacle in this direction
            }
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
struct Ray {
    origin: Vec2,
    direction: Vec2,
}

#[derive(Clone, Debug)]
struct RayHit {
    point: Vec2,
    distance: f32,
    normal: Vec2,
    obstacle_id: u32,
}

#[derive(Clone, Debug)]
struct RayResult {
    ray: Ray,
    hit: Option<RayHit>,
    max_distance: f32,
}

#[derive(Clone, Debug)]
struct Obstacle {
    id: u32,
    shape: ObstacleShape,
}

#[derive(Clone, Debug)]
enum ObstacleShape {
    Circle { center: Vec2, radius: f32 },
    Rectangle { center: Vec2, size: Vec2 },
    Polygon { vertices: Vec<Vec2> },
}

impl Obstacle {
    fn new_circle(id: u32, center: Vec2, radius: f32) -> Self {
        Self {
            id,
            shape: ObstacleShape::Circle { center, radius },
        }
    }

    fn new_rectangle(id: u32, center: Vec2, size: Vec2) -> Self {
        Self {
            id,
            shape: ObstacleShape::Rectangle { center, size },
        }
    }

    fn ray_intersection(&self, ray: Ray, max_distance: f32) -> Option<RayHit> {
        match &self.shape {
            ObstacleShape::Circle { center, radius } => {
                self.circle_ray_intersection(ray, *center, *radius, max_distance)
            }
            ObstacleShape::Rectangle { center, size } => {
                self.rectangle_ray_intersection(ray, *center, *size, max_distance)
            }
            ObstacleShape::Polygon { vertices } => {
                self.polygon_ray_intersection(ray, vertices, max_distance)
            }
        }
    }

    fn circle_ray_intersection(&self, ray: Ray, center: Vec2, radius: f32, max_distance: f32) -> Option<RayHit> {
        // Ray-circle intersection
        let oc = ray.origin - center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

        let mut t = if t1 >= 0.0 { t1 } else { t2 };
        if t < 0.0 {
            return None;
        }

        let hit_point = ray.origin + ray.direction * t;
        let distance = ray.origin.distance(hit_point);

        if distance > max_distance {
            return None;
        }

        let normal = (hit_point - center).normalize();

        Some(RayHit {
            point: hit_point,
            distance,
            normal,
            obstacle_id: self.id,
        })
    }

    fn rectangle_ray_intersection(&self, ray: Ray, center: Vec2, size: Vec2, max_distance: f32) -> Option<RayHit> {
        // AABB ray intersection (simplified)
        let half_size = size / 2.0;
        let min = center - half_size;
        let max = center + half_size;

        let mut t_min = 0.0;
        let mut t_max = max_distance;

        for i in 0..2 {
            let origin = if i == 0 { ray.origin.x } else { ray.origin.y };
            let direction = if i == 0 { ray.direction.x } else { ray.direction.y };
            let min_bound = if i == 0 { min.x } else { min.y };
            let max_bound = if i == 0 { max.x } else { max.y };

            if direction.abs() < 0.001 {
                if origin < min_bound || origin > max_bound {
                    return None;
                }
            } else {
                let t1 = (min_bound - origin) / direction;
                let t2 = (max_bound - origin) / direction;

                let t_near = t1.min(t2);
                let t_far = t1.max(t2);

                t_min = t_min.max(t_near);
                t_max = t_max.min(t_far);

                if t_min > t_max {
                    return None;
                }
            }
        }

        if t_min > max_distance {
            return None;
        }

        let hit_point = ray.origin + ray.direction * t_min;
        let distance = t_min;

        // Calculate normal (simplified)
        let normal = if (hit_point.x - min.x).abs() < 0.1 {
            Vec2::new(-1.0, 0.0)
        } else if (hit_point.x - max.x).abs() < 0.1 {
            Vec2::new(1.0, 0.0)
        } else if (hit_point.y - min.y).abs() < 0.1 {
            Vec2::new(0.0, -1.0)
        } else {
            Vec2::new(0.0, 1.0)
        };

        Some(RayHit {
            point: hit_point,
            distance,
            normal,
            obstacle_id: self.id,
        })
    }

    fn polygon_ray_intersection(&self, ray: Ray, vertices: &[Vec2], max_distance: f32) -> Option<RayHit> {
        // Simplified polygon ray intersection using line segments
        let mut closest_hit: Option<RayHit> = None;
        let mut min_distance = max_distance;

        for i in 0..vertices.len() {
            let start = vertices[i];
            let end = vertices[(i + 1) % vertices.len()];

            if let Some(hit) = self.line_segment_ray_intersection(ray, start, end) {
                if hit.distance < min_distance {
                    min_distance = hit.distance;
                    closest_hit = Some(hit);
                }
            }
        }

        closest_hit
    }

    fn line_segment_ray_intersection(&self, ray: Ray, line_start: Vec2, line_end: Vec2) -> Option<RayHit> {
        let v1 = ray.origin - line_start;
        let v2 = line_end - line_start;
        let v3 = Vec2::new(-ray.direction.y, ray.direction.x);

        let dot = v2.dot(v3);
        if dot.abs() < 0.001 {
            return None; // Parallel
        }

        let t1 = v2.cross(v1) / dot;
        let t2 = v1.dot(v3) / dot;

        if t1 >= 0.0 && t2 >= 0.0 && t2 <= 1.0 {
            let hit_point = ray.origin + ray.direction * t1;
            let distance = t1;

            if distance > 0.0 {
                let normal = Vec2::new(-v2.y, v2.x).normalize();
                return Some(RayHit {
                    point: hit_point,
                    distance,
                    normal,
                    obstacle_id: self.id,
                });
            }
        }

        None
    }

    fn get_render_data(&self) -> (Vec2, Vec2, Color) {
        match &self.shape {
            ObstacleShape::Circle { center, radius } => {
                (*center, Vec2::new(*radius * 2.0, *radius * 2.0), Color::rgb(0.6, 0.6, 0.6))
            }
            ObstacleShape::Rectangle { center, size } => {
                (*center, *size, Color::rgb(0.5, 0.5, 0.5))
            }
            ObstacleShape::Polygon { vertices } => {
                // Use first vertex as center for rendering
                if let Some(center) = vertices.first() {
                    (*center, Vec2::new(20.0, 20.0), Color::rgb(0.4, 0.4, 0.4))
                } else {
                    (Vec2::ZERO, Vec2::ZERO, Color::BLACK)
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct CircularRaycastDemo {
    raycaster: CircularRaycaster,
    obstacles: Vec<Obstacle>,
    visible_targets: Vec<Vec2>,
    next_obstacle_id: u32,
}

impl CircularRaycastDemo {
    fn new() -> Self {
        let raycaster = CircularRaycaster::new(Vec2::new(400.0, 300.0), 150.0, 32);

        let mut obstacles = Vec::new();
        let mut next_id = 0;

        // Add some obstacles
        obstacles.push(Obstacle::new_circle(next_id, Vec2::new(300.0, 250.0), 30.0));
        next_id += 1;
        obstacles.push(Obstacle::new_rectangle(next_id, Vec2::new(500.0, 350.0), Vec2::new(60.0, 40.0)));
        next_id += 1;
        obstacles.push(Obstacle::new_circle(next_id, Vec2::new(450.0, 200.0), 25.0));
        next_id += 1;

        let visible_targets = vec![
            Vec2::new(350.0, 280.0),
            Vec2::new(480.0, 320.0),
            Vec2::new(420.0, 180.0),
            Vec2::new(520.0, 250.0),
        ];

        Self {
            raycaster,
            obstacles,
            visible_targets,
            next_obstacle_id: next_id,
        }
    }

    fn update(&mut self, dt: f32) {
        self.raycaster.update(&self.obstacles, dt);
    }

    fn add_obstacle(&mut self, position: Vec2, obstacle_type: u32) {
        let obstacle = match obstacle_type {
            0 => Obstacle::new_circle(self.next_obstacle_id, position, 20.0 + rand::random::<f32>() * 20.0),
            1 => Obstacle::new_rectangle(self.next_obstacle_id, position, Vec2::new(30.0 + rand::random::<f32>() * 30.0, 20.0 + rand::random::<f32>() * 20.0)),
            _ => Obstacle::new_circle(self.next_obstacle_id, position, 25.0),
        };

        self.obstacles.push(obstacle);
        self.next_obstacle_id += 1;
    }

    fn get_visible_targets(&self) -> Vec<(Vec2, bool)> {
        self.visible_targets.iter()
            .map(|&target| (target, self.raycaster.is_point_visible(target)))
            .collect()
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Circular Raycasting - Field of View & Collision Detection".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(CircularRaycastDemo::new())
        .add_systems(Startup, setup_circular_raycast_demo)
        .add_systems(Update, (
            handle_circular_raycast_input,
            update_circular_raycast_simulation,
            render_circular_raycast_demo,
            display_circular_raycast_info,
        ).chain())
        .run();
}

fn setup_circular_raycast_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_circular_raycast_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<CircularRaycastDemo>,
) {
    // Get mouse position
    let mouse_pos = if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    world_pos.origin.truncate()
                } else {
                    Vec2::ZERO
                }
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        }
    } else {
        Vec2::ZERO
    };

    // Move raycaster center
    if keyboard_input.pressed(KeyCode::KeyW) {
        demo.raycaster.center.y += 2.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        demo.raycaster.center.y -= 2.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        demo.raycaster.center.x -= 2.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        demo.raycaster.center.x += 2.0;
    }

    // Add obstacles
    if mouse_input.just_pressed(MouseButton::Left) {
        demo.add_obstacle(mouse_pos, 0); // Circle
    }
    if mouse_input.just_pressed(MouseButton::Right) {
        demo.add_obstacle(mouse_pos, 1); // Rectangle
    }

    // Adjust ray count
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        demo.raycaster.ray_count = (demo.raycaster.ray_count + 4).min(128);
        println!("Ray count: {}", demo.raycaster.ray_count);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        demo.raycaster.ray_count = (demo.raycaster.ray_count - 4).max(8);
        println!("Ray count: {}", demo.raycaster.ray_count);
    }

    // Adjust radius
    if keyboard_input.just_pressed(KeyCode::Equal) {
        demo.raycaster.radius += 10.0;
        println!("Radius: {:.0}", demo.raycaster.radius);
    }
    if keyboard_input.just_pressed(KeyCode::Minus) {
        demo.raycaster.radius = (demo.raycaster.radius - 10.0).max(20.0);
        println!("Radius: {:.0}", demo.raycaster.radius);
    }
}

fn update_circular_raycast_simulation(time: Res<Time>, mut demo: ResMut<CircularRaycastDemo>) {
    demo.update(time.delta_seconds().min(1.0 / 30.0));
}

fn render_circular_raycast_demo(
    mut commands: Commands,
    mut ray_entities: Local<Vec<Entity>>,
    mut obstacle_entities: Local<Vec<Entity>>,
    mut target_entities: Local<Vec<Entity>>,
    mut center_entity: Local<Option<Entity>>,
    demo: Res<CircularRaycastDemo>,
) {
    // Clear previous frame
    for entity in ray_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in obstacle_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in target_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = *center_entity {
        commands.entity(entity).despawn();
    }

    // Render raycaster center
    let center_entity_id = commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(10.0, 10.0)),
            ..default()
        },
        transform: Transform::from_xyz(demo.raycaster.center.x, demo.raycaster.center.y, 2.0),
        ..default()
    }).id();
    *center_entity = Some(center_entity_id);

    // Render rays
    for ray_result in &demo.raycaster.rays {
        let end_point = if let Some(hit) = &ray_result.hit {
            hit.point
        } else {
            ray_result.ray.origin + ray_result.ray.direction * ray_result.max_distance
        };

        let ray_entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.0, 1.0, 0.0, 0.3),
                custom_size: Some(Vec2::new(2.0, ray_result.ray.origin.distance(end_point))),
                ..default()
            },
            transform: Transform {
                translation: (ray_result.ray.origin + end_point) / 2.0 * Vec3::new(1.0, 1.0, 0.0),
                rotation: Quat::from_rotation_z(
                    (end_point - ray_result.ray.origin).angle_between(Vec2::Y)
                ),
                ..default()
            },
            ..default()
        }).id();
        ray_entities.push(ray_entity);
    }

    // Render obstacles
    for obstacle in &demo.obstacles {
        let (position, size, color) = obstacle.get_render_data();
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 1.0),
            ..default()
        }).id();
        obstacle_entities.push(entity);
    }

    // Render visible targets
    let visible_targets = demo.get_visible_targets();
    for (position, is_visible) in visible_targets {
        let color = if is_visible {
            Color::rgb(0.2, 0.8, 0.2)
        } else {
            Color::rgb(0.8, 0.2, 0.2)
        };

        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 1.0),
            ..default()
        }).id();
        target_entities.push(entity);
    }
}

fn display_circular_raycast_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<CircularRaycastDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        let visible_targets = demo.get_visible_targets();
        let visible_count = visible_targets.iter().filter(|(_, visible)| *visible).count();

        println!("\n=== Circular Raycasting Demo ===");
        println!("Raycaster Center: ({:.1}, {:.1})", demo.raycaster.center.x, demo.raycaster.center.y);
        println!("Ray Count: {}", demo.raycaster.ray_count);
        println!("View Radius: {:.1}", demo.raycaster.radius);
        println!("Obstacles: {}", demo.obstacles.len());
        println!("Visible Targets: {}/{}", visible_count, visible_targets.len());

        println!("\nControls:");
        println!("  WASD: Move raycaster center");
        println!("  Left Click: Add circle obstacle");
        println!("  Right Click: Add rectangle obstacle");
        println!("  ↑/↓: Adjust ray count");
        println!("  +/-: Adjust radius");
        println!("  H: Show this info");
        println!("\nGreen rays: unobstructed");
        println!("Green dots: visible targets");
        println!("Red dots: hidden targets");
        println!("======================\n");
    }
}
