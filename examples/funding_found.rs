// Example: Funding Found
// Based on "Funding Found" blog post
// https://www.slowrush.dev/news/funding-found

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Funding and monetization systems
// Demonstrates loot crates, rewards, and progression systems

#[derive(Clone, Debug)]
enum LootRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl LootRarity {
    fn color(&self) -> Color {
        match self {
            LootRarity::Common => Color::rgb(0.7, 0.7, 0.7),
            LootRarity::Uncommon => Color::rgb(0.2, 0.8, 0.2),
            LootRarity::Rare => Color::rgb(0.2, 0.4, 1.0),
            LootRarity::Epic => Color::rgb(0.8, 0.2, 1.0),
            LootRarity::Legendary => Color::rgb(1.0, 0.8, 0.0),
        }
    }

    fn value(&self) -> u32 {
        match self {
            LootRarity::Common => 10,
            LootRarity::Uncommon => 25,
            LootRarity::Rare => 50,
            LootRarity::Epic => 100,
            LootRarity::Legendary => 250,
        }
    }

    fn drop_chance(&self) -> f32 {
        match self {
            LootRarity::Common => 0.5,
            LootRarity::Uncommon => 0.25,
            LootRarity::Rare => 0.15,
            LootRarity::Epic => 0.08,
            LootRarity::Legendary => 0.02,
        }
    }
}

#[derive(Clone, Debug)]
enum LootType {
    Currency,
    HealthPotion,
    ManaPotion,
    WeaponUpgrade,
    SpellUpgrade,
    Cosmetic,
}

impl LootType {
    fn name(&self) -> &'static str {
        match self {
            LootType::Currency => "Gold Coins",
            LootType::HealthPotion => "Health Potion",
            LootType::ManaPotion => "Mana Potion",
            LootType::WeaponUpgrade => "Weapon Upgrade",
            LootType::SpellUpgrade => "Spell Upgrade",
            LootType::Cosmetic => "Cosmetic Item",
        }
    }

    fn icon_color(&self) -> Color {
        match self {
            LootType::Currency => Color::rgb(1.0, 0.8, 0.0),
            LootType::HealthPotion => Color::rgb(1.0, 0.2, 0.2),
            LootType::ManaPotion => Color::rgb(0.2, 0.4, 1.0),
            LootType::WeaponUpgrade => Color::rgb(0.8, 0.4, 0.2),
            LootType::SpellUpgrade => Color::rgb(0.6, 0.2, 0.8),
            LootType::Cosmetic => Color::rgb(1.0, 0.6, 1.0),
        }
    }
}

#[derive(Clone, Debug)]
struct LootItem {
    loot_type: LootType,
    rarity: LootRarity,
    quantity: u32,
}

impl LootItem {
    fn new(loot_type: LootType, rarity: LootRarity) -> Self {
        let quantity = match loot_type {
            LootType::Currency => rarity.value(),
            _ => 1,
        };

        Self {
            loot_type,
            rarity,
            quantity,
        }
    }

    fn value(&self) -> u32 {
        self.rarity.value() * self.quantity
    }
}

#[derive(Clone, Debug)]
struct LootCrate {
    position: Vec2,
    size: Vec2,
    is_open: bool,
    items: Vec<LootItem>,
    open_timer: f32,
}

impl LootCrate {
    fn new(position: Vec2) -> Self {
        let mut items = Vec::new();

        // Generate random loot based on rarity chances
        for _ in 0..(3 + rand::random::<usize>() % 3) { // 3-5 items
            let loot_type = match rand::random::<f32>() {
                r if r < 0.4 => LootType::Currency,
                r if r < 0.6 => LootType::HealthPotion,
                r if r < 0.7 => LootType::ManaPotion,
                r if r < 0.8 => LootType::WeaponUpgrade,
                r if r < 0.9 => LootType::SpellUpgrade,
                _ => LootType::Cosmetic,
            };

            let rarity = match rand::random::<f32>() {
                r if r < LootRarity::Legendary.drop_chance() => LootRarity::Legendary,
                r if r < LootRarity::Epic.drop_chance() => LootRarity::Epic,
                r if r < LootRarity::Rare.drop_chance() => LootRarity::Rare,
                r if r < LootRarity::Uncommon.drop_chance() => LootRarity::Uncommon,
                _ => LootRarity::Common,
            };

            items.push(LootItem::new(loot_type, rarity));
        }

        Self {
            position,
            size: Vec2::new(40.0, 40.0),
            is_open: false,
            items,
            open_timer: 0.0,
        }
    }

    fn open(&mut self) {
        if !self.is_open {
            self.is_open = true;
            self.open_timer = 1.0; // Animation time
        }
    }

    fn update(&mut self, dt: f32) {
        if self.is_open {
            self.open_timer -= dt;
            if self.open_timer <= 0.0 {
                self.open_timer = 0.0;
            }
        }
    }

