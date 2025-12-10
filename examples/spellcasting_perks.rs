// Example: Spellcasting 3.0: Perks
// Based on "Spellcasting 3.0: Perks" blog post
// https://www.slowrush.dev/news/spellcasting-3.0-perks

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashMap;

// Perk-based spell system
// Demonstrates modular spell construction using perks

#[derive(Clone, Debug, PartialEq)]
enum PerkType {
    Projectile,
    Homing,
    Explosive,
    Chain,
    Freeze,
    Burn,
    Poison,
    Shield,
    Teleport,
    Summon,
    Lightning,
    Gravity,
}

impl PerkType {
    fn name(&self) -> &'static str {
        match self {
            PerkType::Projectile => "Projectile",
            PerkType::Homing => "Homing",
            PerkType::Explosive => "Explosive",
            PerkType::Chain => "Chain",
            PerkType::Freeze => "Freeze",
            PerkType::Burn => "Burn",
            PerkType::Poison => "Poison",
            PerkType::Shield => "Shield",
            PerkType::Teleport => "Teleport",
            PerkType::Summon => "Summon",
            PerkType::Lightning => "Lightning",
            PerkType::Gravity => "Gravity",
        }
    }

    fn color(&self) -> Color {
        match self {
            PerkType::Projectile => Color::rgb(0.8, 0.8, 0.8),
            PerkType::Homing => Color::rgb(1.0, 0.0, 1.0),
            PerkType::Explosive => Color::rgb(1.0, 0.3, 0.0),
            PerkType::Chain => Color::rgb(1.0, 1.0, 0.0),
            PerkType::Freeze => Color::rgb(0.0, 0.8, 1.0),
            PerkType::Burn => Color::rgb(1.0, 0.5, 0.0),
            PerkType::Poison => Color::rgb(0.5, 1.0, 0.0),
            PerkType::Shield => Color::rgb(0.0, 1.0, 1.0),
            PerkType::Teleport => Color::rgb(0.8, 0.0, 0.8),
            PerkType::Summon => Color::rgb(0.5, 0.0, 0.5),
            PerkType::Lightning => Color::rgb(1.0, 1.0, 1.0),
            PerkType::Gravity => Color::rgb(0.3, 0.3, 0.3),
        }
    }

    fn description(&self) -> &'static str {
        match self {
            PerkType::Projectile => "Launches projectiles",
            PerkType::Homing => "Projectiles home in on targets",
            PerkType::Explosive => "Causes explosions on impact",
            PerkType::Chain => "Chains to nearby enemies",
            PerkType::Freeze => "Freezes enemies in place",
            PerkType::Burn => "Sets enemies on fire",
            PerkType::Poison => "Poisons enemies over time",
            PerkType::Shield => "Creates protective shields",
            PerkType::Teleport => "Teleports caster or targets",
            PerkType::Summon => "Summons creatures to fight",
            PerkType::Lightning => "Calls down lightning strikes",
            PerkType::Gravity => "Manipulates gravitational forces",
        }
    }
}

#[derive(Clone, Debug)]
struct SpellPerk {
    perk_type: PerkType,
    level: u32,
    power: f32,
}

impl SpellPerk {
    fn new(perk_type: PerkType) -> Self {
        Self {
            perk_type,
            level: 1,
            power: 1.0,
        }
    }

    fn upgrade(&mut self) {
        self.level += 1;
        self.power *= 1.2; // 20% power increase per level
    }
}

#[derive(Clone, Debug)]
struct Spell {
    perks: Vec<SpellPerk>,
    mana_cost: f32,
    cast_time: f32,
    cooldown: f32,
    name: String,
}

impl Spell {
    fn new(name: String, perks: Vec<SpellPerk>) -> Self {
        let mana_cost = perks.len() as f32 * 10.0;
        let cast_time = perks.len() as f32 * 0.2;
        let cooldown = perks.len() as f32 * 0.5;

        Self {
            perks,
            mana_cost,
            cast_time,
            cooldown,
            name,
        }
    }

