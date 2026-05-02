use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct HelpText;

#[derive(Debug, Event)]
pub struct UpdateHelpText(pub String);