    fn collect_items(&mut self) -> Vec<LootItem> {
        if self.is_open && self.open_timer <= 0.0 {
            std::mem::take(&mut self.items)
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone, Debug)]
struct Player {
    position: Vec2,
    velocity: Vec2,
    size: Vec2,
    currency: u32,
    health: f32,
    max_health: f32,
    experience: u32,
    level: u32,
    inventory: Vec<LootItem>,
}

impl Player {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            size: Vec2::new(20.0, 30.0),
            currency: 0,
            health: 100.0,
            max_health: 100.0,
            experience: 0,
            level: 1,
            inventory: Vec::new(),
        }
    }

    fn add_experience(&mut self, exp: u32) {
        self.experience += exp;
        let exp_needed = self.level * 100;

        while self.experience >= exp_needed {
            self.experience -= exp_needed;
            self.level += 1;
            self.max_health += 10.0;
            self.health = self.max_health;
            println!("🎉 Level up! Now level {}", self.level);
        }
    }

    fn add_item(&mut self, item: LootItem) {
        match item.loot_type {
            LootType::Currency => {
                self.currency += item.quantity;
                println!("💰 +{} gold! Total: {}", item.quantity, self.currency);
            }
            LootType::HealthPotion => {
                self.health = (self.health + 25.0).min(self.max_health);
                println!("❤️ Health potion! HP: {:.0}/{:.0}", self.health, self.max_health);
            }
            LootType::ManaPotion => {
                // Would affect mana in full game
                println!("🔵 Mana potion!");
            }
            _ => {
                self.inventory.push(item.clone());
                println!("📦 {} ({:?})", item.loot_type.name(), item.rarity);
            }
        }
    }

    fn use_currency(&mut self, amount: u32) -> bool {
        if self.currency >= amount {
            self.currency -= amount;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
struct ShopItem {
    item: LootItem,
    cost: u32,
    description: String,
}

struct FundingGame {
    player: Player,
    loot_crates: Vec<LootCrate>,
    shop_items: Vec<ShopItem>,
    width: f32,
    height: f32,
    total_earnings: u32,
}

impl FundingGame {
    fn new(width: f32, height: f32) -> Self {
        let player = Player::new(Vec2::new(width / 2.0, height / 2.0));

        // Create loot crates
        let mut loot_crates = Vec::new();
        for i in 0..8 {
            let x = 100.0 + (i % 4) as f32 * 100.0;
            let y = 100.0 + (i / 4) as f32 * 100.0;
            loot_crates.push(LootCrate::new(Vec2::new(x, y)));
        }

        // Create shop items
        let shop_items = vec![
            ShopItem {
                item: LootItem::new(LootType::WeaponUpgrade, LootRarity::Rare),
                cost: 200,
                description: "Upgrade your weapon damage".to_string(),
            },
            ShopItem {
                item: LootItem::new(LootType::HealthPotion, LootRarity::Common),
                cost: 50,
                description: "Restore health".to_string(),
            },
            ShopItem {
                item: LootItem::new(LootType::Cosmetic, LootRarity::Epic),
                cost: 500,
                description: "Fancy cosmetic item".to_string(),
            },
        ];

        Self {
            player,
            loot_crates,
            shop_items,
            width,
            height,
            total_earnings: 0,
        }
    }

    fn update(&mut self, dt: f32, input: &PlayerInput) {
        // Update player physics
        self.player.velocity.x = input.move_x * 120.0;
        self.player.velocity.y -= 300.0 * dt;

        let new_position = self.player.position + self.player.velocity * dt;

        // Simple ground collision
        if new_position.y <= 50.0 {
            new_position.y = 50.0;
            self.player.velocity.y = 0.0;

            if input.jump {
                self.player.velocity.y = 180.0;
            }
        }

        self.player.position = new_position;

        // Update loot crates
        for crate_ in &mut self.loot_crates {
            crate_.update(dt);
        }

        // Check for crate collection
        let player_rect = Rect::from_center_size(self.player.position, self.player.size);

        for crate_ in &mut self.loot_crates {
            if !crate_.is_open {
                let crate_rect = Rect::from_center_size(crate_.position, crate_.size);
                if player_rect.intersect(crate_rect).is_some() && input.interact {
                    crate_.open();
                    self.player.add_experience(10);
                    println!("🎁 Opened loot crate!");
                }
            } else {
                // Collect items when crate is fully open
                let items = crate_.collect_items();
                for item in items {
                    self.player.add_item(item.clone());
                    self.total_earnings += item.value();
                }
            }
        }
    }

    fn buy_item(&mut self, shop_index: usize) -> bool {
        if let Some(shop_item) = self.shop_items.get(shop_index) {
            if self.player.use_currency(shop_item.cost) {
                self.player.add_item(shop_item.item.clone());
                println!("✅ Purchased: {}", shop_item.item.loot_type.name());
                true
            } else {
                println!("❌ Not enough gold!");
                false
            }
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
struct PlayerInput {
    move_x: f32,
    jump: bool,
    interact: bool,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            move_x: 0.0,
            jump: false,
            interact: false,
        }
    }
}

#[derive(Resource)]
struct FundingGameResource(FundingGame);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Funding Found - Loot Crates & Monetization".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(FundingGameResource(FundingGame::new(800.0, 600.0)))
        .add_systems(Startup, setup_funding_demo)
        .add_systems(Update, (
            handle_funding_input,
            update_funding_game,
            render_funding_game,
            display_funding_stats,
        ).chain())
        .run();
}

fn setup_funding_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_funding_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<FundingGameResource>,
) {
    let mut input = PlayerInput::default();

    // Movement
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.move_x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.move_x = 1.0;
    }

    input.jump = keyboard_input.just_pressed(KeyCode::Space);
    input.interact = keyboard_input.just_pressed(KeyCode::KeyE);

    game.0.update(1.0 / 60.0, &input);

    // Shop purchases
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        game.0.buy_item(0);
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        game.0.buy_item(1);
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        game.0.buy_item(2);
    }
}

