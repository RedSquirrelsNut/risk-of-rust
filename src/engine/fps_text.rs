use crate::assets::GameFont;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
pub struct FpsText;

pub fn spawn_fps_text(mut commands: Commands, game_font: Res<GameFont>) {
    let font = game_font.0.clone();
    // Text with multiple sections
    commands.spawn((
        Name::new("FpsText"),
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
            position_type: PositionType::Relative,
            ..Default::default()
        }),
        FpsText,
    ));
}

pub fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut fps_text {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[0].value = format!("FPS: {value:.0}");
            }
        }
    }
}
