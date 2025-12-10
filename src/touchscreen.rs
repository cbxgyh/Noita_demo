use bevy::input::touch::{Touch, TouchPhase};
use bevy::prelude::*;

// Touchscreen and mobile support as mentioned in "Touching Screens" blog post
// Virtual controls and touch-based interaction

#[derive(Resource)]
pub struct TouchControls {
    pub enabled: bool,
    pub virtual_joystick: Option<VirtualJoystick>,
    pub action_buttons: Vec<VirtualButton>,
    pub brush_area: Option<TouchBrushArea>,
}

#[derive(Clone, Debug)]
pub struct VirtualJoystick {
    pub center: Vec2,
    pub current_touch: Option<u64>,
    pub direction: Vec2,
    pub deadzone: f32,
    pub max_distance: f32,
}

#[derive(Clone, Debug)]
pub struct VirtualButton {
    pub id: String,
    pub position: Vec2,
    pub size: Vec2,
    pub pressed: bool,
    pub current_touch: Option<u64>,
    pub action: ButtonAction,
}

#[derive(Clone, Debug)]
pub enum ButtonAction {
    Jump,
    CastSpell,
    SwitchAtomType,
    ToggleEditor,
    Menu,
}

#[derive(Clone, Debug)]
pub struct TouchBrushArea {
    pub bounds: Rect,
    pub current_touch: Option<u64>,
    pub brush_position: Vec2,
    pub active: bool,
}

// Touch gesture recognition
#[derive(Debug, Clone)]
pub enum TouchGesture {
    Tap { position: Vec2 },
    DoubleTap { position: Vec2 },
    Swipe { start: Vec2, end: Vec2, direction: Vec2 },
    Pinch { center: Vec2, scale: f32 },
    LongPress { position: Vec2, duration: f32 },
}

#[derive(Resource)]
pub struct TouchGestureRecognizer {
    pub active_touches: std::collections::HashMap<u64, TouchData>,
    pub gesture_events: Vec<TouchGesture>,
    pub double_tap_threshold: f32,
    pub long_press_threshold: f32,
}

#[derive(Clone, Debug)]
pub struct TouchData {
    pub id: u64,
    pub start_position: Vec2,
    pub current_position: Vec2,
    pub start_time: f64,
    pub last_update: f64,
}

impl Default for TouchControls {
    fn default() -> Self {
        Self {
            enabled: false,
            virtual_joystick: None,
            action_buttons: Vec::new(),
            brush_area: None,
        }
    }
}

impl Default for TouchGestureRecognizer {
    fn default() -> Self {
        Self {
            active_touches: std::collections::HashMap::new(),
            gesture_events: Vec::new(),
            double_tap_threshold: 0.3, // 300ms
            long_press_threshold: 0.5, // 500ms
        }
    }
}

// Setup touchscreen controls for mobile
pub fn setup_touch_controls(
    mut touch_controls: ResMut<TouchControls>,
    windows: Query<&Window>,
) {
    let window = windows.single();

    // Only enable on touch devices or when requested
    // For demo, enable by default but can be toggled
    touch_controls.enabled = true;

    // Setup virtual joystick on left side
    touch_controls.virtual_joystick = Some(VirtualJoystick {
        center: Vec2::new(100.0, window.height() - 100.0),
        current_touch: None,
        direction: Vec2::ZERO,
        deadzone: 10.0,
        max_distance: 80.0,
    });

    // Setup action buttons on right side
    touch_controls.action_buttons = vec![
        VirtualButton {
            id: "jump".to_string(),
            position: Vec2::new(window.width() - 100.0, window.height() - 150.0),
            size: Vec2::new(60.0, 60.0),
            pressed: false,
            current_touch: None,
            action: ButtonAction::Jump,
        },
        VirtualButton {
            id: "spell".to_string(),
            position: Vec2::new(window.width() - 180.0, window.height() - 100.0),
            size: Vec2::new(60.0, 60.0),
            pressed: false,
            current_touch: None,
            action: ButtonAction::CastSpell,
        },
        VirtualButton {
            id: "brush".to_string(),
            position: Vec2::new(window.width() - 100.0, window.height() - 250.0),
            size: Vec2::new(60.0, 60.0),
            pressed: false,
            current_touch: None,
            action: ButtonAction::SwitchAtomType,
        },
    ];

    // Setup brush area (most of screen for drawing)
    touch_controls.brush_area = Some(TouchBrushArea {
        bounds: Rect::new(
            150.0,
            0.0,
            window.width() - 200.0,
            window.height() - 300.0,
        ),
        current_touch: None,
        brush_position: Vec2::ZERO,
        active: false,
    });
}

