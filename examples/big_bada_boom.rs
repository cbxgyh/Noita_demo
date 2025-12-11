// Example: Big Bada Boom
// Based on "Big Bada Boom" blog post
// https://www.slowrush.dev/news/big-bada-boom

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Explosion system with physics-based destruction
// Demonstrates explosive forces and area damage

#[derive(Clone, Debug)]
enum DebrisType {
    Stone,
    Metal,
    Wood,
    Glass,
}

impl DebrisType {
    fn color(&self) -> Color {
        match self {
            DebrisType::Stone => Color::srgb(0.5, 0.5, 0.5),
            DebrisType::Metal => Color::srgb(0.7, 0.7, 0.8),
            DebrisType::Wood => Color::srgb(0.6, 0.4, 0.2),
            DebrisType::Glass => Color::srgba(0.8, 0.9, 1.0, 0.7),
        }
    }

    fn mass(&self) -> f32 {
        match self {
            DebrisType::Stone => 2.0,
            DebrisType::Metal => 3.0,
            DebrisType::Wood => 1.5,
            DebrisType::Glass => 1.0,
        }
    }

    fn durability(&self) -> f32 {
        match self {
            DebrisType::Stone => 100.0,
            DebrisType::Metal => 80.0,
            DebrisType::Wood => 40.0,
            DebrisType::Glass => 20.0,
        }
    }
}

#[derive(Clone, Debug)]
struct Debris {
    position: Vec2,
    velocity: Vec2,
    debris_type: DebrisType,
    size: f32,
    health: f32,
    lifetime: Option<f32>,
}

impl Debris {
    fn new(debris_type: DebrisType, position: Vec2, size: f32) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            debris_type: debris_type.clone(),
            size,
            health: debris_type.durability(),
            lifetime: None,
        }
    }

    fn apply_force(&mut self, force: Vec2) {
        self.velocity += force / self.debris_type.mass();
    }

    fn update(&mut self, dt: f32) -> bool {
        self.position += self.velocity * dt;
        self.velocity.y -= 30.0 * dt; // Gravity
        self.velocity *= 0.99; // Air resistance

        if let Some(ref mut lifetime) = self.lifetime {
            *lifetime -= dt;
            if *lifetime <= 0.0 {
                return false;
            }
        }

        true
    }

    fn take_damage(&mut self, damage: f32) -> bool {
        self.health -= damage;
        self.health <= 0.0
    }
}

#[derive(Clone, Debug)]
struct Explosion {
    position: Vec2,
    radius: f32,
    damage: f32,
    force: f32,
    lifetime: f32,
    max_lifetime: f32,
}

impl Explosion {
    fn new(position: Vec2, radius: f32, damage: f32, force: f32) -> Self {
        Self {
            position,
            radius,
            damage,
            force,
            lifetime: 0.0,
            max_lifetime: 0.5, // Explosion lasts 0.5 seconds
        }
    }

    fn update(&mut self, dt: f32) -> bool {
    self.lifetime += dt;
        self.lifetime < self.max_lifetime
    }

    fn is_active(&self) -> bool {
        self.lifetime < self.max_lifetime
    }

    fn get_force_at(&self, point: Vec2) -> Vec2 {
        let distance = self.position.distance(point);
        if distance > self.radius || distance < 0.1 {
            return Vec2::ZERO;
        }

        let direction = (point - self.position).normalize();
        let force_magnitude = self.force * (1.0 - distance / self.radius);
        direction * force_magnitude
    }

    fn get_damage_at(&self, point: Vec2) -> f32 {
        let distance = self.position.distance(point);
        if distance > self.radius {
            return 0.0;
        }

        self.damage * (1.0 - distance / self.radius)
    }
}

#[derive(Clone, Debug)]
struct DestructibleObject {
    position: Vec2,
    size: Vec2,
    debris_type: DebrisType,
    health: f32,
    max_health: f32,
}

impl DestructibleObject {
    fn new(debris_type: DebrisType, position: Vec2, size: Vec2) -> Self {
        let health = debris_type.durability() * size.x * size.y;
        Self {
            position,
            size,
            debris_type,
            health,
            max_health: health,
        }
    }