    fn get_description(&self) -> String {
        let mut desc = format!("Spell: {}\n", self.name);
        desc += &format!("Mana Cost: {:.0}\n", self.mana_cost);
        desc += &format!("Cast Time: {:.1}s\n", self.cast_time);
        desc += &format!("Cooldown: {:.1}s\n\n", self.cooldown);
        desc += "Perks:\n";

        for perk in &self.perks {
            desc += &format!("• {} (Lv.{}) - {}\n",
                           perk.perk_type.name(),
                           perk.level,
                           perk.perk_type.description());
        }

        desc
    }

    fn calculate_spell_power(&self) -> f32 {
        self.perks.iter().map(|p| p.power).sum::<f32>() / self.perks.len() as f32
    }
}

#[derive(Clone, Debug)]
struct SpellBook {
    available_perks: Vec<PerkType>,
    selected_perks: Vec<SpellPerk>,
    spells: Vec<Spell>,
    max_perks_per_spell: usize,
}

impl SpellBook {
    fn new() -> Self {
        let available_perks = vec![
            PerkType::Projectile,
            PerkType::Homing,
            PerkType::Explosive,
            PerkType::Chain,
            PerkType::Freeze,
            PerkType::Burn,
            PerkType::Poison,
            PerkType::Shield,
            PerkType::Teleport,
            PerkType::Summon,
            PerkType::Lightning,
            PerkType::Gravity,
        ];

        Self {
            available_perks,
            selected_perks: Vec::new(),
            spells: Vec::new(),
            max_perks_per_spell: 4,
        }
    }

    fn add_perk(&mut self, perk_type: PerkType) {
        if self.selected_perks.len() < self.max_perks_per_spell {
            self.selected_perks.push(SpellPerk::new(perk_type));
        }
    }

    fn remove_perk(&mut self, index: usize) {
        if index < self.selected_perks.len() {
            self.selected_perks.remove(index);
        }
    }

    fn create_spell(&mut self, name: String) -> Option<Spell> {
        if self.selected_perks.is_empty() {
            return None;
        }

        let spell = Spell::new(name, self.selected_perks.clone());
        self.spells.push(spell.clone());
        self.selected_perks.clear();

        Some(spell)
    }

    fn get_spell_combinations(&self) -> Vec<String> {
        // Generate interesting spell combinations
        vec![
            "Fireball: Projectile + Explosive + Burn".to_string(),
            "Ice Lance: Projectile + Freeze + Homing".to_string(),
            "Chain Lightning: Lightning + Chain + Explosive".to_string(),
            "Poison Cloud: Poison + Summon + Gravity".to_string(),
            "Teleport Strike: Teleport + Projectile + Homing".to_string(),
            "Gravity Well: Gravity + Explosive + Chain".to_string(),
            "Shield Bash: Shield + Projectile + Freeze".to_string(),
            "Fire Storm: Burn + Summon + Lightning".to_string(),
        ]
    }
}

#[derive(Clone, Debug)]
struct Player {
    position: Vec2,
    mana: f32,
    max_mana: f32,
    spell_book: SpellBook,
    casting_spell: Option<Spell>,
    cast_timer: f32,
    cooldown_timer: f32,
}

impl Player {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            mana: 100.0,
            max_mana: 100.0,
            spell_book: SpellBook::new(),
            casting_spell: None,
            cast_timer: 0.0,
            cooldown_timer: 0.0,
        }
    }

    fn can_cast_spell(&self, spell: &Spell) -> bool {
        self.mana >= spell.mana_cost && self.casting_spell.is_none() && self.cooldown_timer <= 0.0
    }

    fn start_casting(&mut self, spell: Spell) -> bool {
        if self.can_cast_spell(&spell) {
            self.mana -= spell.mana_cost;
            self.casting_spell = Some(spell);
            self.cast_timer = 0.0;
            true
        } else {
            false
        }
    }

    fn update_casting(&mut self, dt: f32) -> Option<Spell> {
        if let Some(spell) = &self.casting_spell {
            self.cast_timer += dt;

            if self.cast_timer >= spell.cast_time {
                let finished_spell = self.casting_spell.take().unwrap();
                self.cooldown_timer = finished_spell.cooldown;
                return Some(finished_spell);
            }
        }

        if self.cooldown_timer > 0.0 {
            self.cooldown_timer -= dt;
        }

        // Regenerate mana
        self.mana = (self.mana + dt * 10.0).min(self.max_mana);

        None
    }
}

