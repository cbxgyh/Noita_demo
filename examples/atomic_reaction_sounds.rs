// Example: Atomic Reaction Sounds
// Based on "Atomic Reaction Sounds" blog post
// https://www.slowrush.dev/news/atomic-reaction-sounds

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Procedural audio for atomic reactions
// Demonstrates sound generation based on physics interactions

#[derive(Clone, Debug, PartialEq)]
enum Element {
    Sand, Water, Fire, Steam, Stone, Acid, Metal, Electricity,
}

impl Element {
    fn base_frequency(&self) -> f32 {
        match self {
            Element::Sand => 200.0,
            Element::Water => 300.0,
            Element::Fire => 150.0,
            Element::Steam => 400.0,
            Element::Stone => 100.0,
            Element::Acid => 250.0,
            Element::Metal => 180.0,
            Element::Electricity => 600.0,
        }
    }

    fn sound_character(&self) -> SoundCharacter {
        match self {
            Element::Sand => SoundCharacter { duration: 0.1, attack: 0.01, decay: 0.05, sustain: 0.3, release: 0.04 },
            Element::Water => SoundCharacter { duration: 0.15, attack: 0.02, decay: 0.08, sustain: 0.5, release: 0.05 },
            Element::Fire => SoundCharacter { duration: 0.2, attack: 0.01, decay: 0.1, sustain: 0.7, release: 0.09 },
            Element::Steam => SoundCharacter { duration: 0.3, attack: 0.05, decay: 0.15, sustain: 0.4, release: 0.1 },
            Element::Stone => SoundCharacter { duration: 0.08, attack: 0.005, decay: 0.03, sustain: 0.8, release: 0.045 },
            Element::Acid => SoundCharacter { duration: 0.25, attack: 0.03, decay: 0.12, sustain: 0.6, release: 0.08 },
            Element::Metal => SoundCharacter { duration: 0.12, attack: 0.008, decay: 0.04, sustain: 0.9, release: 0.06 },
            Element::Electricity => SoundCharacter { duration: 0.05, attack: 0.002, decay: 0.02, sustain: 1.0, release: 0.028 },
        }
    }
}

#[derive(Clone, Debug)]
struct SoundCharacter {
    duration: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

#[derive(Clone, Debug)]
struct ReactionSound {
    id: u32,
    position: Vec2,
    start_time: f64,
    elements: Vec<Element>,
    intensity: f32,
    frequency_variation: f32,
    is_active: bool,
}

impl ReactionSound {
    fn new(id: u32, position: Vec2, elements: Vec<Element>, intensity: f32) -> Self {
        Self {
            id,
            position,
            start_time: 0.0,
            elements,
            intensity,
            frequency_variation: rand::random::<f32>() * 0.3 - 0.15, // -15% to +15%
            is_active: true,
        }
    }

    fn generate_audio_sample(&self, time: f64) -> f32 {
        let elapsed = time - self.start_time;
        if elapsed < 0.0 || !self.is_active {
            return 0.0;
        }

        let mut sample = 0.0;

        for (i, element) in self.elements.iter().enumerate() {
            let char = element.sound_character();
            let base_freq = element.base_frequency();
            let freq = base_freq * (1.0 + self.frequency_variation) * (1.0 + i as f32 * 0.1);

            // ADSR envelope
            let envelope = self.calculate_adsr_envelope(elapsed as f32, &char);

            // Generate waveform
            let phase = (elapsed * freq as f64 * 2.0 * std::f64::consts::PI) as f32;
            let wave = phase.sin() * 0.3 + (phase * 2.0).sin() * 0.2 + (phase * 3.0).sin() * 0.1;

            sample += wave * envelope * self.intensity;
        }

        // Normalize and limit
        (sample / self.elements.len() as f32).clamp(-1.0, 1.0)
    }

    fn calculate_adsr_envelope(&self, time: f32, char: &SoundCharacter) -> f32 {
        if time < char.attack {
            // Attack phase
            time / char.attack
        } else if time < char.attack + char.decay {
            // Decay phase
            let decay_time = time - char.attack;
            1.0 - (1.0 - char.sustain) * (decay_time / char.decay)
        } else if time < char.duration - char.release {
            // Sustain phase
            char.sustain
        } else if time < char.duration {
            // Release phase
            let release_time = time - (char.duration - char.release);
            char.sustain * (1.0 - release_time / char.release)
        } else {
            0.0
        }
    }

