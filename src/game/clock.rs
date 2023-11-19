use crate::assets::GameFont;

use bevy::prelude::*;

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
pub struct ClockText;

pub fn spawn_clock_text(mut commands: Commands, game_font: Res<GameFont>) {
    let font = game_font.0.clone();
    // Text with multiple sections
    commands.spawn((
        Name::new("ClockText"),
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_section(
            "",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: font.clone(),
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(15.0),
            ..Default::default()
        }),
        ClockText,
    ));
}

pub fn clock_text_update_system(
    time: Res<Time>,
    mut clock_text: Query<&mut Text, With<ClockText>>,
) {
    for mut text in &mut clock_text {
        text.sections[0].value = format!("Clock: {:.1?}", time.elapsed()); //time.elapsed().as_secs());
    }
}