#[derive(Resource)]
struct SpellcastingDemo {
    player: Player,
    cast_spells: Vec<(Spell, Vec2)>,
    current_time: f64,
}

impl SpellcastingDemo {
    fn new() -> Self {
        Self {
            player: Player::new(Vec2::new(400.0, 300.0)),
            cast_spells: Vec::new(),
            current_time: 0.0,
        }
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;

        if let Some(spell) = self.player.update_casting(dt as f32) {
            // Spell finished casting, execute it
            self.execute_spell(spell);
        }
    }

    fn execute_spell(&mut self, spell: Spell) {
        println!("🎆 Executing spell: {}", spell.name);
        println!("   Power: {:.2}", spell.calculate_spell_power());
        println!("   Effects: {:?}", spell.perks.iter().map(|p| p.perk_type.name()).collect::<Vec<_>>());

        self.cast_spells.push((spell, self.player.position));
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spellcasting 3.0: Perks - Modular Spell System".to_string(),
                resolution: (1000.0, 700.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(SpellcastingDemo::new())
        .add_systems(Startup, setup_spellcasting_demo)
        .add_systems(Update, (
            handle_spellcasting_input,
            update_spellcasting_demo,
            render_spellcasting_demo,
            display_spellcasting_info,
        ).chain())
        .run();
}

fn setup_spellcasting_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_spellcasting_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<SpellcastingDemo>,
) {
    demo.update(1.0 / 60.0);

    // Add perks to spell
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        demo.player.spell_book.add_perk(PerkType::Projectile);
        println!("Added Projectile perk");
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        demo.player.spell_book.add_perk(PerkType::Explosive);
        println!("Added Explosive perk");
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        demo.player.spell_book.add_perk(PerkType::Homing);
        println!("Added Homing perk");
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        demo.player.spell_book.add_perk(PerkType::Chain);
        println!("Added Chain perk");
    }

    // Create spell
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        let spell_name = format!("Custom Spell {}", demo.player.spell_book.spells.len() + 1);
        if let Some(spell) = demo.player.spell_book.create_spell(spell_name) {
            println!("Created spell: {}", spell.name);
        } else {
            println!("No perks selected!");
        }
    }

    // Cast spell
    if keyboard_input.just_pressed(KeyCode::Space) && !demo.player.spell_book.spells.is_empty() {
        let spell_index = (demo.current_time * 0.5) as usize % demo.player.spell_book.spells.len();
        if let Some(spell) = demo.player.spell_book.spells.get(spell_index).cloned() {
            if demo.player.start_casting(spell) {
                println!("Started casting spell...");
            } else {
                println!("Cannot cast spell (not enough mana or on cooldown)");
            }
        }
    }

    // Clear perks
    if keyboard_input.just_pressed(KeyCode::KeyX) {
        demo.player.spell_book.selected_perks.clear();
        println!("Cleared selected perks");
    }
}

fn update_spellcasting_demo(time: Res<Time>, mut demo: ResMut<SpellcastingDemo>) {
    // Updates are handled in input system
}

