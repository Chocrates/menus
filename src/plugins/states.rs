use bevy::prelude::*;

pub struct StatesPlugin;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Splash,
    Menu,
}

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        
    }
}


