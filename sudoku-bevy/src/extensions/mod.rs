use std::time::Duration;

use bevy::{
    ecs::{
        component::Component,
        observer::On,
        system::{Commands, EntityCommands, Query},
    },
    math::{Vec3, curve::EaseFunction},
    picking::events::{Over, Pointer},
};
use bevy_tweening::{AnimCompletedEvent, Tween, TweenAnim, lens::TransformScaleLens};

use crate::shared::components::UpdateHelpText;

#[derive(Debug, Component)]
pub struct Destroying;

pub fn quick_hover_help_text(
    text: impl Into<String> + Clone,
) -> impl Fn(On<'_, '_, Pointer<Over>>, Commands<'_, '_>) {
    move |_ev: On<Pointer<Over>>, mut commands: Commands| {
        commands.update_help_text(text.clone());
    }
}

pub trait CustomEntityCommands<'a> {
    fn destroy_with_anim(&'a mut self) -> &'a mut Self;

    fn quick_hover_help_text(
        &'a mut self,
        text: impl Into<String> + Clone + Send + Sync + 'static,
    ) -> &'a mut EntityCommands<'a>;
}

impl<'a> CustomEntityCommands<'a> for EntityCommands<'a> {
    fn destroy_with_anim(&'a mut self) -> &'a mut Self {
        self.observe(
            |ev: On<AnimCompletedEvent>, mut commands: Commands, destroying: Query<&Destroying>| {
                if destroying.get(ev.anim_entity).is_ok() {
                    commands.entity(ev.anim_entity).despawn();
                }
            },
        )
        .insert((
            Destroying,
            TweenAnim::new(Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(1),
                TransformScaleLens {
                    start: Vec3::splat(1.),
                    end: Vec3::splat(0.),
                },
            )),
        ))
    }

    fn quick_hover_help_text(
        &'a mut self,
        text: impl Into<String> + Clone + Send + Sync + 'static,
    ) -> &'a mut EntityCommands<'a> {
        self.observe(move |_ev: On<Pointer<Over>>, mut commands: Commands| {
            commands.update_help_text(text.clone());
        })
    }
}

pub trait CustomCommands<'w, 's> {
    fn update_help_text(&mut self, text: impl Into<String>);
}

impl<'w, 's> CustomCommands<'w, 's> for Commands<'w, 's> {
    fn update_help_text(&mut self, text: impl Into<String>) {
        self.trigger(UpdateHelpText(text.into()));
    }
}
