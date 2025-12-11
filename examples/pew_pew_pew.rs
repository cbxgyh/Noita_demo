// Example: Pew Pew Pew
// Based on "Pew Pew Pew" blog post
// https://www.slowrush.dev/news/pew-pew-pew

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Shooting system with projectiles and weapons
// Demonstrates basic shooting mechanics

#[derive(Clone, Debug)]
struct Projectile {
    position: Vec2,
    velocity: Vec2,
    damage: f32,
    lifetime: f32,
    max_lifetime: f32,
    owner: Entity,
}

impl Projectile {
    fn new(position: Vec2, velocity: Vec2, damage: f32, owner: Entity) -> Self {
        Self {
            position,
            velocity,
            damage,
            lifetime: 0.0,
            max_lifetime: 3.0,
            owner,
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        self.lifetime += dt;
        self.position += self.velocity * dt;

        // Simple bounds checking (would be removed in real game)
        if self.position.x < -500.0 || self.position.x > 500.0 ||
           self.position.y < -500.0 || self.position.y > 500.0 {
            return false;
        }

        self.lifetime < self.max_lifetime
    }
}

#[derive(Clone, Debug)]
enum WeaponType {
    Pistol,
    Shotgun,
    MachineGun,
    Sniper,
    RocketLauncher,
}

impl WeaponType {
    fn fire_rate(&self) -> f32 {
        match self {
            WeaponType::Pistol => 0.5,
            WeaponType::Shotgun => 1.0,
            WeaponType::MachineGun => 0.1,
            WeaponType::Sniper => 2.0,
            WeaponType::RocketLauncher => 1.5,
        }
    }

    fn damage(&self) -> f32 {
        match self {
            WeaponType::Pistol => 25.0,
            WeaponType::Shotgun => 15.0,
            WeaponType::MachineGun => 12.0,
            WeaponType::Sniper => 80.0,
            WeaponType::RocketLauncher => 100.0,
        }
    }

    fn projectile_count(&self) -> usize {
        match self {
            WeaponType::Pistol => 1,
            WeaponType::Shotgun => 8,
            WeaponType::MachineGun => 1,
            WeaponType::Sniper => 1,
            WeaponType::RocketLauncher => 1,
        }
    }

    fn spread_angle(&self) -> f32 {
        match self {
            WeaponType::Pistol => 0.1,
            WeaponType::Shotgun => std::f32::consts::PI / 6.0, // 30 degrees
            WeaponType::MachineGun => 0.05,
            WeaponType::Sniper => 0.0,
            WeaponType::RocketLauncher => 0.0,
        }
    }

    fn projectile_speed(&self) -> f32 {
        match self {
            WeaponType::Pistol => 300.0,
            WeaponType::Shotgun => 250.0,
            WeaponType::MachineGun => 400.0,
            WeaponType::Sniper => 600.0,
            WeaponType::RocketLauncher => 200.0,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            WeaponType::Pistol => "Pistol",
            WeaponType::Shotgun => "Shotgun",
            WeaponType::MachineGun => "Machine Gun",
            WeaponType::Sniper => "Sniper Rifle",
            WeaponType::RocketLauncher => "Rocket Launcher",
        }
    }
}

#[derive(Clone, Debug)]
struct Weapon {
    weapon_type: WeaponType,
    ammo: u32,
    max_ammo: u32,
    last_fired: f64,
}

impl Weapon {
    fn new(weapon_type: WeaponType) -> Self {
        let max_ammo = match weapon_type {
            WeaponType::Pistol => 12,
            WeaponType::Shotgun => 8,
            WeaponType::MachineGun => 30,
            WeaponType::Sniper => 5,
            WeaponType::RocketLauncher => 4,
        };

        Self {
            weapon_type,
            ammo: max_ammo,
            max_ammo,
            last_fired: 0.0,
        }
    }

    fn can_fire(&self, current_time: f64) -> bool {
        self.ammo > 0 && current_time - self.last_fired >= self.weapon_type.fire_rate() as f64
    }

