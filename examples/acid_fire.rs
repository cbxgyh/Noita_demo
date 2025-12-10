// Example: Acid's Ire and Burning Fire
// Based on "Acid's Ire and Burning Fire" blog post
// https://www.slowrush.dev/news/acids-ire-and-burning-fire

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Acid and fire elements with chemical reactions
// Demonstrates reactive atoms that can corrode terrain and burn things

#[derive(Clone, Debug,PartialEq,Eq,PartialOrd,Hash,Copy)]
enum Element {
    Empty,
    Stone,
    Sand,
    Water,
    Acid,
    Fire,
    Smoke,
}

impl Element {
    fn color(&self) -> Color {
        match self {
            Element::Empty => Color::srgba(0.0, 0.0, 0.0, 0.0),
            Element::Stone => Color::srgb(0.4, 0.4, 0.4),
            Element::Sand => Color::srgb(0.8, 0.7, 0.5),
            Element::Water => Color::srgba(0.2, 0.4, 0.8, 0.8),
            Element::Acid => Color::srgb(0.0, 0.8, 0.0),
            Element::Fire => Color::srgb(1.0, 0.3, 0.0),
            Element::Smoke => Color::srgba(0.3, 0.3, 0.3, 0.5),
        }
    }

    fn temperature(&self) -> f32 {
        match self {
            Element::Empty => 20.0,
            Element::Stone => 20.0,
            Element::Sand => 20.0,
            Element::Water => 20.0,
            Element::Acid => 25.0,
            Element::Fire => 800.0,
            Element::Smoke => 100.0,
        }
    }

    fn flammability(&self) -> f32 {
        match self {
            Element::Empty => 0.0,
            Element::Stone => 0.0,
            Element::Sand => 0.1, // Can burn but slowly
            Element::Water => 0.0, // Extinguishes fire
            Element::Acid => 0.0,
            Element::Fire => 1.0,
            Element::Smoke => 0.0,
        }
    }

    fn corrosive(&self) -> bool {
        matches!(self, Element::Acid)
    }

    fn burns(&self) -> bool {
        matches!(self, Element::Fire)
    }

    fn can_burn(&self) -> bool {
        self.flammability() > 0.0
    }
}

#[derive(Clone, Debug)]
struct ReactiveAtom {
    element: Element,
    position: Vec2,
    velocity: Vec2,
    temperature: f32,
    health: f32, // For corrosion and burning
    lifetime: Option<f32>,
}

impl ReactiveAtom {
    fn new(element: Element, position: Vec2) -> Self {
        Self {
            element: element.clone(),
            position,
            velocity: Vec2::ZERO,
            temperature: element.temperature(),
            health: 100.0,
            lifetime: None,
        }
    }

