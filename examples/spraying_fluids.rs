// Example: Spraying Fluids
// Based on "Spraying Fluids" blog post
// https://www.slowrush.dev/news/spraying-fluids

use bevy::prelude::*;
use bevy::tasks::futures_lite::StreamExt;
use bevy_rapier2d::prelude::*;

// Fluid spraying and particle systems
// Demonstrates fluid dynamics, spraying mechanics, and particle effects

#[derive(Clone, Debug)]
struct FluidParticle {
    position: Vec2,
    velocity: Vec2,
    element: FluidElement,
    lifetime: f32,
    max_lifetime: f32,
    size: f32,
    id: u32,
}

#[derive(Clone, Debug,Eq, Hash, PartialEq)]
enum FluidElement {
    Water,
    Oil,
    Acid,
    Fuel,
    Steam,
    Fire,
}

impl FluidElement {
    fn color(&self) -> Color {
        match self {
            FluidElement::Water => Color::rgba(0.3, 0.5, 0.9, 0.8),
            FluidElement::Oil => Color::rgba(0.4, 0.3, 0.1, 0.9),
            FluidElement::Acid => Color::rgba(0.8, 1.0, 0.2, 0.7),
            FluidElement::Fuel => Color::rgba(1.0, 0.6, 0.0, 0.8),
            FluidElement::Steam => Color::rgba(0.9, 0.9, 0.9, 0.5),
            FluidElement::Fire => Color::rgba(1.0, 0.3, 0.0, 0.9),
        }
    }

    fn density(&self) -> f32 {
        match self {
            FluidElement::Water => 1.0,
            FluidElement::Oil => 0.8,
            FluidElement::Acid => 1.2,
            FluidElement::Fuel => 0.7,
            FluidElement::Steam => 0.1,
            FluidElement::Fire => 0.05,
        }
    }

    fn viscosity(&self) -> f32 {
        match self {
            FluidElement::Water => 1.0,
            FluidElement::Oil => 2.0,
            FluidElement::Acid => 1.5,
            FluidElement::Fuel => 0.5,
            FluidElement::Steam => 0.1,
            FluidElement::Fire => 0.05,
        }
    }

    fn volatility(&self) -> f32 {
        match self {
            FluidElement::Water => 0.0,
            FluidElement::Oil => 0.1,
            FluidElement::Acid => 0.3,
            FluidElement::Fuel => 0.8,
            FluidElement::Steam => 0.0, // Already vapor
            FluidElement::Fire => 1.0,
        }
    }
}

impl FluidParticle {
    fn new(position: Vec2, velocity: Vec2, element: FluidElement, id: u32) -> Self {
        let max_lifetime = match element {
            FluidElement::Steam => 2.0,
            FluidElement::Fuel => 5.0,
            _ => 10.0,
        };

        Self {
            position,
            velocity,
            element: element.clone(),
            lifetime: max_lifetime,
            max_lifetime,
            size: 3.0 + rand::random::<f32>() * 2.0,
            id,
        }
    }

    fn update(&mut self, dt: f32, fluid_system: &FluidSystem) {
        // Apply gravity (less for steam)
        let gravity = if self.element == FluidElement::Steam { 100.0 } else { 300.0 };
        self.velocity.y -= gravity * dt;

        // Apply air resistance based on viscosity
        let drag = 0.98 - (self.element.viscosity() * 0.01);
        self.velocity *= drag;

        // Update position
        self.position += self.velocity * dt;

        // Handle collisions with environment
        if self.position.y <= 50.0 {
            self.position.y = 50.0;
            self.velocity.y *= -0.3; // Bounce
            self.velocity.x *= 0.8; // Friction
        }

        // Handle screen boundaries
        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.x *= -0.5;
        }
        if self.position.x > 800.0 {
            self.position.x = 800.0;
            self.velocity.x *= -0.5;
        }

        // Lifetime and evaporation
        self.lifetime -= dt;

        // Volatility affects evaporation
        if rand::random::<f32>() < self.element.volatility() * dt {
            self.lifetime -= dt * 2.0; // Faster evaporation
        }