    fn is_finished(&self, current_time: f64) -> bool {
        let total_duration = self.elements.iter()
            .map(|e| e.sound_character().duration)
            .fold(0.0f32, |a, b| a.max(b));

        current_time - self.start_time > total_duration as f64
    }
}

#[derive(Clone, Debug)]
struct ProceduralAudioEngine {
    sounds: Vec<ReactionSound>,
    listener_position: Vec2,
    master_volume: f32,
    next_sound_id: u32,
    current_time: f64,
}

impl ProceduralAudioEngine {
    fn new() -> Self {
        Self {
            sounds: Vec::new(),
            listener_position: Vec2::ZERO,
            master_volume: 1.0,
            next_sound_id: 0,
            current_time: 0.0,
        }
    }

    fn play_reaction_sound(&mut self, position: Vec2, elements: Vec<Element>, intensity: f32) {
        // Clone so we can both log and store
        let sound = ReactionSound::new(self.next_sound_id, position, elements.clone(), intensity);
        self.sounds.push(sound);
        self.next_sound_id += 1;

        println!("⚛️  Atomic reaction: {:?} at ({:.1}, {:.1})", elements, position.x, position.y);
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;

        // Remove finished sounds
        self.sounds.retain(|sound| !sound.is_finished(self.current_time) && sound.is_active);

        // Update spatial audio (distance attenuation)
        for sound in &mut self.sounds {
            let distance = self.listener_position.distance(sound.position);
            let max_distance = 300.0;

            if distance > max_distance {
                sound.intensity = 0.0;
            } else {
                let attenuation = 1.0 - (distance / max_distance);
                sound.intensity = sound.intensity * attenuation;
            }
        }
    }

    fn get_active_sounds(&self) -> Vec<&ReactionSound> {
        self.sounds.iter().filter(|s| s.is_active).collect()
    }
}

#[derive(Clone, Debug)]
struct AtomicParticle {
    position: Vec2,
    velocity: Vec2,
    element: Element,
    lifetime: f32,
    id: u32,
}

impl AtomicParticle {
    fn new(position: Vec2, velocity: Vec2, element: Element, id: u32) -> Self {
        Self {
            position,
            velocity,
            element,
            lifetime: rand::random::<f32>() * 2.0 + 1.0,
            id,
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        // Apply gravity
        self.velocity.y -= 300.0 * dt;

        // Update position
        self.position += self.velocity * dt;

        // Update lifetime
        self.lifetime -= dt;

        // Boundary collision
        if self.position.y < 50.0 {
            self.position.y = 50.0;
            self.velocity.y *= -0.5;
            self.velocity.x *= 0.8;
        }

        if self.position.x < 0.0 || self.position.x > 800.0 {
            self.velocity.x *= -0.8;
            self.position.x = self.position.x.clamp(0.0, 800.0);
        }

        self.lifetime > 0.0
    }
}

#[derive(Clone, Debug)]
struct AtomicPhysics {
    particles: Vec<AtomicParticle>,
    next_particle_id: u32,
}

impl AtomicPhysics {
    fn new() -> Self {
        Self {
            particles: Vec::new(),
            next_particle_id: 0,
        }
    }

    fn add_particle(&mut self, position: Vec2, velocity: Vec2, element: Element) {
        let particle = AtomicParticle::new(position, velocity, element, self.next_particle_id);
        self.particles.push(particle);
        self.next_particle_id += 1;
    }

    fn update(&mut self, dt: f32, audio_engine: &mut ProceduralAudioEngine) {
        // Update particles
        self.particles.retain_mut(|particle| {
            let alive = particle.update(dt);
            if !alive {
                // Particle died, maybe play a sound
                if rand::random::<f32>() < 0.3 {
                    audio_engine.play_reaction_sound(
                        particle.position,
                        vec![particle.element.clone()],
                        0.5,
                    );
                }
            }
            alive
        });

        // Check for reactions between particles
        self.check_reactions(audio_engine);
    }