    fn update(&mut self, dt: f32, world_bounds: Vec2) {
        if self.element == Element::Empty {
            return;
        }

        // Apply gravity to non-gas elements
        if !matches!(self.element, Element::Smoke) {
            self.velocity.y -= 30.0 * dt;
        }

        // Gas elements rise
        if self.element == Element::Smoke {
            self.velocity.y += 10.0 * dt;
        }

        // Apply drag
        self.velocity *= 0.98;

        // Update position
        self.position += self.velocity * dt;

        // Bounds checking
        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.x *= -0.5;
        } else if self.position.x >= world_bounds.x {
            self.position.x = world_bounds.x - 1.0;
            self.velocity.x *= -0.5;
        }

        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y *= -0.5;
        } else if self.position.y >= world_bounds.y {
            self.position.y = world_bounds.y - 1.0;
            self.velocity.y *= -0.5;
        }

        // Update lifetime
        if let Some(ref mut lifetime) = self.lifetime {
            *lifetime -= dt;
            if *lifetime <= 0.0 {
                self.element = Element::Empty;
            }
        }

        // Temperature effects
        self.handle_temperature_effects(dt);
    }

    fn handle_temperature_effects(&mut self, dt: f32) {
        // Fire spreads and damages
        if self.element == Element::Fire {
            self.health -= 20.0 * dt; // Fire consumes itself
            if self.health <= 0.0 {
                // Fire burns out, leaves smoke
                self.element = Element::Smoke;
                self.lifetime = Some(3.0);
                self.temperature = 100.0;
            }
        }

        // Acid corrodes
        if self.element == Element::Acid {
            // Acid slowly loses potency
            self.health -= 5.0 * dt;
            if self.health <= 0.0 {
                self.element = Element::Water; // Acid becomes water
            }
        }
    }

    fn react_with(&mut self, other: &mut ReactiveAtom, dt: f32) -> Vec<ReactionProduct> {
        let mut products = Vec::new();

        match (&self.element, &other.element) {
            // Fire + Water = Steam + Smoke
            (Element::Fire, Element::Water) | (Element::Water, Element::Fire) => {
                products.push(ReactionProduct {
                    element: Element::Smoke,
                    position: (self.position + other.position) / 2.0,
                    velocity: Vec2::new(0.0, 20.0), // Steam rises
                    lifetime: Some(2.0),
                });

                // Extinguish fire
                if self.element == Element::Fire {
                    self.element = Element::Empty;
                }
                if other.element == Element::Fire {
                    other.element = Element::Empty;
                }

                println!("Fire extinguished by water!");
            }

            // Acid + Stone = Corrosion
            (Element::Acid, Element::Stone) | (Element::Stone, Element::Acid) => {
                // Acid corrodes stone over time
                if self.element == Element::Stone {
                    self.health -= 15.0 * dt;
                    if self.health <= 0.0 {
                        self.element = Element::Sand; // Stone becomes sand
                        self.health = 50.0;
                    }
                }
                if other.element == Element::Stone {
                    other.health -= 15.0 * dt;
                    if other.health <= 0.0 {
                        other.element = Element::Sand;
                        other.health = 50.0;
                    }
                }

                // Acid loses some potency
                if self.element == Element::Acid {
                    self.health -= 10.0 * dt;
                }
                if other.element == Element::Acid {
                    other.health -= 10.0 * dt;
                }

                println!("Acid corroding stone!");
            }

            // Fire + Sand = More fire
            (Element::Fire, Element::Sand) | (Element::Sand, Element::Fire) => {
                if rand::random::<f32>() < 0.1 * dt * 60.0 { // Chance per frame
                    // Sand catches fire
                    if self.element == Element::Sand {
                        self.element = Element::Fire;
                        self.temperature = 600.0;
                        self.lifetime = Some(5.0);
                    }
                    if other.element == Element::Sand {
                        other.element = Element::Fire;
                        other.temperature = 600.0;
                        other.lifetime = Some(5.0);
                    }

                    println!("Sand ignites!");
                }
            }

            // Acid + Sand = Dissolves faster
            (Element::Acid, Element::Sand) | (Element::Sand, Element::Acid) => {
                // Acid dissolves sand quickly
                if self.element == Element::Sand {
                    self.health -= 25.0 * dt;
                    if self.health <= 0.0 {
                        self.element = Element::Empty;
                    }
                }
                if other.element == Element::Sand {
                    other.health -= 25.0 * dt;
                    if other.health <= 0.0 {
                        other.element = Element::Empty;
                    }
                }

                println!("Acid dissolving sand!");
            }

            _ => {}
        }

        products
    }
}

#[derive(Clone, Debug)]
struct ReactionProduct {
    element: Element,
    position: Vec2,
    velocity: Vec2,
    lifetime: Option<f32>,
}

struct ReactiveWorld {
    width: usize,
    height: usize,
    atoms: Vec<ReactiveAtom>,
    bounds: Vec2,
    reaction_cooldown: f32,
}