        // Fluid interactions
        self.handle_fluid_interactions(fluid_system, dt);
    }

    fn handle_fluid_interactions(&mut self, fluid_system: &FluidSystem, dt: f32) {
        // Find nearby particles
        let nearby_particles = fluid_system.get_nearby_particles(self.position, 20.0);

        for &(other_id, other_pos) in &nearby_particles {
            if other_id == self.id {
                continue;
            }

            let distance = self.position.distance(other_pos);
            if distance < 10.0 && distance > 0.1 {
                // Cohesion and separation forces
                let direction = (other_pos - self.position).normalize();
                let force = 50.0 / (distance * distance); // Inverse square law

                // Different fluids interact differently
                let interaction_strength = match (&self.element, fluid_system.get_particle_element(other_id)) {
                    (FluidElement::Water, FluidElement::Oil) => 0.5, // Partial mixing
                    (FluidElement::Acid, FluidElement::Water) => 2.0, // Chemical reaction
                    (FluidElement::Fuel, FluidElement::Fuel) => 1.5, // Cohesion
                    _ => 1.0,
                };

                self.velocity += direction * force * interaction_strength * dt;

                // Chemical reactions
                if let Some(reaction) = self.check_chemical_reaction(fluid_system.get_particle_element(other_id)) {
                    if rand::random::<f32>() < 0.1 * dt {
                        fluid_system.create_particles_at(self.position, reaction, 2);
                    }
                }
            }
        }
    }

    fn check_chemical_reaction(&self, other_element: FluidElement) -> Option<FluidElement> {
        match (&self.element, other_element) {
            (FluidElement::Acid, FluidElement::Water) => Some(FluidElement::Steam), // Acid + Water = Steam
            (FluidElement::Fuel, FluidElement::Fire) => Some(FluidElement::Steam), // Fuel burns
            _ => None,
        }
    }

    fn is_alive(&self) -> bool {
        self.lifetime > 0.0 && self.position.y > -50.0
    }
}

#[derive(Clone, Debug)]
struct FluidSprayer {
    position: Vec2,
    direction: Vec2,
    spray_angle: f32,
    particle_speed: f32,
    spray_rate: f32,
    element: FluidElement,
    is_active: bool,
    spray_timer: f32,
}

impl FluidSprayer {
    fn new(position: Vec2, element: FluidElement) -> Self {
        Self {
            position,
            direction: Vec2::new(1.0, 0.0),
            spray_angle: std::f32::consts::PI / 6.0, // 30 degrees
            particle_speed: 200.0,
            spray_rate: 0.05, // Particles per second
            element,
            is_active: false,
            spray_timer: 0.0,
        }
    }

    fn update(&mut self, dt: f32, fluid_system: &mut FluidSystem) {
        self.spray_timer += dt;

        if self.is_active && self.spray_timer >= self.spray_rate {
            self.spray_timer = 0.0;
            self.emit_particles(fluid_system);
        }
    }

    fn emit_particles(&self, fluid_system: &mut FluidSystem) {
        let particle_count = 3 + (rand::random::<f32>() * 3.0) as usize;

        for _ in 0..particle_count {
            // Random angle within spray cone
            let angle_offset = (rand::random::<f32>() - 0.5) * self.spray_angle;
            let angle = self.direction.angle_between(Vec2::X) + angle_offset;
            let direction = Vec2::new(angle.cos(), angle.sin());

            // Random speed variation
            let speed = self.particle_speed * (0.8 + rand::random::<f32>() * 0.4);
            let velocity = direction * speed;
            let next_particle_id=fluid_system.next_particle_id();
            fluid_system.add_particle(FluidParticle::new(
                self.position,
                velocity,
                self.element.clone(),
                next_particle_id,
            ));
        }
    }

    fn set_direction(&mut self, direction: Vec2) {
        self.direction = direction.normalize();
    }

    fn move_to(&mut self, position: Vec2) {
        self.position = position;
    }
}

#[derive(Clone, Debug)]
struct FluidSystem {
    particles: Vec<FluidParticle>,
    next_id: u32,
}

impl FluidSystem {
    fn new() -> Self {
        Self {
            particles: Vec::new(),
            next_id: 0,
        }
    }