    fn take_damage(&mut self, damage: f32) -> bool {
        self.health -= damage;
        self.health <= 0.0
    }

    fn create_debris(&self) -> Vec<Debris> {
        let mut debris = Vec::new();
        let debris_count = ((self.size.x * self.size.y) / 10.0).max(3.0) as usize;

        for _ in 0..debris_count {
            let offset = Vec2::new(
                (rand::random::<f32>() - 0.5) * self.size.x,
                (rand::random::<f32>() - 0.5) * self.size.y,
            );
            let pos = self.position + offset;

            let size = (self.size.x.min(self.size.y) * 0.2).max(2.0);
            let mut debris_piece = Debris::new(self.debris_type.clone(), pos, size);

            // Add some initial velocity
            debris_piece.velocity = Vec2::new(
                (rand::random::<f32>() - 0.5) * 100.0,
                rand::random::<f32>() * 50.0 + 20.0,
            );

            debris.push(debris_piece);
        }

        debris
    }
}

struct DestructionWorld {
    destructible_objects: Vec<DestructibleObject>,
    debris: Vec<Debris>,
    explosions: Vec<Explosion>,
    width: f32,
    height: f32,
}

impl DestructionWorld {
    fn new(width: f32, height: f32) -> Self {
        let mut destructible_objects = Vec::new();

        // Create buildings
        for i in 0..5 {
            let x = -200.0 + i as f32 * 100.0;
            let height = 50.0 + rand::random::<f32>() * 50.0;
            let debris_type = match rand::random::<usize>() % 4 {
                0 => DebrisType::Stone,
                1 => DebrisType::Metal,
                2 => DebrisType::Wood,
                _ => DebrisType::Glass,
            };

            destructible_objects.push(DestructibleObject::new(
                debris_type,
                Vec2::new(x, height / 2.0),
                Vec2::new(20.0, height),
            ));
        }

        // Create ground
        destructible_objects.push(DestructibleObject::new(
            DebrisType::Stone,
            Vec2::new(0.0, -10.0),
            Vec2::new(width, 20.0),
        ));

        Self {
            destructible_objects,
            debris: Vec::new(),
            explosions: Vec::new(),
            width,
            height,
        }
    }

    fn update(&mut self, dt: f32) {
        // Update explosions
        self.explosions.retain_mut(|exp| exp.update(dt));

        // Apply explosion forces to objects and debris
        for explosion in &self.explosions {
            if explosion.is_active() {
                // Apply forces to destructible objects
                for obj in &mut self.destructible_objects {
                    let force = explosion.get_force_at(obj.position);
                    // Objects don't move in this simplified demo, but force could be used for physics
                }

                // Apply forces to debris
                for debris in &mut self.debris {
                    let force = explosion.get_force_at(debris.position);
                    debris.apply_force(force);
                }

                // Damage nearby objects
                let mut destroyed_objects = Vec::new();
                for (i, obj) in self.destructible_objects.iter_mut().enumerate() {
                    let damage = explosion.get_damage_at(obj.position);
                    if damage > 0.0 && obj.take_damage(damage) {
                        destroyed_objects.push(i);
                    }
                }

                // Create debris from destroyed objects
                for &obj_idx in destroyed_objects.iter().rev() {
                    let obj = self.destructible_objects.remove(obj_idx);
                    let mut new_debris = obj.create_debris();
                    self.debris.append(&mut new_debris);
                }
            }
        }

        // Update debris
        self.debris.retain_mut(|d| d.update(dt));

        // Clean up expired explosions
        self.explosions.retain(|exp| exp.is_active());
    }

    fn add_explosion(&mut self, position: Vec2, size: ExplosionSize) {
        let (radius, damage, force) = match size {
            ExplosionSize::Small => (30.0, 50.0, 500.0),
            ExplosionSize::Medium => (60.0, 100.0, 1000.0),
            ExplosionSize::Large => (100.0, 200.0, 2000.0),
        };

        self.explosions.push(Explosion::new(position, radius, damage, force));
    }

    fn add_projectile_explosion(&mut self, position: Vec2) {
        self.add_explosion(position, ExplosionSize::Small);
    }
}

#[derive(Clone, Debug)]
enum ExplosionSize {
    Small,
    Medium,
    Large,
}

