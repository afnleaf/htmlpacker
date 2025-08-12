#![allow(dead_code)]
#![allow(unused_variables)]

use bevy::{
    prelude::*,
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    image::Image,
    render::{
        render_resource::{
            Extent3d, TextureDimension, TextureFormat,
        },
        render_asset::RenderAssetUsages,
    },
};

// FPS COUNTER ----------------------------------------------------------------
// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
pub struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
pub struct ColorText;

pub fn text_color_system(time: Res<Time>, mut query: Query<&mut TextColor, With<ColorText>>) {
    for mut text_color in &mut query {
        let seconds = time.elapsed_secs();

        // Update the color of the ColorText span.
        text_color.0 = Color::srgb(
            ops::sin(1.25 * seconds) / 2.0 + 0.5,
            ops::sin(0.75 * seconds) / 2.0 + 0.5,
            ops::sin(0.50 * seconds) / 2.0 + 0.5,
        );
    }
}

pub fn fps_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                **span = format!("{value:.2}");
            }
        }
    }
}

pub fn ui(commands: &mut Commands) {
    // Text with one section
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("hello\nbevy!"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            //font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 67.0,
            ..default()
        },
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        ColorText,
    ));

    commands.spawn((
        // Here we are able to call the `From` method instead of creating a new `TextSection`.
        // This will use the default font (a minimal subset of FiraMono) and apply the default styling.
        Text::new("From an &str into a Text with the default font!"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));
}

pub fn fps_widget(commands: &mut Commands) {
    // Empty text for the parent to satisfy Bevy’s hierarchy
    // WHY, im not sure
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(5.0),
            ..default()
        },
    ))
    .with_child((
        TextSpan::default(),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(GOLD.into()),
        FpsText,
    ));
}

// Creates a colorful test pattern
pub fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