    fn check_reactions(&mut self, audio_engine: &mut ProceduralAudioEngine) {
        let mut reactions = Vec::new();

        for i in 0..self.particles.len() {
            for j in (i + 1)..self.particles.len() {
                if let (Some(p1), Some(p2)) = (self.particles.get(i), self.particles.get(j)) {
                    let distance = p1.position.distance(p2.position);

                    if distance < 20.0 {
                        // Check for chemical reactions
                        let reaction = self.get_reaction(&p1.element, &p2.element);
                        if let Some((new_elements, intensity)) = reaction {
                            reactions.push((p1.position, new_elements, intensity, vec![i, j]));
                        }
                    }
                }
            }
        }

        // Process reactions
        for (position, new_elements, intensity, particle_indices) in reactions {
            // Clone elements for audio so we can still iterate them below
            audio_engine.play_reaction_sound(position, new_elements.clone(), intensity);

            // Remove reacting particles and add new ones
            for &index in &particle_indices {
                // Clone particle data so we can mutably borrow self later
                if let Some(particle) = self.particles.get(index).cloned() {
                    // Create new particles based on the reaction result
                    for new_element in &new_elements {
                        self.add_particle(
                            particle.position + Vec2::new(rand::random::<f32>() * 10.0 - 5.0, 0.0),
                            particle.velocity * 0.5
                                + Vec2::new(
                                    rand::random::<f32>() * 50.0 - 25.0,
                                    rand::random::<f32>() * 50.0,
                                ),
                            new_element.clone(),
                        );
                    }
                }
            }

            // Remove old particles (simplified)
            for &index in particle_indices.iter().rev() {
                if index < self.particles.len() {
                    self.particles.swap_remove(index);
                }
            }
        }
    }

    fn get_reaction(&self, elem1: &Element, elem2: &Element) -> Option<(Vec<Element>, f32)> {
        match (elem1, elem2) {
            (Element::Fire, Element::Water) => Some((vec![Element::Steam], 1.0)),
            (Element::Acid, Element::Metal) => Some((vec![Element::Electricity], 0.8)),
            (Element::Electricity, Element::Water) => Some((vec![Element::Steam, Element::Electricity], 0.9)),
            (Element::Fire, Element::Sand) => Some((vec![Element::Stone], 0.6)),
            _ => None,
        }
    }
}

#[derive(Resource)]
struct AtomicReactionDemo {
    physics: AtomicPhysics,
    audio_engine: ProceduralAudioEngine,
    current_time: f64,
}

impl AtomicReactionDemo {
    fn new() -> Self {
        Self {
            physics: AtomicPhysics::new(),
            audio_engine: ProceduralAudioEngine::new(),
            current_time: 0.0,
        }
    }

    fn update(&mut self, dt: f64) {
        self.current_time += dt;
        self.audio_engine.update(dt);
        self.physics.update(dt as f32, &mut self.audio_engine);
        self.audio_engine.listener_position = Vec2::new(400.0, 300.0);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Atomic Reaction Sounds - Procedural Audio for Physics".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(AtomicReactionDemo::new())
        .add_systems(Startup, setup_atomic_demo)
        .add_systems(Update, (
            handle_atomic_input,
            update_atomic_demo,
            render_atomic_demo,
            display_atomic_info,
        ).chain())
        .run();
}

fn setup_atomic_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_atomic_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<AtomicReactionDemo>,
) {
    demo.update(1.0 / 60.0);

    // Get mouse position
    let mouse_pos = if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
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

    // Add particles based on input
    if mouse_input.pressed(MouseButton::Left) {
        let velocity = Vec2::new((rand::random::<f32>() - 0.5) * 200.0, rand::random::<f32>() * 100.0);
        let element = match rand::random::<u32>() % 8 {
            0 => Element::Sand,
            1 => Element::Water,
            2 => Element::Fire,
            3 => Element::Stone,
            4 => Element::Acid,
            5 => Element::Metal,
            6 => Element::Electricity,
            _ => Element::Steam,
        };
        demo.physics.add_particle(mouse_pos, velocity, element);
    }

    // Trigger specific reactions
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Fire + Water reaction
        demo.physics.add_particle(Vec2::new(300.0, 200.0), Vec2::new(50.0, 0.0), Element::Fire);
        demo.physics.add_particle(Vec2::new(320.0, 200.0), Vec2::new(-50.0, 0.0), Element::Water);
    }