    fn fire(&mut self, position: Vec2, direction: Vec2, current_time: f64) -> Vec<Projectile> {
        if !self.can_fire(current_time) {
            return Vec::new();
        }

        self.last_fired = current_time;
        self.ammo -= 1;

        let mut projectiles = Vec::new();
        let base_direction = direction.normalize();
        let projectile_count = self.weapon_type.projectile_count();
        let spread_angle = self.weapon_type.spread_angle();
        let damage = self.weapon_type.damage();
        let speed = self.weapon_type.projectile_speed();

        for i in 0..projectile_count {
            let angle_offset = if projectile_count == 1 {
                0.0
            } else {
                spread_angle * (i as f32 / (projectile_count - 1) as f32 - 0.5) * 2.0
            };

            let direction = Vec2::new(
                base_direction.x * angle_offset.cos() - base_direction.y * angle_offset.sin(),
                base_direction.x * angle_offset.sin() + base_direction.y * angle_offset.cos(),
            );

            let velocity = direction * speed;
            let projectile = Projectile::new(position, velocity, damage, Entity::PLACEHOLDER);

            projectiles.push(projectile);
        }

        projectiles
    }
}

#[derive(Clone, Debug)]
struct Player {
    position: Vec2,
    weapons: Vec<Weapon>,
    current_weapon: usize,
    health: f32,
    max_health: f32,
}

impl Player {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            weapons: vec![
                Weapon::new(WeaponType::Pistol),
                Weapon::new(WeaponType::Shotgun),
                Weapon::new(WeaponType::MachineGun),
            ],
            current_weapon: 0,
            health: 100.0,
            max_health: 100.0,
        }
    }

    fn current_weapon(&self) -> &Weapon {
        &self.weapons[self.current_weapon]
    }

    fn current_weapon_mut(&mut self) -> &mut Weapon {
        &mut self.weapons[self.current_weapon]
    }

    fn switch_weapon(&mut self, index: usize) {
        if index < self.weapons.len() {
            self.current_weapon = index;
        }
    }

    fn reload_current_weapon(&mut self) {
        let weapon = &mut self.weapons[self.current_weapon];
        weapon.ammo = weapon.max_ammo;
    }
}

#[derive(Clone, Debug)]
struct Target {
    position: Vec2,
    health: f32,
    max_health: f32,
    size: f32,
}

impl Target {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            health: 50.0,
            max_health: 50.0,
            size: 20.0,
        }
    }

    fn take_damage(&mut self, damage: f32) -> bool {
        self.health -= damage;
        self.health <= 0.0
    }
}

struct ShootingGame {
    player: Player,
    targets: Vec<Target>,
    projectiles: Vec<Projectile>,
    width: f32,
    height: f32,
}

impl ShootingGame {
    fn new() -> Self {
        let player = Player::new(Vec2::new(0.0, -200.0));

        let mut targets = Vec::new();
        for i in 0..5 {
            let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
            let distance = 150.0;
            let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);
            targets.push(Target::new(position));
        }

        Self {
            player,
            targets,
            projectiles: Vec::new(),
            width: 800.0,
            height: 600.0,
        }
    }

    fn update(&mut self, dt: f32, current_time: f64) {
        // Update projectiles
        self.projectiles.retain_mut(|proj| proj.update(dt));

        // Check projectile collisions with targets
        let mut hit_targets = Vec::new();
        let mut projectiles_to_remove = Vec::new();

        for (proj_idx, projectile) in self.projectiles.iter().enumerate() {
            for (target_idx, target) in self.targets.iter_mut().enumerate() {
                let distance = projectile.position.distance(target.position);
                if distance < target.size / 2.0 {
                    if target.take_damage(projectile.damage) {
                        hit_targets.push(target_idx);
                    }
                    projectiles_to_remove.push(proj_idx);
                    break;
                }
            }
        }

        // Remove dead targets
        for &target_idx in hit_targets.iter().rev() {
            self.targets.remove(target_idx);
        }

        // Remove used projectiles
        for &proj_idx in projectiles_to_remove.iter().rev() {
            if proj_idx < self.projectiles.len() {
                self.projectiles.remove(proj_idx);
            }
        }

        // Respawn targets if all are dead
        if self.targets.is_empty() {
            for i in 0..5 {
                let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
                let distance = 150.0 + rand::random::<f32>() * 50.0;
                let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);
                self.targets.push(Target::new(position));
            }
        }
    }

    fn player_shoot(&mut self, direction: Vec2, current_time: f64) {
        let position =self.player.position;
        let weapon = self.player.current_weapon_mut();
        let new_projectiles = weapon.fire(position, direction, current_time);
        self.projectiles.extend(new_projectiles);
    }
}

