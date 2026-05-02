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

pub trait NoInputGameCommand: GameCommand<()> {
    fn actuator() -> GameCommandActuator {
        GameCommandActuator::new::<Self>()
    }
}

impl<T: GameCommand<()>> NoInputGameCommand for T {}

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

#[derive(Component)]
pub struct GameCommandActuator {
    system_runner: Box<dyn Fn(&mut Commands) + Send + Sync + 'static>,
}

impl GameCommandActuator {
    pub fn new<G: GameCommand>() -> Self {
        Self {
            system_runner: Box::new(move |commands| {
                commands.trigger_game_command::<G>();
            }),
        }
    }

    pub fn new_with<G: GameCommand<Input>, Input: SystemInput + std::marker::Send + 'static>(
        input: Input::Inner<'static>,
    ) -> Self
    where
        <Input as bevy::prelude::SystemInput>::Inner<'static>: Send + Sync + Clone,
    {
        Self {
            system_runner: Box::new(move |commands| {
                commands.trigger_game_command_with::<G>(input.clone());
            }),
        }
    }
}

pub fn activate_game_command_on_click(
    ev: On<Pointer<Click>>,
    mut commands: Commands,
    actuator_query: Query<&GameCommandActuator>,
) {
    if let Ok(actuator) = actuator_query.get(ev.entity) {
        (actuator.system_runner)(&mut commands);
    }
}

pub trait GameCommandActuatorExtensions<'a> {
    fn activate_game_command_on_click(&mut self) -> &mut EntityCommands<'a>;
}

impl<'a> GameCommandActuatorExtensions<'a> for EntityCommands<'a> {
    fn activate_game_command_on_click(&mut self) -> &mut EntityCommands<'a> {
        self.observe(activate_game_command_on_click)
    }
}

pub trait InputGameCommandActuatorExtensions<'a, Input: SystemInput + std::marker::Send + 'static> {
    fn game_command_click_actuator_with<G: GameCommand<Input>>(
        &mut self,
        input: Input::Inner<'static>,
    ) -> &mut EntityCommands<'a>
    where
        <Input as bevy::prelude::SystemInput>::Inner<'static>: Send + Sync + Clone;
}

impl<'a, Input: SystemInput + std::marker::Send + 'static>
    InputGameCommandActuatorExtensions<'a, Input> for EntityCommands<'a>
{
    fn game_command_click_actuator_with<G: GameCommand<Input>>(
        &mut self,
        input: Input::Inner<'static>,
    ) -> &mut EntityCommands<'a>
    where
        <Input as bevy::prelude::SystemInput>::Inner<'static>: Send + Sync + Clone,
    {
        self.insert(GameCommandActuator::new_with::<G, Input>(input))
            .activate_game_command_on_click()
    }
}

pub trait NoInputGameCommandActuatorExtensions<'a> {
    fn game_command_click_actuator<G: GameCommand>(&mut self) -> &mut EntityCommands<'a>;
}

impl<'a> NoInputGameCommandActuatorExtensions<'a> for EntityCommands<'a> {
    fn game_command_click_actuator<G: GameCommand>(&mut self) -> &mut EntityCommands<'a> {
        self.insert(G::actuator()).activate_game_command_on_click()
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
