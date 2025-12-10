// Example: Designing a Magic System
// Based on "Designing a Magic System" blog post
// https://www.slowrush.dev/news/designing-a-magic-system

use bevy::prelude::*;
use std::collections::HashMap;

// Magic system design focused on spell creation and enemy targeting
// Demonstrates how magic integrates with gameplay and provides goals

#[derive(Clone, Debug)]
enum Element {
    Fire,
    Water,
    Earth,
    Air,
    Lightning,
    Ice,
    Poison,
    Healing,
}

#[derive(Clone, Debug)]
enum SpellShape {
    Projectile,
    Area,
    Beam,
    Summon,
    Shield,
    Teleport,
}

#[derive(Clone, Debug)]
struct Spell {
    name: String,
    elements: Vec<Element>,
    shape: SpellShape,
    damage: f32,
    mana_cost: f32,
    cooldown: f32,
    range: f32,
}

#[derive(Clone, Debug)]
struct Enemy {
    position: Vec2,
    health: f32,
    max_health: f32,
    element_weakness: Option<Element>,
    velocity: Vec2,
    size: f32,
}

#[derive(Clone, Debug)]
struct Player {
    position: Vec2,
    mana: f32,
    max_mana: f32,
    spells: Vec<Spell>,
    selected_spell: usize,
}

#[derive(Clone, Debug)]
struct ActiveSpell {
    spell: Spell,
    position: Vec2,
    velocity: Vec2,
    caster: Entity,
    lifetime: f32,
    target: Option<Entity>,
}

struct GameWorld {
    player: Player,
    enemies: Vec<Enemy>,
    active_spells: Vec<ActiveSpell>,
    width: f32,
    height: f32,
}

impl GameWorld {
    fn new() -> Self {
        let mut player = Player {
            position: Vec2::new(400.0, 300.0),
            mana: 100.0,
            max_mana: 100.0,
            selected_spell: 0,
            spells: Vec::new(),
        };

        // Create some basic spells
        player.spells.push(Spell {
            name: "Fireball".to_string(),
            elements: vec![Element::Fire],
            shape: SpellShape::Projectile,
            damage: 25.0,
            mana_cost: 15.0,
            cooldown: 0.5,
            range: 300.0,
        });

        player.spells.push(Spell {
            name: "Ice Blast".to_string(),
            elements: vec![Element::Ice],
            shape: SpellShape::Area,
            damage: 15.0,
            mana_cost: 20.0,
            cooldown: 1.0,
            range: 150.0,
        });

        player.spells.push(Spell {
            name: "Lightning Chain".to_string(),
            elements: vec![Element::Lightning],
            shape: SpellShape::Beam,
            damage: 30.0,
            mana_cost: 25.0,
            cooldown: 2.0,
            range: 200.0,
        });

        player.spells.push(Spell {
            name: "Earth Wall".to_string(),
            elements: vec![Element::Earth],
            shape: SpellShape::Shield,
            damage: 0.0,
            mana_cost: 30.0,
            cooldown: 3.0,
            range: 50.0,
        });

        Self {
            player,
            enemies: Vec::new(),
            active_spells: Vec::new(),
            width: 800.0,
            height: 600.0,
        }
    }

    fn spawn_enemy(&mut self, position: Vec2) {
        let elements = vec![Element::Fire, Element::Water, Element::Earth, Element::Air];
        let weakness = elements[rand::random::<usize>() % elements.len()];

        self.enemies.push(Enemy {
            position,
            health: 50.0,
            max_health: 50.0,
            element_weakness: Some(weakness),
            velocity: Vec2::ZERO,
            size: 20.0,
        });
    }

    fn cast_spell(&mut self, target_pos: Vec2) {
        if let Some(spell) = self.player.spells.get(self.player.selected_spell).cloned() {
            if self.player.mana >= spell.mana_cost {
                let direction = (target_pos - self.player.position).normalize();
                let speed = match spell.shape {
                    SpellShape::Projectile => 200.0,
                    SpellShape::Beam => 400.0,
                    _ => 0.0,
                };

                self.active_spells.push(ActiveSpell {
                    spell,
                    position: self.player.position,
                    velocity: direction * speed,
                    caster: Entity::PLACEHOLDER,
                    lifetime: 2.0,
                    target: None,
                });

                self.player.mana -= spell.mana_cost;
            }
        }
    }

