use crate::plugins::states_plugin::{despawn_screen, GameState};
use bevy::prelude::*;

pub struct TimerPlugin;
#[derive(Component)]
pub struct Timer;


impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
    }
}
