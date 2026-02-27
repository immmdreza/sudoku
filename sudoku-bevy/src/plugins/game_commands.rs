use std::{any::TypeId, collections::HashMap};

use bevy::{ecs::system::SystemId, prelude::*};

pub struct GameCommandsPlugin;

impl Plugin for GameCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameCommandsSystemIdMap>().add_observer(
            |event: On<GameCommandsSystemIdMapEvent>, mut map: ResMut<GameCommandsSystemIdMap>| {
                if !map.map.contains_key(&event.0.0) {
                    map.map.insert(event.0.0.clone(), event.0.1);
                }
            },
        );
    }
}

#[derive(Debug, Resource, Default)]
struct GameCommandsSystemIdMap {
    map: HashMap<GameCommandId, Entity>,
}

#[derive(Debug, Event)]
struct GameCommandsSystemIdMapEvent((GameCommandId, Entity));

/// Describes a game command to be executed.
pub trait GameCommand<Input: SystemInput = ()>: Sized + 'static {
    fn system() -> impl System<In = Input, Out = ()>;

    /// Will makes it easier to implement [`Self::system`] method.
    fn fit<M, S: bevy::prelude::IntoSystem<Input, (), M>>(
        system: S,
    ) -> <S as IntoSystem<Input, (), M>>::System {
        IntoSystem::<Input, (), M>::into_system(system)
    }

    fn id() -> GameCommandId {
        GameCommandId::new::<Self, Input>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameCommandId(TypeId);

impl GameCommandId {
    pub fn new<G: GameCommand<In>, In: SystemInput>() -> Self {
        Self(TypeId::of::<G>())
    }
}

pub trait GameCommandsRegisterExtensions<
    'w,
    's,
    In: SystemInput + std::marker::Send + 'static,
    Marker,
>
{
    fn register_game_command<G: GameCommand<In>>(&mut self);
}

impl<'w, 's, Input: SystemInput + std::marker::Send + 'static>
    GameCommandsRegisterExtensions<'w, 's, Input, (Input,)> for Commands<'w, 's>
{
    fn register_game_command<G: GameCommand<Input>>(&mut self) {
        let id = self.register_system(G::system());
        self.trigger(GameCommandsSystemIdMapEvent((G::id(), id.entity())));
    }
}

pub trait GameCommandsExtensions<'w, 's, In: SystemInput + std::marker::Send + 'static> {
    fn trigger_game_command_with<G: GameCommand<In>>(&mut self, input: In::Inner<'static>)
    where
        <In as bevy::prelude::SystemInput>::Inner<'static>: std::marker::Send + std::marker::Sync;
}

impl<'w, 's, Input: SystemInput + std::marker::Send + 'static> GameCommandsExtensions<'w, 's, Input>
    for Commands<'w, 's>
{
    fn trigger_game_command_with<G: GameCommand<Input>>(&mut self, input: Input::Inner<'static>)
    where
        <Input as bevy::prelude::SystemInput>::Inner<'static>:
            std::marker::Send + std::marker::Sync,
    {
        self.run_system_cached_with(
            |input: In<Input::Inner<'static>>,
             mut commands: Commands,
             map: Res<GameCommandsSystemIdMap>| {
                if let Some(system_id_entity) = map.map.get(&G::id()) {
                    commands.run_system_with(
                        SystemId::<Input>::from_entity(*system_id_entity),
                        input.0,
                    );
                }
            },
            input,
        );
    }
}

pub trait GameCommandsNoInputExtensions<'w, 's> {
    fn trigger_game_command<G: GameCommand<()>>(&mut self);
}

impl<'w, 's> GameCommandsNoInputExtensions<'w, 's> for Commands<'w, 's> {
    fn trigger_game_command<G: GameCommand<()>>(&mut self) {
        self.run_system_cached(
            |mut commands: Commands, map: Res<GameCommandsSystemIdMap>| {
                if let Some(system_id_entity) = map.map.get(&G::id()) {
                    commands.run_system(SystemId::from_entity(*system_id_entity));
                }
            },
        );
    }
}

#[macro_export]
macro_rules! create_game_command {
    ($name: ident, $system: expr) => {
        pub struct $name;

        impl GameCommand for $name {
            fn system() -> impl System<In = (), Out = ()> {
                Self::fit($system)
            }
        }
    };
    ($name: ident, $input: ident, $system: expr) => {
        pub struct $name;

        impl GameCommand<In<$input>> for $name {
            fn system() -> impl System<In = In<$input>, Out = ()> {
                Self::fit($system)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use bevy::ecs::{
        component::Component,
        system::{Commands, Query, System},
    };

    use crate::plugins::game_commands::{GameCommand, GameCommandsRegisterExtensions};

    #[derive(Debug, Component)]
    struct _A;

    fn _sys(_: Query<&_A>, mut commands: Commands) {
        commands.register_game_command::<_MYGC>();
    }

    struct _MYGC;

    impl GameCommand for _MYGC {
        fn system() -> impl System<In = (), Out = ()> {
            Self::fit(_sys)
        }
    }
}