impl ReactiveWorld {
    fn new(width: usize, height: usize) -> Self {
        let mut atoms = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let mut element = Element::Empty;

                // Create terrain
                if y < 10 {
                    element = Element::Stone;
                } else if y < 20 && x > width / 2 - 10 && x < width / 2 + 10 {
                    element = Element::Sand;
                }

                atoms.push(ReactiveAtom::new(element, Vec2::new(x as f32, y as f32)));
            }
        }

        Self {
            width,
            height,
            atoms,
            bounds: Vec2::new(width as f32, height as f32),
            reaction_cooldown: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.reaction_cooldown -= dt;

        // Update all atoms
        for atom in &mut self.atoms {
            atom.update(dt, self.bounds);
        }

        // Handle reactions
        if self.reaction_cooldown <= 0.0 {
            self.process_reactions(dt);
            self.reaction_cooldown = 0.05; // Process reactions every 50ms
        }

        // Clean up dead atoms
        for atom in &mut self.atoms {
            if atom.element != Element::Empty && atom.health <= 0.0 {
                match atom.element {
                    Element::Fire => {
                        // Fire leaves smoke
                        atom.element = Element::Smoke;
                        atom.lifetime = Some(2.0);
                        atom.temperature = 80.0;
                    }
                    Element::Acid => {
                        // Acid becomes water
                        atom.element = Element::Water;
                    }
                    _ => {
                        atom.element = Element::Empty;
                    }
                }
            }
        }
    }

    fn process_reactions(&mut self, dt: f32) {
        let mut reaction_products = Vec::new();

        // Check each atom against its neighbors 将每个原子与其邻居进行检查
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;

                if self.atoms[idx].element == Element::Empty {
                    continue;
                }

                // Check neighboring atoms 检查邻近原子
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 { continue; }

                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                            let nidx = ny as usize * self.width + nx as usize;

                            if self.atoms[nidx].element != Element::Empty {
                                // Use split_at_mut to get two mutable references
                                // Only process when idx < nidx to avoid processing the same pair twice
                                // and to ensure we can split the slice correctly
                                // 使用 split_at_mut 获取两个可变引用  仅在 idx < nidx 时处理，以避免重复处理相同的元素对，并确保我们可以正确地拆分切片
                                //
                                //  let products = self.atoms[idx].react_with(&mut self.atoms[nidx], dt); ##
                                //  reaction_products.extend(products);
                                if idx < nidx {
                                    let (left, right) = self.atoms.split_at_mut(nidx);
                                    let products = left[idx].react_with(&mut right[0], dt);
                                    reaction_products.extend(products);
                                }
                                // If idx > nidx, this pair will be processed when we iterate to nidx
                                // idx == nidx should never happen due to the dx == 0 && dy == 0 check
                            //     如果 idx > nidx，当我们迭代到 nidx 时，这对将被处理。由于 dx == 0 && dy == 0 检查，idx == nidx 不应发生。
                            }
                        }
                    }
                }
            }
        }

        // Add reaction products
        for product in reaction_products {
            self.add_atom_at_position(product.element, product.position, product.velocity, product.lifetime);
        }
    }

    fn add_atom_at_position(&mut self, element: Element, position: Vec2, velocity: Vec2, lifetime: Option<f32>) {
        let x = position.x.round() as usize;
        let y = position.y.round() as usize;

        if x < self.width && y < self.height {
            let idx = y * self.width + x;

            // Only place if empty or replacing with more reactive element
            if self.atoms[idx].element == Element::Empty ||
               (element == Element::Fire && self.atoms[idx].element.can_burn()) {

                self.atoms[idx] = ReactiveAtom::new(element, position);
                self.atoms[idx].velocity = velocity;
                self.atoms[idx].lifetime = lifetime;

                match element {
                    Element::Fire => {
                        self.atoms[idx].temperature = 700.0;
                        self.atoms[idx].lifetime = Some(8.0);
                    }
                    Element::Smoke => {
                        self.atoms[idx].temperature = 100.0;
                    }
                    _ => {}
                }
            }
        }
    }

    fn add_element_at(&mut self, element: Element, position: Vec2, spread: f32, count: usize) {
        for _ in 0..count {
            let offset = Vec2::new(
                (rand::random::<f32>() - 0.5) * spread,
                (rand::random::<f32>() - 0.5) * spread,
            );
            let pos = position + offset;
            let velocity = Vec2::new(
                (rand::random::<f32>() - 0.5) * 20.0,
                rand::random::<f32>() * 10.0,
            );

            self.add_atom_at_position(element, pos, velocity, None);
        }
    }

    fn get_atom(&self, x: usize, y: usize) -> Option<&ReactiveAtom> {
        if x < self.width && y < self.height {
            Some(&self.atoms[y * self.width + x])
        } else {
            None
        }
    }
}

#[derive(Resource)]
struct ReactiveWorldResource(ReactiveWorld);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Acid's Ire and Burning Fire - Chemical Reactions".to_string(),
                resolution: (1000.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        .insert_resource(ReactiveWorldResource(ReactiveWorld::new(150, 100)))
        .add_systems(Startup, setup_reactive_demo)
        .add_systems(Update, (
            update_reactive_world,
            render_reactive_world,
            handle_reactive_input,
            demonstrate_chemical_reactions,
        ))
        .run();
}

