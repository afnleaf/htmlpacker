use bevy::{
    prelude::*,
    color::palettes::basic::SILVER,
    render::{
        extract_resource::{ExtractResource},
    },
    time::{Timer},
};


#[derive(Component)]
pub struct CurrentMapText;

//#[derive(Component, Default, Clone, ExtractComponent)]
#[derive(Resource, Default, Clone, ExtractResource)]
pub struct CurrentMap {
    pub index: usize,
}

pub fn current_map_widget(
    commands: &mut Commands,
) {
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(5.0),
            ..default()
        },
    ))
    .with_child((
        TextSpan::default(),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(SILVER.into()),
        CurrentMapText,
    ));
}

#[derive(Resource)]
pub struct KeyRepeatTimer(pub Timer);

const INCREASE_KEYS: [KeyCode; 1] = [KeyCode::ArrowLeft];
const DECREASE_KEYS: [KeyCode; 1] = [KeyCode::ArrowRight];

// This system runs every frame but only does work when CurrentMap changes.
pub fn update_map_text_display(
    current_map: Res<CurrentMap>,
    mut query: Query<&mut TextSpan, With<CurrentMapText>>,
) {
    // Res<T>::is_changed() is the key. It's true only on the frame the resource was modified.
    if current_map.is_changed() {
        // Since it changed, we can now afford the cost of updating the UI text.
        for mut span in &mut query {
            **span = format!("{}", current_map.index);
        }
    }
}

// This system now ONLY handles input and changes the data resource.
// It is now extremely fast.
pub fn map_update_system(
    mut current_map: ResMut<CurrentMap>,
    kbd: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut key_repeat_timer: ResMut<KeyRepeatTimer>,
    mut last_direction: Local<i8>,
) {
    let current_direction = if kbd.any_pressed(INCREASE_KEYS) {
        1
    } else if kbd.any_pressed(DECREASE_KEYS) {
        -1
    } else {
        0
    };

    let mut take_action = false;
    if current_direction != *last_direction {
        key_repeat_timer.0.reset();
        if current_direction != 0 {
            take_action = true;
        }
    } else if current_direction != 0 {
        key_repeat_timer.0.tick(time.delta());
        if key_repeat_timer.0.finished() {
            take_action = true;
        }
    }

    if take_action {
        let new_index = match current_direction {
            1 => (current_map.index + 1).min(108),
            -1 => current_map.index.saturating_sub(1),
            _ => current_map.index,
        };

        // Important: Just change the index. Don't touch the UI here.
        if new_index != current_map.index {
            current_map.index = new_index;
        }
    }

    *last_direction = current_direction;
}

