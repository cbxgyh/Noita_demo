use bevy::prelude::*;
use std::collections::HashMap;

// Magic system based on "Spellcasting 3.0: Perks" and "Ability Subroutines"

// Spell perks as described in the blog posts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellPerk {
    // Basic projectile perks
    Projectile,
    ProjectileGravity,
    ProjectilePierce,
    ProjectileBounce,
    ProjectileHoming,

    // Damage perks
    Damage,
    DamageArea,
    DamagePoison,
    DamageFire,
    DamageIce,

    // Effect perks
    Explosion,
    ChainReaction,
    TriggerTimer,
    TriggerDeath,

    // Modifier perks
    Multiply,
    Split,
    Accelerate,
    Decelerate,

    // Special perks
    Teleport,
    Summon,
    Shield,
    Invisibility,
}

impl SpellPerk {
    pub fn name(&self) -> &'static str {
        match self {
            SpellPerk::Projectile => "Projectile",
            SpellPerk::ProjectileGravity => "Gravity",
            SpellPerk::ProjectilePierce => "Pierce",
            SpellPerk::ProjectileBounce => "Bounce",
            SpellPerk::ProjectileHoming => "Homing",
            SpellPerk::Damage => "Damage",
            SpellPerk::DamageArea => "Area Damage",
            SpellPerk::DamagePoison => "Poison",
            SpellPerk::DamageFire => "Fire",
            SpellPerk::DamageIce => "Ice",
            SpellPerk::Explosion => "Explosion",
            SpellPerk::ChainReaction => "Chain",
            SpellPerk::TriggerTimer => "Timer",
            SpellPerk::TriggerDeath => "On Death",
            SpellPerk::Multiply => "Multiply",
            SpellPerk::Split => "Split",
            SpellPerk::Accelerate => "Accelerate",
            SpellPerk::Decelerate => "Decelerate",
            SpellPerk::Teleport => "Teleport",
            SpellPerk::Summon => "Summon",
            SpellPerk::Shield => "Shield",
            SpellPerk::Invisibility => "Invisible",
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            SpellPerk::Projectile => 10,
            SpellPerk::ProjectileGravity => 5,
            SpellPerk::ProjectilePierce => 15,
            SpellPerk::ProjectileBounce => 12,
            SpellPerk::ProjectileHoming => 20,
            SpellPerk::Damage => 8,
            SpellPerk::DamageArea => 12,
            SpellPerk::DamagePoison => 10,
            SpellPerk::DamageFire => 10,
            SpellPerk::DamageIce => 10,
            SpellPerk::Explosion => 25,
            SpellPerk::ChainReaction => 18,
            SpellPerk::TriggerTimer => 15,
            SpellPerk::TriggerDeath => 15,
            SpellPerk::Multiply => 20,
            SpellPerk::Split => 16,
            SpellPerk::Accelerate => 8,
            SpellPerk::Decelerate => 8,
            SpellPerk::Teleport => 30,
            SpellPerk::Summon => 25,
            SpellPerk::Shield => 20,
            SpellPerk::Invisibility => 35,
        }
    }
}

// Spell represents a combination of perks
#[derive(Component, Debug, Clone)]
pub struct Spell {
    pub perks: Vec<SpellPerk>,
    pub mana_cost: u32,
    pub cooldown: f32,
    pub cooldown_timer: f32,
    pub cast_position: Vec2,
    pub caster: Entity,
}

impl Spell {
    pub fn new(perks: Vec<SpellPerk>) -> Self {
        let mana_cost = perks.iter().map(|p| p.cost()).sum();
        Self {
            perks,
            mana_cost,
            cooldown: 1.0, // Base cooldown
            cooldown_timer: 0.0,
            cast_position: Vec2::ZERO,
            caster: Entity::PLACEHOLDER,
        }
    }

    pub fn can_cast(&self, mana: u32) -> bool {
        self.cooldown_timer <= 0.0 && mana >= self.mana_cost
    }

    pub fn cast(&mut self, position: Vec2, caster: Entity) -> SpellInstance {
        self.cast_position = position;
        self.caster = caster;
        self.cooldown_timer = self.cooldown;

        SpellInstance::new(self.clone())
    }
}

// Active spell instance during execution
#[derive(Component, Debug, Clone)]
pub struct SpellInstance {
    pub spell: Spell,
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub effects: Vec<SpellEffect>,
    pub children: Vec<SpellInstance>,
}