fn setup_reactive_demo(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_reactive_world(mut world: ResMut<ReactiveWorldResource>, time: Res<Time>) {
    let dt = time.delta_seconds().min(1.0 / 30.0);
    world.0.update(dt);
}

fn render_reactive_world(
    mut commands: Commands,
    mut atom_entities: Local<Vec<Entity>>,
    world: Res<ReactiveWorldResource>,
) {
    // Clear previous frame
    for entity in atom_entities.drain(..) {
        commands.entity(entity).despawn();
    }

    // Render atoms
    for atom in &world.0.atoms {
        if atom.element != Element::Empty {
            let base_color = atom.element.color();
            let [r, g, b, a] = base_color.to_srgba().to_f32_array();

            // Show temperature with color intensity
            //  let temp_factor = (atom.temperature / 1000.0).min(1.0);
            //             color = color.with_r((color.r() + temp_factor * 0.5).min(1.0))
            //                         .with_g(color.g() * (1.0 - temp_factor * 0.3))
            //                         .with_b(color.b() * (1.0 - temp_factor * 0.5));
            let temp_factor = (atom.temperature / 1000.0).min(1.0);
            let new_r = (r + temp_factor * 0.5).min(1.0);
            let new_g = g * (1.0 - temp_factor * 0.3);
            let new_b = b * (1.0 - temp_factor * 0.5);

            // Show health/damage
            let health_factor = atom.health / 100.0;
            let new_a = a * health_factor;

            let color = Color::srgba(new_r, new_g, new_b, new_a);

            let entity = commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                transform: Transform::from_xyz(
                    atom.position.x - world.0.width as f32 / 2.0,
                    atom.position.y - world.0.height as f32 / 2.0,
                    0.0,
                ),
                ..default()
            }).id();
            atom_entities.push(entity);
        }
    }
}

fn handle_reactive_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut world: ResMut<ReactiveWorldResource>,
) {
    if let Ok((camera, camera_transform)) = camera_query.get_single() {
        if let Some(window) = windows.iter().next() {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    let atom_x = (world_pos.origin.x + world.0.width as f32 / 2.0) as f32;
                    let atom_y = (world_pos.origin.y + world.0.height as f32 / 2.0) as f32;
                    let position = Vec2::new(atom_x, atom_y);

                    // Add elements based on mouse buttons
                    if mouse_input.just_pressed(MouseButton::Left) {
                        world.0.add_element_at(Element::Fire, position, 5.0, 5);
                    }

                    if mouse_input.just_pressed(MouseButton::Right) {
                        world.0.add_element_at(Element::Acid, position, 5.0, 5);
                    }

                    if mouse_input.just_pressed(MouseButton::Middle) {
                        world.0.add_element_at(Element::Water, position, 5.0, 8);
                    }
                }
            }
        }
    }

    // Keyboard shortcuts
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        // Add fire
        world.0.add_element_at(Element::Fire, Vec2::new(50.0, 50.0), 3.0, 3);
    }

    if keyboard_input.just_pressed(KeyCode::KeyA) {
        // Add acid
        world.0.add_element_at(Element::Acid, Vec2::new(100.0, 50.0), 3.0, 3);
    }

    if keyboard_input.just_pressed(KeyCode::KeyW) {
        // Add water
        world.0.add_element_at(Element::Water, Vec2::new(75.0, 70.0), 5.0, 10);
    }

    if keyboard_input.just_pressed(KeyCode::KeyS) {
        // Add sand
        world.0.add_element_at(Element::Sand, Vec2::new(75.0, 30.0), 10.0, 20);
    }
}

fn demonstrate_chemical_reactions(keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        println!("\n=== Acid's Ire and Burning Fire Demo ===");
        println!("Chemical Reactions:");
        println!("- Fire + Water = Steam/Smoke (extinguishes fire)");
        println!("- Acid + Stone = Corrosion (stone becomes sand)");
        println!("- Fire + Sand = Ignition (sand catches fire)");
        println!("- Acid + Sand = Dissolution (sand dissolves)");
        println!("");
        println!("Visual Cues:");
        println!("- Bright colors = High temperature");
        println!("- Transparent = Damaged/Low health");
        println!("- Green = Acid, Red = Fire, Blue = Water");
        println!("");
        println!("Controls:");
        println!("Left click: Add Fire");
        println!("Right click: Add Acid");
        println!("Middle click: Add Water");
        println!("F: Add Fire | A: Add Acid | W: Add Water | S: Add Sand");
        println!("H: Show this help");
        println!("=======================================\n");
    }
}
