use std::collections::HashMap;

use bevy::{ecs::system::SystemId, prelude::*};

pub struct GameCommandsPlugin;

impl Plugin for GameCommandsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameCommandsSystemIdMap>()
            .add_observer(
                |event: On<GameCommandsSystemIdMapEvent>,
                 mut map: ResMut<GameCommandsSystemIdMap>| {
                    if !map.map.contains_key(&event.0.0) {
                        map.map.insert(event.0.0.clone(), event.0.1);
                    }
                },
            )
            .add_observer(
                |event: On<GameCommandEvent>,
                 mut commands: Commands,
                 map: Res<GameCommandsSystemIdMap>| {
                    if let Some(system_id) = map.map.get(&event.event_shortcut) {
                        commands.run_system(*system_id);
                    }
                },
            );
    }
}

#[derive(Debug, Resource, Default)]
pub struct GameCommandsSystemIdMap {
    map: HashMap<String, SystemId<(), ()>>,
}

#[derive(Debug, Event)]
pub struct GameCommandsSystemIdMapEvent((String, SystemId<(), ()>));

pub struct GameCommandSystem;

/// Describes a game command to be executed.
pub trait GameCommand {
    const EXCLUSIVE_NAME: &str;

    fn system() -> impl System<In = (), Out = ()>;

    fn fit<M, S: bevy::prelude::IntoSystem<(), (), M>>(
        system: S,
    ) -> <S as IntoSystem<(), (), M>>::System {
        IntoSystem::<(), (), M>::into_system(system)
    }
}

#[derive(Debug, Event)]
pub struct GameCommandEvent {
    event_shortcut: String,
}

impl GameCommandEvent {
    pub fn new<G: GameCommand>() -> Self {
        Self {
            event_shortcut: G::EXCLUSIVE_NAME.to_string(),
        }
    }
}

pub trait GameCommandsExtensions<'w, 's> {
    fn register_game_command<G: GameCommand>(&mut self);

    fn run_game_command<G: GameCommand>(&mut self);
}

impl<'w, 's> GameCommandsExtensions<'w, 's> for Commands<'w, 's> {
    fn register_game_command<G: GameCommand>(&mut self) {
        let id = self.register_system(G::system());
        self.trigger(GameCommandsSystemIdMapEvent((
            G::EXCLUSIVE_NAME.to_string(),
            id,
        )));
    }

    fn run_game_command<G: GameCommand>(&mut self) {
        self.trigger(GameCommandEvent::new::<G>());
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::{
        component::Component,
        system::{Commands, Query, System},
    };

    use crate::plugins::game_commands::{GameCommand, GameCommandsExtensions};

    #[derive(Debug, Component)]
    struct _A;

    fn _sys(_: Query<&_A>, mut commands: Commands) {
        commands.register_game_command::<_MYGC>();
    }

    struct _MYGC;

    impl GameCommand for _MYGC {
        const EXCLUSIVE_NAME: &str = "T";

        fn system() -> impl System<In = (), Out = ()> {
            Self::fit(_sys)
        }
    }
}