    fn next_particle_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id - 1
    }

    fn add_particle(&mut self, particle: FluidParticle) {
        self.particles.push(particle);
    }

    fn create_particles_at(&self, position: Vec2, element: FluidElement, count: usize) {
        // This would normally add particles, but we're immutable here
        // In practice, this would be handled differently
    }

    fn get_nearby_particles(&self, position: Vec2, radius: f32) -> Vec<(u32, Vec2)> {
        self.particles.iter()
            .filter(|p| p.position.distance(position) <= radius)
            .map(|p| (p.id, p.position))
            .collect()
    }

    fn get_particle_element(&self, id: u32) -> FluidElement {
        self.particles.iter()
            .find(|p| p.id == id)
            .map(|p| p.element.clone())
            .unwrap_or(FluidElement::Water)
    }

    fn update(&mut self, dt: f32) {
        // Update all particles
        // for i in 0..self.particles.len() {
        //     if let Some(particle) = self.particles.get_mut(i) {
        //         particle.update(dt, self);
        //     }
        // }
        //
        // // Remove dead particles
        // self.particles.retain(|p| p.is_alive());
        let mut i = 0;
        while i < self.particles.len() {
            let mut particle = self.particles.remove(i);
            particle.update(dt, self);

            if particle.is_alive() {
                // Reinsert the updated atom and advance.
                self.particles.insert(i, particle);
                i += 1;
            }
            // Dead atoms are dropped; newly spawned atoms (via add_atom) are appended.
        }
        // Limit particle count for performance
        if self.particles.len() > 1000 {
            self.particles.truncate(800);
        }
    }

    fn get_particle_stats(&self) -> std::collections::HashMap<FluidElement, usize> {
        let mut stats = std::collections::HashMap::new();
        for particle in &self.particles {
            *stats.entry(particle.element.clone()).or_insert(0) += 1;
        }
        stats
    }
}

#[derive(Resource)]
struct FluidSprayDemo {
    fluid_system: FluidSystem,
    sprayers: Vec<FluidSprayer>,
    selected_element: FluidElement,
    mouse_position: Vec2,
}

impl FluidSprayDemo {
    fn new() -> Self {
        let mut sprayers = Vec::new();

        // Create different sprayers for different fluids
        sprayers.push(FluidSprayer::new(Vec2::new(200.0, 500.0), FluidElement::Water));
        sprayers.push(FluidSprayer::new(Vec2::new(400.0, 500.0), FluidElement::Oil));
        sprayers.push(FluidSprayer::new(Vec2::new(600.0, 500.0), FluidElement::Acid));

        Self {
            fluid_system: FluidSystem::new(),
            sprayers,
            selected_element: FluidElement::Water,
            mouse_position: Vec2::ZERO,
        }
    }

    fn update(&mut self, dt: f32) {
        // Update sprayers
        for sprayer in &mut self.sprayers {
            sprayer.update(dt, &mut self.fluid_system);
        }

        // Update fluid system
        self.fluid_system.update(dt);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spraying Fluids - Particle Systems & Fluid Dynamics".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(FluidSprayDemo::new())
        .add_systems(Startup, setup_fluid_spray_demo)
        .add_systems(Update, (
            handle_fluid_spray_input,
            update_fluid_spray_simulation,
            render_fluid_spray_demo,
            display_fluid_spray_info,
        ).chain())
        .run();
}

fn setup_fluid_spray_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_fluid_spray_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut demo: ResMut<FluidSprayDemo>,
) {
    // Update mouse position
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    demo.mouse_position = world_pos.origin.truncate();
                }
            }
        }
    }

    // Select fluid type
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        demo.selected_element = FluidElement::Water;
        println!("Selected: Water");
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        demo.selected_element = FluidElement::Oil;
        println!("Selected: Oil");
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        demo.selected_element = FluidElement::Acid;
        println!("Selected: Acid");
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        demo.selected_element = FluidElement::Fuel;
        println!("Selected: Fuel");
    }

    // Control sprayers
    if keyboard_input.pressed(KeyCode::KeyQ) {
        if let Some(sprayer) = demo.sprayers.get_mut(0) {
            sprayer.is_active = true;
        }
    } else {
        if let Some(sprayer) = demo.sprayers.get_mut(0) {
            sprayer.is_active = false;
        }
    }

    if keyboard_input.pressed(KeyCode::KeyW) {
        if let Some(sprayer) = demo.sprayers.get_mut(1) {
            sprayer.is_active = true;
        }
    } else {
        if let Some(sprayer) = demo.sprayers.get_mut(1) {
            sprayer.is_active = false;
        }
    }

    if keyboard_input.pressed(KeyCode::KeyE) {
        if let Some(sprayer) = demo.sprayers.get_mut(2) {
            sprayer.is_active = true;
        }
    } else {
        if let Some(sprayer) = demo.sprayers.get_mut(2) {
            sprayer.is_active = false;
        }
    }
    let mouse_position = demo.mouse_position.clone();
    let selected_element =demo.selected_element.clone();
    let fluid_system =demo.fluid_system.next_particle_id();
    // Manual particle creation
    if mouse_input.pressed(MouseButton::Left) {
        let velocity = (mouse_position - Vec2::new(400.0, 300.0)).normalize() * 150.0;
        demo.fluid_system.add_particle(FluidParticle::new(
            mouse_position,
            velocity,
            selected_element,
            fluid_system,
        ));
    }

    // Clear particles
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        demo.fluid_system.particles.clear();
        println!("Cleared all particles");
    }

    // Adjust sprayer directions toward mouse
    for sprayer in &mut demo.sprayers {
        let direction = (mouse_position - sprayer.position).normalize();
        sprayer.set_direction(direction);
    }
}