// Process touch input
pub fn process_touch_input(
    touches: Res<Touches>,
    time: Res<Time>,
    mut touch_controls: ResMut<TouchControls>,
    mut gesture_recognizer: ResMut<TouchGestureRecognizer>,
) {
    if !touch_controls.enabled {
        return;
    }

    let current_time = time.elapsed_seconds_f64();

    // Process touch events
    for touch in touches.iter() {
        let touch_id = touch.id();
        if touches.just_pressed(touch_id) {
            handle_touch_start(touch, current_time, &mut touch_controls, &mut gesture_recognizer);
        } else if touches.get_pressed(touch_id).is_some() {
            handle_touch_move(touch, current_time, &mut touch_controls, &mut gesture_recognizer);
        } else if touches.just_released(touch_id) {
            handle_touch_end(touch, current_time, &mut touch_controls, &mut gesture_recognizer);
        }
    }

    // Update virtual controls
    update_virtual_joystick(&mut touch_controls);
    update_virtual_buttons(&touches, &mut touch_controls);
    update_brush_area(&touches, &mut touch_controls);
}

// Handle touch start
fn handle_touch_start(
    touch: &Touch,
    time: f64,
    touch_controls: &mut TouchControls,
    gesture_recognizer: &mut TouchGestureRecognizer,
) {
    let touch_data = TouchData {
        id: touch.id(),
        start_position: touch.position(),
        current_position: touch.position(),
        start_time: time,
        last_update: time,
    };

    gesture_recognizer.active_touches.insert(touch.id(), touch_data);

    // Check for double tap
    check_double_tap(touch, time, gesture_recognizer);
}

// Handle touch move
fn handle_touch_move(
    touch: &Touch,
    time: f64,
    touch_controls: &mut TouchControls,
    gesture_recognizer: &mut TouchGestureRecognizer,
) {
    if let Some(touch_data) = gesture_recognizer.active_touches.get_mut(&touch.id()) {
        touch_data.current_position = touch.position();
        touch_data.last_update = time;

        // Update virtual joystick if this touch is controlling it
        if let Some(ref mut joystick) = touch_controls.virtual_joystick {
            if joystick.current_touch == Some(touch.id()) {
                let offset = touch.position() - joystick.center;
                let distance = offset.length();

                if distance > joystick.deadzone {
                    joystick.direction = if distance > joystick.max_distance {
                        offset.normalize()
                    } else {
                        offset / joystick.max_distance
                    };
                } else {
                    joystick.direction = Vec2::ZERO;
                }
            }
        }

        // Update brush area if this touch is in brush area
        if let Some(ref mut brush_area) = touch_controls.brush_area {
            if brush_area.bounds.contains(touch.position()) && brush_area.current_touch == Some(touch.id()) {
                brush_area.brush_position = touch.position();
                brush_area.active = true;
            }
        }
    }
}

// Handle touch end
fn handle_touch_end(
    touch: &Touch,
    time: f64,
    touch_controls: &mut TouchControls,
    gesture_recognizer: &mut TouchGestureRecognizer,
) {
    if let Some(touch_data) = gesture_recognizer.active_touches.remove(&touch.id()) {
        let duration = time - touch_data.start_time;
        let distance = (touch_data.current_position - touch_data.start_position).length();

        // Recognize gestures
        if duration < 0.2 && distance < 10.0 {
            // Tap
            gesture_recognizer.gesture_events.push(TouchGesture::Tap {
                position: touch_data.start_position,
            });
        } else if distance > 50.0 {
            // Swipe
            let direction = (touch_data.current_position - touch_data.start_position).normalize();
            gesture_recognizer.gesture_events.push(TouchGesture::Swipe {
                start: touch_data.start_position,
                end: touch_data.current_position,
                direction,
            });
        } else if duration > gesture_recognizer.long_press_threshold as f64 {
            // Long press
            gesture_recognizer.gesture_events.push(TouchGesture::LongPress {
                position: touch_data.start_position,
                duration: duration as f32,
            });
        }
    }

    // Clear virtual control touches
    if let Some(ref mut joystick) = touch_controls.virtual_joystick {
        if joystick.current_touch == Some(touch.id()) {
            joystick.current_touch = None;
            joystick.direction = Vec2::ZERO;
        }
    }

    for button in &mut touch_controls.action_buttons {
        if button.current_touch == Some(touch.id()) {
            button.current_touch = None;
            button.pressed = false;
        }
    }

    if let Some(ref mut brush_area) = touch_controls.brush_area {
        if brush_area.current_touch == Some(touch.id()) {
            brush_area.current_touch = None;
            brush_area.active = false;
        }
    }
}

