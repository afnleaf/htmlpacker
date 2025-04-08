use bevy::prelude::*;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseScrollUnit, MouseWheel};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

// Bundle to spawn our trackball camera easily
#[derive(Bundle, Default)]
pub struct TrackballCameraBundle {
    pub camera: Camera3dBundle,
    pub state: TrackballState,
    pub settings: TrackballSettings,
}

// The internal state of the trackball controller
#[derive(Component)]
pub struct TrackballState {
    pub target: Vec3,        // The point being orbited around
    pub position: Vec3,      // Current camera position
    pub up: Vec3,            // Camera's up vector
    
    // Internal tracking
    pub rotation_quat: Quat, // Current rotation as quaternion
    pub distance: f32,       // Distance from target
    pub last_position: Vec3, // For detecting changes
    
    // For dynamic movement
    pub moving: bool,        // Whether the camera is being moved
    pub velocity: Vec3,      // Current movement velocity for damping
    pub rotation_velocity: Quat, // Current rotation velocity for damping
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
    
    // Control mappings
    pub rotate_button: MouseButton,
    pub zoom_button: MouseButton,
    pub pan_button: MouseButton,
    
    // Alternative keyboard controls (A, S, D by default like Three.js)
    pub keys: [KeyCode; 3],
}

impl Default for TrackballState {
    fn default() -> Self {
        TrackballState {
            target: Vec3::ZERO,
            position: Vec3::new(0.0, 0.0, 5.0),
            up: Vec3::Y,
            rotation_quat: Quat::IDENTITY,
            distance: 5.0,
            last_position: Vec3::new(0.0, 0.0, 5.0),
            moving: false,
            velocity: Vec3::ZERO,
            rotation_velocity: Quat::IDENTITY,
        }
    }
}

impl Default for TrackballSettings {
    fn default() -> Self {
        TrackballSettings {
            rotate_speed: 1.0,
            zoom_speed: 1.2,
            pan_speed: 0.3,
            
            static_moving: false,
            damping_factor: 0.2,
            
            min_distance: 0.1,
            max_distance: 1000.0,
            
            // Default to Three.js settings
            rotate_button: MouseButton::Left,
            zoom_button: MouseButton::Middle,
            pan_button: MouseButton::Right,
            
            // A, S, D keys like in Three.js
            keys: [KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD],
        }
    }
}

pub fn spawn_trackball_camera(mut commands: Commands) {
    let mut camera = TrackballCameraBundle::default();
    // Initial position
    camera.state.target = Vec3::ZERO;
    camera.state.position = Vec3::new(0.0, 0.0, 50.0);
    camera.state.distance = 50.0;
    commands.spawn(camera);
}