impl SpellInstance {
    pub fn new(spell: Spell) -> Self {
        Self {
            spell,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            lifetime: 0.0,
            max_lifetime: 5.0, // Default lifetime
            effects: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.lifetime += dt;

        // Apply perks to movement and behavior
        self.apply_perks(dt);

        // Update position
        self.position += self.velocity * dt;

        // Update effects
        self.effects.retain_mut(|effect| {
            effect.update(dt);
            effect.is_alive()
        });

        // Update children
        for child in &mut self.children {
            child.update(dt);
        }
        self.children.retain(|child| child.lifetime < child.max_lifetime);
    }

    fn apply_perks(&mut self, dt: f32) {
        for perk in &self.spell.perks {
            match perk {
                SpellPerk::ProjectileGravity => {
                    self.velocity.y -= 30.0 * dt; // Gravity
                }
                SpellPerk::ProjectileBounce => {
                    // Bounce off boundaries (would need collision detection)
                }
                SpellPerk::ProjectileHoming => {
                    // Home towards nearest enemy (would need target finding)
                }
                SpellPerk::Accelerate => {
                    let dir = self.velocity.normalize_or_zero();
                    self.velocity += dir * 100.0 * dt;
                }
                SpellPerk::Decelerate => {
                    self.velocity *= 1.0 - 2.0 * dt; // Slow down
                }
                _ => {} // Other perks handled in effects
            }
        }
    }

    pub fn create_effects(&mut self) {
        for perk in &self.spell.perks {
            match perk {
                SpellPerk::Damage => {
                    self.effects.push(SpellEffect::Damage { amount: 10.0 });
                }
                SpellPerk::DamageArea => {
                    self.effects.push(SpellEffect::AreaDamage { radius: 20.0, amount: 15.0 });
                }
                SpellPerk::DamageFire => {
                    self.effects.push(SpellEffect::FireDamage { amount: 8.0, duration: 3.0 });
                }
                SpellPerk::DamagePoison => {
                    self.effects.push(SpellEffect::PoisonDamage { amount: 5.0, duration: 5.0 });
                }
                SpellPerk::Explosion => {
                    self.effects.push(SpellEffect::Explosion { radius: 50.0, damage: 30.0 });
                }
                SpellPerk::Multiply => {
                    // Create multiple projectiles
                    for i in 1..3 {
                        let angle = (i as f32 / 3.0) * std::f32::consts::TAU;
                        let mut child = SpellInstance::new(self.spell.clone());
                        child.velocity = Vec2::new(angle.cos(), angle.sin()) * 100.0;
                        child.position = self.position;
                        self.children.push(child);
                    }
                }
                SpellPerk::Split => {
                    // Split into smaller projectiles
                    // Implementation similar to multiply but smaller
                }
                _ => {}
            }
        }
    }
}

// Spell effects that can be applied
#[derive(Debug, Clone)]
pub enum SpellEffect {
    Damage { amount: f32 },
    AreaDamage { radius: f32, amount: f32 },
    FireDamage { amount: f32, duration: f32 },
    PoisonDamage { amount: f32, duration: f32 },
    Explosion { radius: f32, damage: f32 },
    Heal { amount: f32 },
    Buff { stat: String, amount: f32, duration: f32 },
    Debuff { stat: String, amount: f32, duration: f32 },
}

impl SpellEffect {
    pub fn update(&mut self, dt: f32) {
        match self {
            SpellEffect::FireDamage { duration, .. } |
            SpellEffect::PoisonDamage { duration, .. } |
            SpellEffect::Buff { duration, .. } |
            SpellEffect::Debuff { duration, .. } => {
                *duration -= dt;
            }
            _ => {}
        }
    }

