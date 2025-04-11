use bevy::{
    color::palettes::css::*,
    pbr::{
        //CascadeShadowConfigBuilder, 
        NotShadowCaster, 
        //NotShadowReceiver
    },
    render::{
        mesh::*,
    },
    prelude::*,
};
use std::f32::consts::PI;

/*
Illuminance (lux)	Surfaces illuminated by
0.0001	        Moonless, overcast night sky (starlight)
0.002	        Moonless clear night sky with airglow
0.05–0.3	    Full moon on a clear night
3.4	            Dark limit of civil twilight under a clear sky
20–50       	Public areas with dark surroundings
50	            Family living room lights
80	            Office building hallway/toilet lighting
100	            Very dark overcast day
150	            Train station platforms
320–500	        Office lighting
400	            Sunrise or sunset on a clear day.
1000	        Overcast day; typical TV studio lighting
10,000–25,000	Full daylight (not direct sun)
32,000–100,000	Direct sunlight
*/

#[derive(Component)]
pub struct Star;

#[derive(Component, Clone)]
pub struct Orbit {
    pub speed: f32,
    pub axis: Vec3,
    pub center: Vec3,
}

const DAY: f32 = PI / 64.0;
const LUX: f32 = 3200.0;

// Startup system, create light and sun
pub fn spawn_sun_geocentrism(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sun is 149 million km away from earth
    let initial_light_position = Vec3::new(149_000.0, 0.0, 0.0);
    let target_point = Vec3::ZERO; // geocentric model
    let up_direction = Vec3::Y;
    let orbit = Orbit {
        speed: DAY,
        axis: Vec3::Y,
        center: target_point,
    };
    
    // light source itself
    commands.spawn((
        DirectionalLight {
            //color: Color::WHITE,
            illuminance: LUX,
            shadows_enabled: true,
            shadow_depth_bias: DirectionalLight::DEFAULT_SHADOW_DEPTH_BIAS,
            shadow_normal_bias: DirectionalLight::DEFAULT_SHADOW_NORMAL_BIAS,
            ..default()
        },
        // Its position and initial orientation
        Transform::from_translation(initial_light_position)
            .looking_at(target_point, up_direction),
        // Marker component for the system to find it
        Star,
        orbit.clone(),
    ));

    // sphere at the same position as physical sun
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(696.34).mesh().uv(32, 18))),
        MeshMaterial3d(materials.add(StandardMaterial {
                base_color: WHITE.into(),
                //emissive: YELLOW.into() * 1.5,// Make it glow
                emissive: Color::srgb(1.0, 1.0, 0.0).into(),
                ..default()
            })),
        NotShadowCaster,
        Transform::from_translation(initial_light_position),
        Star,
        orbit.clone(),
    ));
}

// Update system, orbits sun around earth at 0,0,0
pub fn orbit_geocentrism(
    mut query: Query<(&mut Transform, &Orbit), With<Star>>,
    time: Res<Time>,
) {
    for (mut transform, orbit) in &mut query {
        // total elapsed time instead of delta time 
        // allows for smoother and more predictable movement
        let total_time = time.elapsed_secs();
        
        // how far away sun is from eart
        let radius = 149_000.0;
        let angle = orbit.speed * total_time;
        // calc new position in orbit directly using sine and cosine
        let new_x = radius * angle.cos();
        let new_z = radius * angle.sin();
        
        // set new position and point at center
        transform.translation = Vec3::new(new_x, 0.0, new_z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
        
        //println!("Light at ({}, 0, {}), angle: {}", new_x, new_z, angle);
    }
}


pub fn ambient_light(commands: &mut Commands) {
    commands.insert_resource(AmbientLight {
        //color: WHITE.into(),
        brightness: 1000.0,
        ..default()
    });
}


    /*
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(-16.0, 0.0, -16.0),
    ));
    
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(16.0, 0.0, 16.0),
    ));
    */
        
    /*
    // Mesh component - a sphere
    PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Sphere { radius: 0.3 })),
        material: materials.add(StandardMaterial {
            base_color: Color::YELLOW,
            emissive: Color::YELLOW * 2.0, // Make it glow
            ..default()
        }),
        transform: Transform::from_translation(initial_light_position),
        ..default()
    },
    */