// Check for double tap
fn check_double_tap(
    touch: &Touch,
    time: f64,
    gesture_recognizer: &mut TouchGestureRecognizer,
) {
    // Simple double tap detection
    // In a real implementation, this would track recent taps
    static mut LAST_TAP_TIME: f64 = 0.0;
    static mut LAST_TAP_POSITION: Vec2 = Vec2::ZERO;

    unsafe {
        if time - LAST_TAP_TIME < gesture_recognizer.double_tap_threshold as f64 &&
           (touch.position() - LAST_TAP_POSITION).length() < 50.0 {
            gesture_recognizer.gesture_events.push(TouchGesture::DoubleTap {
                position: touch.position(),
            });
        }

        LAST_TAP_TIME = time;
        LAST_TAP_POSITION = touch.position();
    }
}

// Update virtual joystick
fn update_virtual_joystick(touch_controls: &mut TouchControls) {
    if let Some(ref mut joystick) = touch_controls.virtual_joystick {
        // Joystick is updated in touch move handler
        // Here we could add visual feedback or smoothing
    }
}

// Update virtual buttons
fn update_virtual_buttons(touches: &Res<Touches>, touch_controls: &mut TouchControls) {
    for touch in touches.iter() {
        let touch_id = touch.id();
        for button in &mut touch_controls.action_buttons {
            if button.current_touch == Some(touch_id) {
                button.pressed = touches.get_pressed(touch_id).is_some() || touches.just_pressed(touch_id);
            } else if button.current_touch.is_none() &&
                      touches.just_pressed(touch_id) &&
                      is_point_in_rect(touch.position(), button.position, button.size) {
                button.current_touch = Some(touch_id);
                button.pressed = true;
            }
        }
    }
}

// Update brush area
fn update_brush_area(touches: &Res<Touches>, touch_controls: &mut TouchControls) {
    if let Some(ref mut brush_area) = touch_controls.brush_area {
        for touch in touches.iter() {
            let touch_id = touch.id();
            if brush_area.current_touch == Some(touch_id) {
                brush_area.brush_position = touch.position();
                brush_area.active = touches.get_pressed(touch_id).is_some() || touches.just_pressed(touch_id);
            } else if brush_area.current_touch.is_none() &&
                      touches.just_pressed(touch_id) &&
                      brush_area.bounds.contains(touch.position()) {
                brush_area.current_touch = Some(touch_id);
                brush_area.brush_position = touch.position();
                brush_area.active = true;
            }
        }
    }
}

// Helper function
fn is_point_in_rect(point: Vec2, rect_pos: Vec2, rect_size: Vec2) -> bool {
    point.x >= rect_pos.x &&
    point.x <= rect_pos.x + rect_size.x &&
    point.y >= rect_pos.y &&
    point.y <= rect_pos.y + rect_size.y
}