fn render_spellcasting_demo(
    mut commands: Commands,
    mut ui_entities: Local<Vec<Entity>>,
    mut spell_entities: Local<Vec<Entity>>,
    mut player_entity: Local<Option<Entity>>,
    demo: Res<SpellcastingDemo>,
) {
    // Clear previous frame
    for entity in ui_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in spell_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }

    // Render player
    let player_color = if demo.player.casting_spell.is_some() {
        Color::rgb(1.0, 0.5, 0.0) // Casting - orange
    } else if demo.player.cooldown_timer > 0.0 {
        Color::rgb(0.5, 0.5, 0.5) // Cooldown - gray
    } else {
        Color::rgb(0.2, 0.8, 0.2) // Ready - green
    };

    let player_render = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: player_color,
                custom_size: Some(Vec2::new(30.0, 40.0)),
                ..default()
            },
            transform: Transform::from_xyz(demo.player.position.x, demo.player.position.y, 1.0),
            ..default()
        },
        Text2dBundle {
            text: Text::from_section(
                format!("MP: {:.0}/{:.0}", demo.player.mana, demo.player.max_mana),
                TextStyle {
                    font_size: 10.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(0.0, 25.0, 2.0),
            ..default()
        },
    )).id();
    *player_entity = Some(player_render);

    // Render cast spells
    for (i, (spell, position)) in demo.cast_spells.iter().enumerate() {
        let spell_color = Color::rgb(0.8, 0.2, 0.8);
        let spell_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: spell_color,
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x + 50.0 + i as f32 * 30.0, position.y, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    &spell.name,
                    TextStyle {
                        font_size: 8.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ..default()
            },
        )).id();
        spell_entities.push(spell_entity);
    }

    // Render available perks
    let perk_types = [
        (PerkType::Projectile, Vec2::new(50.0, 600.0)),
        (PerkType::Explosive, Vec2::new(100.0, 600.0)),
        (PerkType::Homing, Vec2::new(150.0, 600.0)),
        (PerkType::Chain, Vec2::new(200.0, 600.0)),
    ];

    for (perk_type, position) in &perk_types {
        let perk_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: perk_type.color(),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    perk_type.name(),
                    TextStyle {
                        font_size: 8.0,
                        color: Color::BLACK,
                        ..default()
                    },
                ),
                ..default()
            },
        )).id();
        ui_entities.push(perk_entity);
    }

    // Render selected perks
    for (i, perk) in demo.player.spell_book.selected_perks.iter().enumerate() {
        let selected_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: perk.perk_type.color(),
                    custom_size: Some(Vec2::new(25.0, 25.0)),
                    ..default()
                },
                transform: Transform::from_xyz(400.0 + i as f32 * 30.0, 50.0, 2.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    format!("Lv.{}", perk.level),
                    TextStyle {
                        font_size: 6.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ..default()
            },
        )).id();
        ui_entities.push(selected_entity);
    }

    // Render created spells
    for (i, spell) in demo.player.spell_book.spells.iter().enumerate() {
        let spell_ui_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.5, 0.5, 0.8),
                    custom_size: Some(Vec2::new(80.0, 40.0)),
                    ..default()
                },
                transform: Transform::from_xyz(800.0, 600.0 - i as f32 * 50.0, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    &spell.name,
                    TextStyle {
                        font_size: 10.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ..default()
            },
        )).id();
        ui_entities.push(spell_ui_entity);
    }
}

fn display_spellcasting_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<SpellcastingDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        println!("\n=== Spellcasting 3.0: Perks Demo ===");
        println!("Player Mana: {:.0}/{:.0}", demo.player.mana, demo.player.max_mana);
        println!("Selected Perks: {}/{}", demo.player.spell_book.selected_perks.len(), demo.player.spell_book.max_perks_per_spell);
        println!("Created Spells: {}", demo.player.spell_book.spells.len());
        println!("Cast Spells: {}", demo.cast_spells.len());

        if let Some(casting_spell) = &demo.player.casting_spell {
            println!("Currently Casting: {} ({:.1}s / {:.1}s)",
                    casting_spell.name, demo.player.cast_timer, casting_spell.cast_time);
        } else if demo.player.cooldown_timer > 0.0 {
            println!("Cooldown: {:.1}s", demo.player.cooldown_timer);
        } else {
            println!("Ready to cast!");
        }

        println!("\nSelected Perks:");
        for (i, perk) in demo.player.spell_book.selected_perks.iter().enumerate() {
            println!("  {}: {} (Lv.{}, Power: {:.1})",
                    i + 1, perk.perk_type.name(), perk.level, perk.power);
        }

        println!("\nCreated Spells:");
        for spell in &demo.player.spell_book.spells {
            println!("  {} - Cost: {:.0} MP, Power: {:.2}",
                    spell.name, spell.mana_cost, spell.calculate_spell_power());
        }

        println!("\nPerk Keys:");
        println!("  1: Projectile | 2: Explosive | 3: Homing | 4: Chain");
        println!("  C: Create Spell | Space: Cast Spell | X: Clear Perks");
        println!("  I: Show this info");
        println!("\nSpell System:");
        println!("• Combine perks to create unique spells");
        println!("• Higher level perks have more power");
        println!("• More perks = higher mana cost and cast time");
        println!("• Spells have cooldowns after casting");
        println!("======================\n");
    }
}