fn update_fluid_spray_simulation(time: Res<Time>, mut demo: ResMut<FluidSprayDemo>) {
    demo.update(time.delta_seconds().min(1.0 / 30.0));
}

fn render_fluid_spray_demo(
    mut commands: Commands,
    mut particle_entities: Local<Vec<Entity>>,
    mut sprayer_entities: Local<Vec<Entity>>,
    demo: Res<FluidSprayDemo>,
) {
    // Clear previous frame
    for entity in particle_entities.drain(..) {
        commands.entity(entity).despawn();
    }
    for entity in sprayer_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render particles
    for particle in &demo.fluid_system.particles {
        let alpha = particle.lifetime / particle.max_lifetime;
        let mut color = particle.element.color();
        color.set_alpha(color.alpha() * alpha);

        let entity = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(particle.size, particle.size)),
                ..default()
            },
            transform: Transform::from_xyz(particle.position.x, particle.position.y, 0.0),
            ..default()
        }).id();
        particle_entities.push(entity);
    }

    // Render sprayers
    for (i, sprayer) in demo.sprayers.iter().enumerate() {
        let color = sprayer.element.color();
        let status_color = if sprayer.is_active {
            Color::rgb(0.0, 1.0, 0.0) // Green when active
        } else {
            Color::rgb(0.5, 0.5, 0.5) // Gray when inactive
        };

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(20.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(sprayer.position.x, sprayer.position.y, 1.0),
                ..default()
            },

        )).id();
        sprayer_entities.push(entity);
    }
}

fn display_fluid_spray_info(keyboard_input: Res<ButtonInput<KeyCode>>, demo: Res<FluidSprayDemo>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        let stats = demo.fluid_system.get_particle_stats();

        println!("\n=== Spraying Fluids Demo ===");
        println!("Total Particles: {}", demo.fluid_system.particles.len());
        println!("Particle Distribution:");

        for (element, count) in stats {
            println!("  {:?}: {}", element, count);
        }

        println!("\nSprayers:");
        for (i, sprayer) in demo.sprayers.iter().enumerate() {
            println!("  {}: {:?} at ({:.1}, {:.1}) - {}",
                    i + 1,
                    sprayer.element,
                    sprayer.position.x,
                    sprayer.position.y,
                    if sprayer.is_active { "ACTIVE" } else { "INACTIVE" });
        }

        println!("\nControls:");
        println!("  1-4: Select fluid type");
        println!("  Q/W/E: Toggle sprayers 1/2/3");
        println!("  Left Click: Manual particle creation");
        println!("  C: Clear all particles");
        println!("  H: Show this info");
        println!("\nSelected Element: {:?}", demo.selected_element);
        println!("======================\n");
    }
}
