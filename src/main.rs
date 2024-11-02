use bevy::prelude::*;
mod plugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(plugins::states::StatesPlugin)
        .add_systems(Startup, startup)
        .init_state::<GameState>()
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