#[derive(Resource)]
struct ShootingGameResource(ShootingGame);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pew Pew Pew - Shooting System".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ShootingGameResource(ShootingGame::new()))
        .add_systems(Startup, setup_shooting_demo)
        .add_systems(Update, (
            update_shooting_game,
            render_shooting_game,
            handle_shooting_input,
            display_shooting_info,
        ).chain())
        .run();
}

fn setup_shooting_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_shooting_game(
    mut game: ResMut<ShootingGameResource>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let current_time = time.elapsed_seconds_f64();
    game.0.update(dt, current_time);
}

fn render_shooting_game(
    mut commands: Commands,
    mut player_entity: Local<Option<Entity>>,
    mut target_entities: Local<Vec<Entity>>,
    mut projectile_entities: Local<Vec<Entity>>,
    game: Res<ShootingGameResource>,
) {
    // Clear previous frame
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }
    for entity in target_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in projectile_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render player
    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(20.0, 30.0)),
                ..default()
            },
            transform: Transform::from_xyz(game.0.player.position.x, game.0.player.position.y, 1.0),
            ..default()
        },

    )).id();
    *player_entity = Some(entity);

    // Render targets
    for target in &game.0.targets {
        let health_percent = target.health / target.max_health;
        let color = Color::hsl(health_percent * 120.0, 1.0, 0.5); // Green to red based on health

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(target.size, target.size)),
                    ..default()
                },
                transform: Transform::from_xyz(target.position.x, target.position.y, 1.0),
                ..default()
            },

        )).id();
        target_entities.push(entity);
    }

    // Render projectiles
    for projectile in &game.0.projectiles {
        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 1.0, 0.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..default()
            },
            transform: Transform::from_xyz(projectile.position.x, projectile.position.y, 1.0),
            ..default()
        }).id();
        projectile_entities.push(entity);
    }
}

fn handle_shooting_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut game: ResMut<ShootingGameResource>,
    time: Res<Time>,
) {
    // Weapon switching
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        game.0.player.switch_weapon(0);
        println!("Switched to Pistol");
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        game.0.player.switch_weapon(1);
        println!("Switched to Shotgun");
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        game.0.player.switch_weapon(2);
        println!("Switched to Machine Gun");
    }

    // Reload
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        game.0.player.reload_current_weapon();
        println!("Reloaded!");
    }

    // Shooting
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        let target_pos = world_pos.origin.truncate();
                        let direction = (target_pos - game.0.player.position).normalize();
                        game.0.player_shoot(direction, time.elapsed_seconds_f64());
                    }
                }
            }
        }
    }
}

fn display_shooting_info(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    game: Res<ShootingGameResource>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        let weapon = game.0.player.current_weapon();

        println!("\n=== Pew Pew Pew Shooting Demo ===");
        println!("Current Weapon: {}", weapon.weapon_type.name());
        println!("Ammo: {}/{}", weapon.ammo, weapon.max_ammo);
        println!("Damage: {:.0}", weapon.weapon_type.damage());
        println!("Fire Rate: {:.1}/s", 1.0 / weapon.weapon_type.fire_rate());
        println!("Projectile Count: {}", weapon.weapon_type.projectile_count());
        println!("Targets Remaining: {}", game.0.targets.len());
        println!("Active Projectiles: {}", game.0.projectiles.len());
        println!("\nControls:");
        println!("1-3: Switch weapons");
        println!("Left click: Shoot");
        println!("R: Reload");
        println!("H: Show this info");
        println!("=============================\n");
    }
}