// Convert touch input to game input
pub fn touch_to_game_input(
    touch_controls: Res<TouchControls>,
    mut gesture_recognizer: ResMut<TouchGestureRecognizer>,
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut mouse_input: ResMut<ButtonInput<MouseButton>>,
    mut mouse_motion: EventWriter<CursorMoved>,
) {
    if !touch_controls.enabled {
        return;
    }

    // Convert virtual joystick to movement keys
    if let Some(ref joystick) = touch_controls.virtual_joystick {
        let threshold = 0.1;

        // Clear existing movement inputs
        keyboard_input.release(KeyCode::KeyA);
        keyboard_input.release(KeyCode::KeyD);
        keyboard_input.release(KeyCode::KeyW);
        keyboard_input.release(KeyCode::KeyS);

        if joystick.direction.x > threshold {
            keyboard_input.press(KeyCode::KeyD);
        } else if joystick.direction.x < -threshold {
            keyboard_input.press(KeyCode::KeyA);
        }

        if joystick.direction.y > threshold {
            keyboard_input.press(KeyCode::KeyW);
        } else if joystick.direction.y < -threshold {
            keyboard_input.press(KeyCode::KeyS);
        }
    }

    // Convert virtual buttons to game actions
    for button in &touch_controls.action_buttons {
        match button.action {
            ButtonAction::Jump if button.pressed => {
                keyboard_input.press(KeyCode::Space);
            }
            ButtonAction::CastSpell if button.pressed => {
                mouse_input.press(MouseButton::Right);
            }
            ButtonAction::SwitchAtomType if button.pressed => {
                keyboard_input.press(KeyCode::Digit1); // Cycle through atom types
            }
            ButtonAction::ToggleEditor if button.pressed => {
                keyboard_input.press(KeyCode::F1);
            }
            ButtonAction::Menu if button.pressed => {
                keyboard_input.press(KeyCode::Escape);
            }
            _ => {}
        }
    }

    // Convert brush area to mouse input
    if let Some(ref brush_area) = touch_controls.brush_area {
        if brush_area.active {
            // Simulate mouse position for brush
            mouse_motion.send(CursorMoved {
                window: Entity::PLACEHOLDER,
                position: brush_area.brush_position,
                delta: None,
            });

            mouse_input.press(MouseButton::Left);
        } else {
            mouse_input.release(MouseButton::Left);
        }
    }

    // Handle gestures
    for gesture in &gesture_recognizer.gesture_events {
        match gesture {
            TouchGesture::DoubleTap { .. } => {
                // Double tap could toggle editor or switch modes
                keyboard_input.press(KeyCode::F1);
            }
            TouchGesture::Swipe { direction, .. } => {
                // Swipe could change atom types
                if direction.x > 0.7 {
                    keyboard_input.press(KeyCode::Digit2); // Next atom type
                } else if direction.x < -0.7 {
                    keyboard_input.press(KeyCode::Digit1); // Previous atom type
                }
            }
            TouchGesture::LongPress { .. } => {
                // Long press could open menu
                keyboard_input.press(KeyCode::Escape);
            }
            _ => {}
        }
    }

    // Clear processed gestures
    gesture_recognizer.gesture_events.clear();
}

// Toggle touchscreen controls
pub fn toggle_touchscreen(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut touch_controls: ResMut<TouchControls>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        touch_controls.enabled = !touch_controls.enabled;
        println!("Touchscreen controls: {}", if touch_controls.enabled { "enabled" } else { "disabled" });
    }
}

// Visual feedback for touch controls
pub fn render_touch_controls(
    touch_controls: Res<TouchControls>,
    mut commands: Commands,
) {
    if !touch_controls.enabled {
        return;
    }

    // Render virtual joystick
    if let Some(ref joystick) = touch_controls.virtual_joystick {
        // Render joystick base
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.5, 0.5, 0.5, 0.3),
                custom_size: Some(Vec2::new(joystick.max_distance * 2.0, joystick.max_distance * 2.0)),
                ..default()
            },
            transform: Transform::from_translation(joystick.center.extend(5.0)),
            ..default()
        });

        // Render joystick handle
        let handle_pos = joystick.center + joystick.direction * joystick.max_distance * 0.5;
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 0.7),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            transform: Transform::from_translation(handle_pos.extend(6.0)),
            ..default()
        });
    }

    // Render virtual buttons
    for button in &touch_controls.action_buttons {
        let color = if button.pressed {
            Color::rgba(1.0, 0.5, 0.5, 0.8)
        } else {
            Color::rgba(0.5, 0.5, 1.0, 0.6)
        };

        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(button.size),
                ..default()
            },
            transform: Transform::from_translation(button.position.extend(5.0)),
            ..default()
        });
    }
}

// Mobile-specific optimizations
// pub fn mobile_optimizations(
//     mut app: ResMut<App>,
// ) {
//     // On mobile, we might want to:
//     // - Reduce particle count
//     // - Lower resolution
//     // - Enable touch controls by default
//     // - Adjust UI scaling
//
//     #[cfg(target_os = "ios")]
//     {
//         println!("Running on iOS - enabling mobile optimizations");
//         // iOS specific optimizations
//     }
//
//     #[cfg(target_os = "android")]
//     {
//         println!("Running on Android - enabling mobile optimizations");
//         // Android specific optimizations
//     }
// }