    fn update(&mut self, dt: f32) {
        // Update active spells
        self.active_spells.retain_mut(|spell| {
            spell.lifetime -= dt;

            match spell.spell.shape {
                SpellShape::Projectile => {
                    spell.position += spell.velocity * dt;
                }
                SpellShape::Beam => {
                    // Beam spells move instantly to target
                    spell.lifetime = 0.0;
                }
                SpellShape::Area => {
                    // Area spells expand and then disappear
                    spell.lifetime = 0.0;
                }
                _ => {}
            }

            spell.lifetime > 0.0
        });

        // Check spell collisions with enemies
        let mut spells_to_remove = Vec::new();
        let mut damage_events = Vec::new();

        for (spell_idx, spell) in self.active_spells.iter().enumerate() {
            for (enemy_idx, enemy) in self.enemies.iter_mut().enumerate() {
                let distance = spell.position.distance(enemy.position);
                let hit = match spell.spell.shape {
                    SpellShape::Projectile => distance < 10.0,
                    SpellShape::Beam => distance < spell.spell.range,
                    SpellShape::Area => distance < spell.spell.range,
                    _ => false,
                };

                if hit {
                    let mut damage = spell.spell.damage;

                    // Check elemental weakness
                    if let Some(weakness) = enemy.element_weakness {
                        if spell.spell.elements.contains(&weakness) {
                            damage *= 2.0; // Critical hit!
                            println!("Critical hit! {} is weak to {:?}",
                                   enemy.element_weakness.unwrap() as usize, weakness);
                        }
                    }

                    damage_events.push((enemy_idx, damage));
                    spells_to_remove.push(spell_idx);
                    break;
                }
            }
        }

        // Apply damage
        for (enemy_idx, damage) in damage_events {
            if let Some(enemy) = self.enemies.get_mut(enemy_idx) {
                enemy.health -= damage;
                if enemy.health <= 0.0 {
                    println!("Enemy defeated!");
                }
            }
        }

        // Remove used spells
        for &idx in spells_to_remove.iter().rev() {
            if idx < self.active_spells.len() {
                self.active_spells.remove(idx);
            }
        }

        // Remove dead enemies
        self.enemies.retain(|enemy| enemy.health > 0.0);

        // Regenerate mana
        self.player.mana = (self.player.mana + 20.0 * dt).min(self.player.max_mana);

        // Spawn new enemies occasionally
        if rand::random::<f32>() < dt * 0.5 {
            let x = rand::random::<f32>() * self.width;
            let y = rand::random::<f32>() * self.height;
            self.spawn_enemy(Vec2::new(x, y));
        }
    }
}

#[derive(Resource)]
struct MagicWorldResource(GameWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Designing a Magic System - Spell Creation & Combat".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(MagicWorldResource(GameWorld::new()))
        .add_systems(Startup, setup_magic_demo)
        .add_systems(Update, (
            update_magic_world,
            render_magic_world,
            handle_magic_input,
            demonstrate_magic_system,
        ).chain())
        .run();
}

fn setup_magic_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_magic_world(mut world: ResMut<MagicWorldResource>, time: Res<Time>) {
    let dt = time.delta_seconds();
    world.0.update(dt);
}

fn render_magic_world(
    mut commands: Commands,
    mut player_entity: Local<Option<Entity>>,
    mut enemy_entities: Local<Vec<Entity>>,
    mut spell_entities: Local<Vec<Entity>>,
    world: Res<MagicWorldResource>,
) {
    // Clear previous frame
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }
    for entity in enemy_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in spell_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render player
    let player_entity_id = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_xyz(world.0.player.position.x, world.0.player.position.y, 1.0),
            ..default()
        },
        Text2dBundle {
            text: Text::from_section(
                format!("Mana: {:.0}/{:.0}", world.0.player.mana, world.0.player.max_mana),
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(0.0, 30.0, 2.0),
            ..default()
        },
    )).id();
    *player_entity = Some(player_entity_id);

    // Render enemies
    for enemy in &world.0.enemies {
        let health_percent = enemy.health / enemy.max_health;
        let color = if health_percent > 0.5 {
            Color::rgb(0.8, 0.2, 0.2)
        } else {
            Color::rgb(1.0, 0.2, 0.2)
        };

        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(enemy.size, enemy.size)),
                ..default()
            },
            transform: Transform::from_xyz(enemy.position.x, enemy.position.y, 1.0),
            ..default()
        }).id();
        enemy_entities.push(entity);
    }

    // Render active spells
    for spell in &world.0.active_spells {
        let color = match spell.spell.elements.first() {
            Some(Element::Fire) => Color::rgb(1.0, 0.3, 0.0),
            Some(Element::Ice) => Color::rgb(0.3, 0.8, 1.0),
            Some(Element::Lightning) => Color::rgb(1.0, 1.0, 0.0),
            Some(Element::Earth) => Color::rgb(0.5, 0.3, 0.1),
            _ => Color::rgb(0.8, 0.8, 0.8),
        };

        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            transform: Transform::from_xyz(spell.position.x, spell.position.y, 1.0),
            ..default()
        }).id();
        spell_entities.push(entity);
    }
}

fn handle_magic_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<MagicWorldResource>,
) {
    // Change selected spell
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        world.0.player.selected_spell = 0;
        println!("Selected: Fireball");
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        world.0.player.selected_spell = 1;
        println!("Selected: Ice Blast");
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        world.0.player.selected_spell = 2;
        println!("Selected: Lightning Chain");
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        world.0.player.selected_spell = 3;
        println!("Selected: Earth Wall");
    }

    // Cast spell on mouse click
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Ok(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        world.0.cast_spell(world_pos.origin.truncate());
                    }
                }
            }
        }
    }
}

fn demonstrate_magic_system(keyboard_input: Res<ButtonInput<KeyCode>>, world: Res<MagicWorldResource>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Magic System Demo ===");
        println!("Current spell: {}", world.0.player.spells[world.0.player.selected_spell].name);
        println!("Enemies alive: {}", world.0.enemies.len());
        println!("Active spells: {}", world.0.active_spells.len());
        println!("Mana: {:.0}/{:.0}", world.0.player.mana, world.0.player.max_mana);
        println!("\nControls:");
        println!("1-4: Select spells");
        println!("Left click: Cast spell");
        println!("H: Show this help");
        println!("====================\n");
    }
}