    pub fn is_alive(&self) -> bool {
        match self {
            SpellEffect::FireDamage { duration, .. } |
            SpellEffect::PoisonDamage { duration, .. } |
            SpellEffect::Buff { duration, .. } |
            SpellEffect::Debuff { duration, .. } => *duration > 0.0,
            _ => true, // Instant effects
        }
    }
}

// Player magic system
#[derive(Component)]
pub struct MagicUser {
    pub mana: u32,
    pub max_mana: u32,
    pub mana_regen: f32,
    pub spells: Vec<Spell>,
    pub selected_spell: usize,
}

impl Default for MagicUser {
    fn default() -> Self {
        Self {
            mana: 100,
            max_mana: 100,
            mana_regen: 5.0, // Mana per second
            spells: vec![
                Spell::new(vec![SpellPerk::Projectile, SpellPerk::Damage]),
                Spell::new(vec![SpellPerk::Projectile, SpellPerk::Explosion]),
            ],
            selected_spell: 0,
        }
    }
}

// Subroutines for spell modification as described in "Ability Subroutines"
#[derive(Debug, Clone)]
pub struct SpellSubroutine {
    pub perks: Vec<SpellPerk>,
    pub condition: SubroutineCondition,
}

#[derive(Debug, Clone)]
pub enum SubroutineCondition {
    OnHit,
    OnDeath,
    Timer(f32),
    Distance(f32),
    HealthPercent(f32),
}

// Systems for magic
pub fn update_magic_users(time: Res<Time>, mut query: Query<&mut MagicUser>) {
    let dt = time.delta_seconds();

    for mut magic_user in query.iter_mut() {
        // Regen mana
        magic_user.mana = (magic_user.mana as f32 + magic_user.mana_regen * dt)
            .min(magic_user.max_mana as f32) as u32;

        // Update spell cooldowns
        for spell in &mut magic_user.spells {
            spell.cooldown_timer = (spell.cooldown_timer - dt).max(0.0);
        }
    }
}

pub fn cast_spell(
    mut commands: Commands,
    mut magic_users: Query<(&mut MagicUser, &Transform)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if !mouse_input.just_pressed(MouseButton::Right) {
        return;
    }

    for (mut magic_user, transform) in magic_users.iter_mut() {
        if magic_user.spells.is_empty() {
            continue;
        }
        //   let spell = &mut magic_user.spells[magic_user.selected_spell];
        //   if !spell.can_cast(magic_user.mana) {

        // Get selected_spell index first to avoid borrowing conflicts
        let selected_spell_index = magic_user.selected_spell;
        let mana = magic_user.mana;
        
        let spell = &mut magic_user.spells[selected_spell_index];

        if !spell.can_cast(mana) {
            continue;
        }

        // Get mouse world position
        if let Ok((camera, camera_transform)) = camera_query.get_single() {
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                        let cast_pos = world_pos.origin.truncate();

                        // Cast the spell
                        let mut instance = spell.cast(cast_pos, Entity::PLACEHOLDER);
                        instance.position = transform.translation.truncate();
                        instance.velocity = (cast_pos - instance.position).normalize() * 200.0;

                        // Create spell entity
                        commands.spawn((
                            SpriteBundle {
                                sprite: Sprite {
                                    color: Color::rgb(0.8, 0.2, 1.0),
                                    custom_size: Some(Vec2::new(8.0, 8.0)),
                                    ..default()
                                },
                                transform: Transform::from_translation(instance.position.extend(0.0)),
                                ..default()
                            },
                            instance,
                        ));

                        magic_user.mana -= spell.mana_cost;
                    }
                }
            }
        }
    }
}

pub fn update_spell_instances(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut SpellInstance, &mut Transform)>,
) {
    let dt = time.delta_seconds();

    for (entity, mut instance, mut transform) in query.iter_mut() {
        instance.update(dt);
        transform.translation = instance.position.extend(0.0);

        // Remove dead spells
        if instance.lifetime >= instance.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spell_collision_detection(
    mut spell_query: Query<&mut SpellInstance>,
    // Would need collision detection with enemies/objects
) {
    // Handle spell collisions and apply effects
    // This would integrate with the physics system
}

// Spell crafting system
pub fn create_spell_from_perks(perks: Vec<SpellPerk>) -> Spell {
    Spell::new(perks)
}

// Example spell combinations
pub fn get_example_spells() -> Vec<Spell> {
    vec![
        // Basic projectile
        Spell::new(vec![SpellPerk::Projectile, SpellPerk::Damage]),

        // Fireball
        Spell::new(vec![SpellPerk::Projectile, SpellPerk::DamageFire, SpellPerk::Explosion]),

        // Poison arrow
        Spell::new(vec![SpellPerk::Projectile, SpellPerk::DamagePoison, SpellPerk::ProjectilePierce]),

        // Bouncing bomb
        Spell::new(vec![SpellPerk::Projectile, SpellPerk::ProjectileBounce, SpellPerk::Explosion]),

        // Homing missile
        Spell::new(vec![SpellPerk::Projectile, SpellPerk::ProjectileHoming, SpellPerk::DamageArea]),

        // Chain lightning
        Spell::new(vec![SpellPerk::Projectile, SpellPerk::ChainReaction, SpellPerk::Damage]),

        // Multi-shot
        Spell::new(vec![SpellPerk::Projectile, SpellPerk::Multiply, SpellPerk::Damage]),

        // Teleport
        Spell::new(vec![SpellPerk::Teleport]),
    ]
}