fn update_funding_game(time: Res<Time>, mut game: ResMut<FundingGameResource>) {
    // Updates are handled in input system
}

fn render_funding_game(
    mut commands: Commands,
    mut crate_entities: Local<Vec<Entity>>,
    mut item_entities: Local<Vec<Entity>>,
    mut player_entity: Local<Option<Entity>>,
    mut shop_entities: Local<Vec<Entity>>,
    game: Res<FundingGameResource>,
) {
    // Clear previous frame
    for entity in crate_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in item_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    if let Some(entity) = *player_entity {
        commands.entity(entity).despawn();
    }
    for entity in shop_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render loot crates
    for (i, crate_) in game.0.loot_crates.iter().enumerate() {
        let color = if crate_.is_open {
            Color::rgb(0.6, 0.3, 0.1) // Open crate color
        } else {
            Color::rgb(0.8, 0.6, 0.4) // Closed crate color
        };

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(crate_.size),
                    ..default()
                },
                transform: Transform::from_xyz(crate_.position.x, crate_.position.y, 0.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    if crate_.is_open { "OPEN" } else { "PRESS E" },
                    TextStyle {
                        font_size: 10.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 25.0, 1.0),
                ..default()
            },
        )).id();
        crate_entities.push(entity);
    }

    // Render player
    let entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(game.0.player.size),
                ..default()
            },
            transform: Transform::from_xyz(game.0.player.position.x, game.0.player.position.y, 1.0),
            ..default()
        },
        Text2dBundle {
            text: Text::from_section(
                format!("Lv.{} ${}", game.0.player.level, game.0.player.currency),
                TextStyle {
                    font_size: 12.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            transform: Transform::from_xyz(0.0, 20.0, 2.0),
            ..default()
        },
    )).id();
    *player_entity = Some(entity);

    // Render shop
    let shop_items = vec![
        ("Weapon Upgrade - $200", Vec2::new(650.0, 500.0)),
        ("Health Potion - $50", Vec2::new(650.0, 450.0)),
        ("Cosmetic - $500", Vec2::new(650.0, 400.0)),
    ];

    for (i, (item_name, position)) in shop_items.iter().enumerate() {
        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.4, 0.4, 0.6),
                    custom_size: Some(Vec2::new(120.0, 30.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 1.0),
                ..default()
            },
            Text2dBundle {
                text: Text::from_section(
                    item_name,
                    TextStyle {
                        font_size: 10.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                ..default()
            },
        )).id();
        shop_entities.push(entity);
    }
}

fn display_funding_stats(keyboard_input: Res<ButtonInput<KeyCode>>, game: Res<FundingGameResource>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Funding Found Demo ===");
        println!("Player Stats:");
        println!("  Level: {}", game.0.player.level);
        println!("  Experience: {}/{}", game.0.player.experience, game.0.player.level * 100);
        println!("  Gold: ${}", game.0.player.currency);
        println!("  Health: {:.0}/{:.0}", game.0.player.health, game.0.player.max_health);
        println!("  Total Earnings: ${}", game.0.total_earnings);
        println!("\nLoot Crates: {}/{}", game.0.loot_crates.iter().filter(|c| c.is_open).count(), game.0.loot_crates.len());
        println!("Inventory Items: {}", game.0.player.inventory.len());

        println!("\nShop (Press 1-3 to buy):");
        for (i, shop_item) in game.0.shop_items.iter().enumerate() {
            println!("  {}: {} - ${}", i + 1, shop_item.item.loot_type.name(), shop_item.cost);
        }

        println!("\nControls:");
        println!("  A/D: Move | Space: Jump | E: Open crate");
        println!("  1-3: Buy shop items");
        println!("  H: Show this info");
        println!("====================\n");
    }
}