pub fn trackball_camera_system(
    time: Res<Time>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(
        &TrackballSettings,
        &mut TrackballState,
        &mut Transform,
    )>,
) {
    // Accumulate mouse motion
    let mouse_delta: Vec2 = evr_motion.read().map(|ev| ev.delta).sum();
    
    // Accumulate scroll
    let mut scroll_delta = 0.0;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                scroll_delta -= ev.y * 0.1;
            }
            MouseScrollUnit::Pixel => {
                scroll_delta -= ev.y * 0.001;
            }
        }
    }
    
    for (settings, mut state, mut transform) in &mut q_camera {
        // Determine active mode based on mouse buttons
        let rotate_active = mouse_button.pressed(settings.rotate_button) || 
                           kbd.pressed(settings.keys[0]);
        let zoom_active = mouse_button.pressed(settings.zoom_button) || 
                         kbd.pressed(settings.keys[1]) || 
                         scroll_delta != 0.0;
        let pan_active = mouse_button.pressed(settings.pan_button) || 
                        kbd.pressed(settings.keys[2]);
        
        // Check for newly pressed buttons to provide immediate response
        let rotate_just_pressed = mouse_button.just_pressed(settings.rotate_button) || 
                                 kbd.just_pressed(settings.keys[0]);
        let zoom_just_pressed = mouse_button.just_pressed(settings.zoom_button) || 
                               kbd.just_pressed(settings.keys[1]);
        let pan_just_pressed = mouse_button.just_pressed(settings.pan_button) || 
                              kbd.just_pressed(settings.keys[2]);
        
        // Reset velocities immediately when input starts to ensure immediate response
        if rotate_just_pressed || zoom_just_pressed || pan_just_pressed {
            state.velocity = Vec3::ZERO;
            state.rotation_velocity = Quat::IDENTITY;
        }
        
        // Set moving flag if any control is active
        state.moving = rotate_active || zoom_active || pan_active;
        
        // ROTATION
        if rotate_active && mouse_delta != Vec2::ZERO {
            // Scale the motion by rotation speed
            let rot_delta = mouse_delta * settings.rotate_speed * 0.01;
            
            // Create a quaternion for this rotation
            // For a true trackball, we need to map 2D mouse motion to 3D rotation
            let axis = Vec3::new(-rot_delta.y, rot_delta.x, 0.0).normalize();
            let angle = rot_delta.length();
            let delta_quat = Quat::from_axis_angle(axis, angle);
            
            // Apply rotation immediately for responsiveness
            let current_rotation = state.rotation_quat;
            state.rotation_quat = delta_quat * current_rotation;
            
            if !settings.static_moving {
                // Store in velocity only for continued motion after release
                state.rotation_velocity = delta_quat;
            }
        }
        
        // ZOOM
        if zoom_active {
            let zoom_factor = if scroll_delta != 0.0 {
                // Zoom with scroll wheel
                1.0 + scroll_delta * settings.zoom_speed
            } else if mouse_delta.y != 0.0 {
                // Zoom with middle mouse drag
                1.0 - mouse_delta.y * 0.01 * settings.zoom_speed
            } else {
                1.0
            };
            
            // Get current distance
            let current_distance = state.distance;
            
            // Apply zoom to distance
            state.distance = (current_distance * zoom_factor)
                .clamp(settings.min_distance, settings.max_distance);
        }
        
        // PAN
        if pan_active && mouse_delta != Vec2::ZERO {
            // Scale by distance and pan speed
            let pan_scale = state.distance * settings.pan_speed * 0.01;
            let right = transform.right();
            let up = transform.up();
            
            // Movement in camera's local space
            let pan_delta = (right * -mouse_delta.x + up * mouse_delta.y) * pan_scale;
            
            // Apply pan immediately for responsiveness
            state.target += pan_delta;
            
            if !settings.static_moving {
                // Store in velocity only for continued motion after release
                state.velocity += pan_delta * 0.2; // Reduced to prevent excessive inertia
            }
        }
        
        // Apply damping if not static moving
        if !settings.static_moving {
            // Apply rotation velocity with damping
            if !state.moving && !state.rotation_velocity.is_near_identity() {
                // Create temporary copies to avoid double borrowing
                let current_rotation = state.rotation_quat;
                let current_velocity = state.rotation_velocity;
                
                state.rotation_quat = current_rotation * current_velocity;
                state.rotation_velocity = Quat::lerp(
                    current_velocity,
                    Quat::IDENTITY,
                    settings.damping_factor
                );
            }
            
            // Apply translation velocity with damping
            if !state.moving && state.velocity.length_squared() > 0.001 {
                // Create a temporary copy of velocity to avoid double borrowing
                let current_velocity = state.velocity;
                state.target += current_velocity;
                state.velocity *= 1.0 - settings.damping_factor;
            }
        }
        
        // Update the transform based on state
        // First apply rotation
        transform.rotation = state.rotation_quat;
        
        // Calculate new position based on target, distance, and rotation
        let offset = transform.back() * state.distance;
        transform.translation = state.target + offset;
        
        // Update last position
        state.last_position = transform.translation;
    }
}

// Add this system to your App
// app.add_systems(Update, trackball_camera_system);
