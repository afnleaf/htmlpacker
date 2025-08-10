/* 
This is an experimental LLM generated file
I took https://www.npmjs.com/package/three-trackballcontrols?activeTab=code
+ https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
then put it into an LLM asking for Rust code. Honestly, the results are subpar.
Tried Sonnet 3.7 Thinking, Gemini 2.5 Pro, Grok 3 Think

I could probably figure this out myself but this felt like a good experiment
for LLMs. Take some existing code and transform it into another language.
The solution is already there, no problem solving required, just translate. 
Basically the ideal use case for an LLM. Pure vibe coding.
*/


// Grok 3 version
// I fix width when too long
// also added the R to reset
use bevy::prelude::*;
use bevy::input::mouse::{
    MouseButton, MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::window::{Window, PrimaryWindow};

// Bundle to spawn our trackball camera easily
#[derive(Bundle, Default)]
pub struct TrackballCameraBundle {
    pub camera: Camera3d,
    pub state: TrackballState,
    pub settings: TrackballSettings,
}

// The internal state of the trackball controller
#[derive(Component)]
pub struct TrackballState {
    pub target: Vec3,        // The point being orbited around
    pub position: Vec3,      // Current camera position
    pub up: Vec3,            // Camera's up vector
    pub rotation_quat: Quat, // Current rotation as quaternion
    pub distance: f32,       // Distance from target
    pub last_position: Vec3, // For detecting changes
    pub moving: bool,        // Whether the camera is being moved
    pub velocity: Vec3,      // For pan damping
    pub last_rotation_axis: Option<Vec3>, // For rotation damping
    pub last_rotation_angle: f32,  // For rotation damping
}

impl Default for TrackballState {
    fn default() -> Self {
        TrackballState {
            target: Vec3::ZERO,
            position: Vec3::new(0.0, 0.0, 50.0),
            up: Vec3::Y,
            rotation_quat: Quat::IDENTITY,
            distance: 50.0,
            last_position: Vec3::new(0.0, 0.0, 50.0),
            moving: false,
            velocity: Vec3::ZERO,
            last_rotation_axis: None,
            last_rotation_angle: 0.0,
        }
    }
}

// The configuration of the trackball controller
#[derive(Component)]
pub struct TrackballSettings {
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,
    pub static_moving: bool,  // If true, no damping is applied
    pub damping_factor: f32,  // For non-static movement (lower = more damping)
    pub min_distance: f32,    // Minimum distance from target
    pub max_distance: f32,    // Maximum distance from target
    pub no_rotate: Option<bool>,
    pub no_zoom: Option<bool>,
    pub no_pan: Option<bool>,
    pub rotate_button: MouseButton,
    pub zoom_button: MouseButton,
    pub pan_button: MouseButton,
    pub keys: [KeyCode; 4],
}

impl Default for TrackballSettings {
    fn default() -> Self {
        TrackballSettings {
            rotate_speed: 1.0,
            zoom_speed: 3.5,
            pan_speed: 0.1,
            static_moving: false,
            damping_factor: 0.2,
            min_distance: 0.0001,
            max_distance: f32::INFINITY,
            no_rotate: Some(false),
            no_zoom: Some(false),
            no_pan: Some(false),
            rotate_button: MouseButton::Left,
            zoom_button: MouseButton::Middle,
            pan_button: MouseButton::Right,
            keys: [
                KeyCode::KeyA,
                KeyCode::KeyS, 
                KeyCode::KeyD,
                KeyCode::KeyR,
            ],
        }
    }
}

use bevy::render::view::{NoIndirectDrawing};

pub fn spawn_trackball_camera(mut commands: Commands) {
    let mut camera = TrackballCameraBundle::default();
    commands.spawn((camera, NoIndirectDrawing));
}

pub fn trackball_camera_system(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_scroll: EventReader<MouseWheel>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&TrackballSettings, &mut TrackballState, &mut Transform)>,
) {
    // Get the primary window
    let window = primary_window_query.single();
    //let screen_width = window.expect("REASON").width();
    let screen_width = window.unwrap().width();

    // Accumulate mouse motion
    let mouse_delta: Vec2 = evr_motion.read().map(|ev| ev.delta).sum();

    // Accumulate scroll
    let mut scroll_delta = 0.0;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => scroll_delta -= ev.y * 0.01,
            MouseScrollUnit::Pixel => scroll_delta -= ev.y * 0.00025,
        }
    }

    for (settings, mut state, mut transform) in &mut q_camera {
        // Reset check
        if kbd.pressed(settings.keys[3]) {
            *state = TrackballState::default();
        }

        // Input states
        let rotate_active = 
            (!settings.no_rotate.unwrap_or(false)) &&
            (mouse_button.pressed(settings.rotate_button) || 
            kbd.pressed(settings.keys[0]));
        let zoom_active = 
            (!settings.no_zoom.unwrap_or(false)) &&
            (mouse_button.pressed(settings.zoom_button) || 
            kbd.pressed(settings.keys[1]) || 
            scroll_delta != 0.0);
        let pan_active = 
            (!settings.no_pan.unwrap_or(false)) &&
            (mouse_button.pressed(settings.pan_button) || 
            kbd.pressed(settings.keys[2]));

        state.moving = rotate_active || zoom_active || pan_active;

        // **Rotation**
        if rotate_active && mouse_delta != Vec2::ZERO {
            // Scale mouse delta like getMouseOnCircle
            let dx = 2.0 * mouse_delta.x / screen_width;
            let dy = -2.0 * mouse_delta.y / screen_width;

            let eye = state.position - state.target;
            let eye_direction = eye.normalize_or_zero();
            let up_direction = transform.up();
            let sideways_direction = 
                up_direction.cross(eye_direction).normalize_or_zero();

            let move_direction = sideways_direction * dx + up_direction * dy;
            let move_length = move_direction.length();

            if move_length > 0.000001 {
                let axis = move_direction.cross(eye).normalize_or_zero();
                let angle = move_length * settings.rotate_speed;
                let delta_quat = Quat::from_axis_angle(axis, angle);

                // Apply rotation
                state.rotation_quat = delta_quat * state.rotation_quat;

                // Store for damping
                if !settings.static_moving {
                    state.last_rotation_axis = Some(axis);
                    state.last_rotation_angle = angle;
                }
            }
        } else if 
            !settings.static_moving && 
            state.last_rotation_angle > 0.000001 
        {
            // Apply damping
            state.last_rotation_angle *= 1.0 - settings.damping_factor;
            if let Some(axis) = state.last_rotation_axis {
                let delta_quat = 
                    Quat::from_axis_angle(axis, state.last_rotation_angle);
                state.rotation_quat = delta_quat * state.rotation_quat;
            }
            // Reset if angle becomes negligible
            if state.last_rotation_angle < 0.000001 {
                state.last_rotation_axis = None;
                state.last_rotation_angle = 0.0;
            }
        }

        // **Zoom**
        if zoom_active {
            let mut factor = 1.0;
            if scroll_delta != 0.0 {
                // Mouse wheel zoom (direct, no damping)
                factor = 1.0 + scroll_delta * settings.zoom_speed;
            } else if mouse_delta.y != 0.0 {
                // Middle mouse drag zoom (with damping if not static)
                factor = 1.0 + (-mouse_delta.y * 0.01) * settings.zoom_speed;
            }
            state.distance = 
                (state.distance * factor)
                .clamp(settings.min_distance, settings.max_distance);
        }

        // **Pan**
        if pan_active && mouse_delta != Vec2::ZERO {
            let pan_scale = state.distance * settings.pan_speed;
            let right = transform.right();
            let up = transform.up();
            let mouse_change = Vec2::new(
                -mouse_delta.x * 0.01, mouse_delta.y * 0.01);
            let pan_delta = (right * mouse_change.x +
                up * mouse_change.y) * pan_scale;

            state.target += pan_delta;

            if !settings.static_moving {
                state.velocity += pan_delta
            }
        }

        // **Damping for Pan**
        if 
        !settings.static_moving && 
        !state.moving && 
        state.velocity.length_squared() > 0.000001 {
            state.velocity *= 1.0 - settings.damping_factor;
            if state.velocity.length_squared() < 0.000001 {
                state.velocity = Vec3::ZERO;
            }
        }

        // **Update Transform**
        transform.rotation = state.rotation_quat;
        let offset = transform.back() * state.distance;
        transform.translation = state.target + offset;
        state.position = transform.translation;

        // Ensure distance constraints
        let eye = state.position - state.target;
        let eye_length = eye.length();
        if eye_length > settings.max_distance {
            state.distance = settings.max_distance;
            transform.translation = 
                state.target + eye.normalize() * settings.max_distance;
            state.position = transform.translation;
        } else if eye_length < settings.min_distance {
            state.distance = settings.min_distance;
            transform.translation = 
                state.target + eye.normalize() * settings.min_distance;
            state.position = transform.translation;
        }
    }
}

        
            
        /*
* Old pan did not update each frame
        // **Pan**
        if pan_active && mouse_delta != Vec2::ZERO {
            let pan_scale = state.distance * settings.pan_speed;
            let right = transform.right();
            let up = transform.up();
            let mouse_change = Vec2::new(-mouse_delta.x * 0.01, mouse_delta.y * 0.01);
            let pan_delta = (right * mouse_change.x + up * mouse_change.y) * pan_scale;

            if settings.static_moving {
                state.target += pan_delta;
            } else {
                state.velocity += pan_delta;
            }
        }

        // **Damping for Pan**
        if !settings.static_moving && !state.moving && state.velocity.length_squared() > 0.000001 {
            let velocity = state.velocity; // Clone velocity to avoid borrowing issue
            state.target += velocity;
            state.velocity *= 1.0 - settings.damping_factor;
            if state.velocity.length_squared() < 0.000001 {
                state.velocity = Vec3::ZERO;
            }
        }
        */
