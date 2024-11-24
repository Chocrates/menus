use crate::plugins::states_plugin::{despawn_screen, GameState};
use bevy::prelude::*;

use serde::Deserialize;
use std::sync::Mutex;

use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};

use rand::Rng;
use std::time::Duration;

pub struct MenuPlugin;

#[derive(Component)]
pub struct Menu;

// Tag component used to tag entities added on the splash screen
#[derive(Component)]
struct OnMenuScreen;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Clone)]
struct _Timer {
    id: u32,
}

static _TIMERS: Mutex<Vec<_Timer>> = Mutex::new(Vec::new());

// Data structure for USNO API response
#[derive(Debug, Deserialize)]
struct UsnoResponse {
    results: Results,
}

#[derive(Debug, Deserialize)]
struct Results {
    sunrise: String,
    sunset: String,
}

// Component to store fetched sunrise/sunset data
#[derive(Component)]
struct SunriseSunsetData {
    sunrise: String,
    sunset: String,
}

#[derive(Component)]
struct ComputeTask(Task<UsnoResponse>);

const NUM_CUBES: u32 = 6;

#[derive(Resource, Deref)]
struct BoxMeshHandle(Handle<Mesh>);

#[derive(Resource, Deref)]
struct BoxMaterialHandle(Handle<StandardMaterial>);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Menu),
            (menu_setup_system, setup_env, add_assets, spawn_tasks),
        )
        .add_systems(Update, (menu_update_system, handle_tasks))
        .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMenuScreen>);
    }
}

fn menu_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    {
        let mut list = _TIMERS.lock().unwrap();
        list.push(_Timer { id: 1 });
        println!("Added struct with id: {}", list[0].id);
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,

                    ..default()
                },
                ..default()
            },
            OnMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(90.0),
                        height: Val::Percent(90.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        ..default()
                    },
                    background_color: Color::srgb(1., 1., 1.).into(),
                    ..default()
                })
                .with_children(|parent| {
                    let timers = {
                        let timers_guard = _TIMERS.lock().unwrap();
                        timers_guard.clone()
                    };
                    for timer in timers {
                        parent
                            .spawn(ButtonBundle {
                                style: Style {
                                    width: Val::Percent(75.0),
                                    height: Val::Px(65.0),
                                    border: UiRect::all(Val::Px(5.0)),
                                    // horizontally center child text
                                    justify_content: JustifyContent::Center,
                                    // vertically center child text
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                border_color: BorderColor(Color::BLACK),
                                border_radius: BorderRadius::MAX,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Button",
                                    TextStyle {
                                        font: asset_server.load("fonts/ShadeBlue-2OozX.ttf"),
                                        font_size: 40.0,
                                        color: Color::srgb(0.9, 0.9, 0.9),
                                    },
                                ));
                            });
                    }
                });

            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(90.0),
                    height: Val::Percent(90.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                },
                background_color: Color::srgb(0.9, 0.9, 0.9).into(),
                ..default()
            });
        });
}

fn menu_update_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
                bevy::log::info!("Button PRESSED");
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
                bevy::log::info!("Button Hovered");
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
                bevy::log::info!("Button Out");
            }
        }
    }
}

/// Startup system which runs only once and generates our Box Mesh
/// and Box Material assets, adds them to their respective Asset
/// Resources, and stores their handles as resources so we can access
/// them later when we're ready to render our Boxes
fn add_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let box_mesh_handle = meshes.add(Cuboid::new(0.25, 0.25, 0.25));
    commands.insert_resource(BoxMeshHandle(box_mesh_handle));

    let box_material_handle = materials.add(Color::srgb(1.0, 0.2, 0.3));
    commands.insert_resource(BoxMaterialHandle(box_material_handle));
}

#[derive(Component)]
struct ComputeTransform(Task<CommandQueue>);

/// This system generates tasks simulating computationally intensive
/// work that potentially spans multiple frames/ticks. A separate
/// system, [`handle_tasks`], will poll the spawned tasks on subsequent
/// frames/ticks, and use the results to spawn cubes
fn spawn_tasks(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    bevy::log::info!("plugins::states::menu::spawn_tasks: NUM_CUBES: {NUM_CUBES}");
    for x in 0..NUM_CUBES {
        for y in 0..NUM_CUBES {
            for z in 0..NUM_CUBES {
                // Spawn new task on the AsyncComputeTaskPool; the task will be
                // executed in the background, and the Task future returned by
                // spawn() can be used to poll for the result
                let entity = commands.spawn_empty().id();
                let task = thread_pool.spawn(async move {
                    let duration = Duration::from_secs_f32(rand::thread_rng().gen_range(0.05..5.0));

                    // Pretend this is a time-intensive function. :)
                    async_std::task::sleep(duration).await;

                    // Such hard work, all done!
                    let transform = Transform::from_xyz(x as f32, y as f32, z as f32);
                    let mut command_queue = CommandQueue::default();

                    // we use a raw command queue to pass a FnOne(&mut World) back to be
                    // applied in a deferred manner.
                    command_queue.push(move |world: &mut World| {
                        let (box_mesh_handle, box_material_handle) = {
                            let mut system_state = SystemState::<(
                                Res<BoxMeshHandle>,
                                Res<BoxMaterialHandle>,
                            )>::new(world);
                            let (box_mesh_handle, box_material_handle) =
                                system_state.get_mut(world);

                            (box_mesh_handle.clone(), box_material_handle.clone())
                        };

                        world
                            .entity_mut(entity)
                            // Add our new PbrBundle of components to our tagged entity
                            .insert(PbrBundle {
                                mesh: box_mesh_handle,
                                material: box_material_handle,
                                transform,
                                ..default()
                            })
                            // Task is complete, so remove task component from entity
                            .remove::<ComputeTransform>();
                    });

                    command_queue
                });

                // Spawn new entity and add our new task as a component
                commands.entity(entity).insert(ComputeTransform(task));
            }
        }
    }
}

/// This system queries for entities that have our Task<Transform> component. It polls the
/// tasks to see if they're complete. If the task is complete it takes the result, adds a
/// new [`PbrBundle`] of components to the entity using the result from the task's work, and
/// removes the task component from the entity.
fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut ComputeTransform>) {
    for mut task in &mut transform_tasks {
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            bevy::log:: info!("Spawning cubes");
            commands.append(&mut commands_queue);
        }
    }
}

/// This system is only used to setup light and camera for the environment
fn setup_env(mut commands: Commands) {
    bevy::log::info!("plugins::states::menu::setup_env");
    // Used to center camera on spawned cubes
    let offset = if NUM_CUBES % 2 == 0 {
        (NUM_CUBES / 2) as f32 - 0.5
    } else {
        (NUM_CUBES / 2) as f32
    };

    // lights
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 12.0, 15.0),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(offset, offset, 15.0)
            .looking_at(Vec3::new(offset, offset, 0.0), Vec3::Y),
        camera: Camera {
            order: 1,
            ..default()
        },
        ..default()
    });
}
