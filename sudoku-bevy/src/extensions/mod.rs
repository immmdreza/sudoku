use std::time::Duration;

use bevy::{
    ecs::{
        component::Component,
        observer::On,
        system::{Commands, EntityCommands, Query},
    },
    math::{Vec3, curve::EaseFunction},
};
use bevy_tweening::{AnimCompletedEvent, Tween, TweenAnim, lens::TransformScaleLens};

#[derive(Debug, Component)]
pub struct Destroying;

pub trait CustomEntityCommands<'a> {
    fn destroy_with_anim(&'a mut self) -> &'a mut Self;
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
}