#[derive(Resource)]
struct DestructionWorldResource(DestructionWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Big Bada Boom - Explosive Destruction".to_string(),
                resolution: (1000.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(DestructionWorldResource(DestructionWorld::new(1000.0, 800.0)))
        .add_systems(Startup, setup_destruction_demo)
        .add_systems(Update, (
            update_destruction_world,
            render_destruction_world,
            handle_destruction_input,
            demonstrate_explosions,
        ).chain())
        .run();
}

fn setup_destruction_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_destruction_world(mut world: ResMut<DestructionWorldResource>, time: Res<Time>) {
    let dt = time.delta_seconds().min(1.0 / 30.0);
    world.0.update(dt);
}

fn render_destruction_world(
    mut commands: Commands,
    mut object_entities: Local<Vec<Entity>>,
    mut debris_entities: Local<Vec<Entity>>,
    mut explosion_entities: Local<Vec<Entity>>,
    world: Res<DestructionWorldResource>,
) {
    // Clear previous frame
    for entity in object_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in debris_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in explosion_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render destructible objects
    for obj in &world.0.destructible_objects {
        let health_percent = obj.health / obj.max_health;
        let base_color = obj.debris_type.color();
        let [r, g, b, a] = base_color.to_srgba().to_f32_array();
        let factor = health_percent.max(0.3);
        let color = Color::srgba(r * factor, g * factor, b * factor, a);
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(obj.size),
                ..default()
            },
            transform: Transform::from_xyz(obj.position.x, obj.position.y, 0.0),
            ..default()
        }).id();
        object_entities.push(entity);
    }

    // Render debris
    for debris in &world.0.debris {
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: debris.debris_type.color(),
                custom_size: Some(Vec2::new(debris.size, debris.size)),
                ..default()
            },
            transform: Transform::from_xyz(debris.position.x, debris.position.y, 1.0),
            ..default()
        }).id();
        debris_entities.push(entity);
    }

    // Render explosions
    for explosion in &world.0.explosions {
        if explosion.is_active() {
            let alpha = 1.0 - (explosion.lifetime / explosion.max_lifetime);
            let color = Color::rgba(1.0, 0.5, 0.0, alpha);

            let entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(explosion.radius * 2.0, explosion.radius * 2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(explosion.position.x, explosion.position.y, 2.0),
                ..default()
            }).id();
            explosion_entities.push(entity);
        }
    }
}

fn handle_destruction_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<DestructionWorldResource>,
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    let atom_x = world_pos.origin.x;
                    let atom_y = world_pos.origin.y;

                    // Different explosion sizes
                    if mouse_input.just_pressed(MouseButton::Left) {
                        world.0.add_explosion(Vec2::new(atom_x, atom_y), ExplosionSize::Small);
                    }

                    if mouse_input.just_pressed(MouseButton::Right) {
                        world.0.add_explosion(Vec2::new(atom_x, atom_y), ExplosionSize::Medium);
                    }

                    if mouse_input.just_pressed(MouseButton::Middle) {
                        world.0.add_explosion(Vec2::new(atom_x, atom_y), ExplosionSize::Large);
                    }
                }
            }
        }
    }

    // Reset demo
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        *world = DestructionWorldResource(DestructionWorld::new(1000.0, 800.0));
    }
}

fn demonstrate_explosions(keyboard_input: Res<ButtonInput<KeyCode>>, world: Res<DestructionWorldResource>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Big Bada Boom Destruction Demo ===");
        println!("Physics-based destruction with explosions!");
        println!("");
        println!("Active Explosions: {}", world.0.explosions.len());
        println!("Destructible Objects: {}", world.0.destructible_objects.len());
        println!("Flying Debris: {}", world.0.debris.len());
        println!("");
        println!("Explosion Physics:");
        println!("- Radial force application");
        println!("- Damage falloff with distance");
        println!("- Objects break into debris");
        println!("- Debris affected by gravity");
        println!("");
        println!("Controls:");
        println!("Left click: Small explosion");
        println!("Right click: Medium explosion");
        println!("Middle click: Large explosion");
        println!("R: Reset demo");
        println!("H: Show this info");
        println!("=========================\n");
    }
}
