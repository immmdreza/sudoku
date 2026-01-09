use bevy::prelude::*;

#[derive(Debug, Bundle)]
pub struct TextBundle {
    text: Text2d,
    font: TextFont,
    color: TextColor,
    layout: TextLayout,
    transform: Transform,
}

impl TextBundle {
    pub fn new(
        text: impl Into<String>,
        font: impl Into<Handle<Font>>,
        font_size: f32,
        color: impl Into<Color>,
        transform: Transform,
    ) -> Self {
        Self {
            text: Text2d(text.into()),
            font: TextFont {
                font: font.into(),
                font_size,
                ..Default::default()
            },
            color: TextColor(color.into()),
            layout: TextLayout::new(Justify::Center, LineBreak::NoWrap),
            transform,
        }
    }
}