    if keyboard_input.just_pressed(KeyCode::KeyE) {
        // Acid + Metal reaction
        demo.physics.add_particle(Vec2::new(500.0, 200.0), Vec2::new(0.0, 50.0), Element::Acid);
        demo.physics.add_particle(Vec2::new(500.0, 220.0), Vec2::new(0.0, -50.0), Element::Metal);
    }
}

fn update_atomic_demo(time: Res<Time>, mut demo: ResMut<AtomicReactionDemo>) {
    // Updates are handled in input system
}

fn render_atomic_demo(
    mut commands: Commands,
    mut particle_entities: Local<Vec<Entity>>,
    mut sound_entities: Local<Vec<Entity>>,
    demo: Res<AtomicReactionDemo>,
) {
    // Clear previous frame
    for entity in particle_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in sound_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render ground
    let ground_entity = commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::srgb(0.4, 0.4, 0.4),
            custom_size: Some(Vec2::new(800.0, 20.0)),
            ..default()
        },
        transform: Transform::from_xyz(400.0, 30.0, 0.0),
        ..default()
    }).id();
    particle_entities.push(ground_entity);

    // Render particles
    let element_colors = [
        (Element::Sand, Color::srgb(0.9, 0.8, 0.6)),
        (Element::Water, Color::srgba(0.3, 0.5, 0.9, 0.8)),
        (Element::Fire, Color::srgb(1.0, 0.3, 0.0)),
        (Element::Steam, Color::srgba(0.9, 0.9, 0.9, 0.5)),
        (Element::Stone, Color::srgb(0.5, 0.5, 0.5)),
        (Element::Acid, Color::srgb(0.8, 1.0, 0.2)),
        (Element::Metal, Color::srgb(0.7, 0.7, 0.8)),
        (Element::Electricity, Color::srgb(1.0, 1.0, 0.0)),
    ];

    for particle in &demo.physics.particles {
        let color = element_colors.iter()
            .find(|(elem, _)| elem == &particle.element)
            .map(|(_, color)| *color)
            .unwrap_or(Color::WHITE);

        let size = 6.0 + particle.lifetime * 2.0;

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                transform: Transform::from_xyz(particle.position.x, particle.position.y, 1.0),
                ..default()
            },

        )).id();
        particle_entities.push(entity);
    }

    // Render active reaction sounds
    let active_sounds = demo.audio_engine.get_active_sounds();
    for (i, sound) in active_sounds.iter().enumerate() {
        let intensity = sound.intensity;
        let size = 20.0 + intensity * 30.0;

        let sound_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(1.0, 0.5, 0.0, intensity * 0.7),
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                transform: Transform::from_xyz(sound.position.x, sound.position.y + 50.0 + i as f32 * 25.0, 2.0),
                ..default()
            },

        )).id();
        sound_entities.push(sound_entity);
    }
}

fn display_atomic_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<AtomicReactionDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        let active_sounds = demo.audio_engine.get_active_sounds();

        println!("\n=== Atomic Reaction Sounds Demo ===");
        println!("Particles: {}", demo.physics.particles.len());
        println!("Active Reaction Sounds: {}", active_sounds.len());
        println!("Master Volume: {:.1}", demo.audio_engine.master_volume);

        println!("\nParticle Distribution:");
        let mut element_counts = std::collections::HashMap::new();
        for particle in &demo.physics.particles {
            *element_counts.entry(format!("{:?}", particle.element)).or_insert(0) += 1;
        }
        for (element, count) in element_counts {
            println!("  {}: {}", element, count);
        }

        println!("\nActive Reactions:");
        for sound in &active_sounds {
            println!("  ID {}: {:?} at ({:.1}, {:.1}), intensity: {:.2}",
                    sound.id, sound.elements, sound.position.x, sound.position.y, sound.intensity);
        }

        println!("\nChemical Reactions Available:");
        println!("  Fire + Water → Steam");
        println!("  Acid + Metal → Electricity");
        println!("  Electricity + Water → Steam + Electricity");
        println!("  Fire + Sand → Stone");

        println!("\nControls:");
        println!("  Left Click: Add random particles");
        println!("  R: Trigger Fire + Water reaction");
        println!("  E: Trigger Acid + Metal reaction");
        println!("  I: Show this info");
        println!("\nNote: Audio is procedural - sounds based on element properties!");
        println!("======================\n");
    }
}
