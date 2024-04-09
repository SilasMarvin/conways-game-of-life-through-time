use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_egui::{
    egui::{self, text::LayoutJob, Color32, TextFormat},
    EguiContexts, EguiPlugin,
};
use bevy_flycam::prelude::*;
use rand::Fill;
use wasm_bindgen::prelude::*;

mod custom_material_plugin;
use custom_material_plugin::{CustomMaterialPlugin, InstanceData, InstanceMaterialData};

const DEFAULT_GAME_SIZE: usize = 32;
const STEP_HEIGHT: f32 = 1.;
const CELL_SIZE: f32 = 1.;

#[derive(Resource)]
struct Game {
    state: Vec<bool>,
    step: usize,
    size: usize,
}

fn create_random_state(game_size: usize) -> Vec<bool> {
    let mut tiles = vec![false; game_size * game_size];
    let mut rng = rand::thread_rng();
    tiles[0..game_size * game_size].try_fill(&mut rng).unwrap();
    tiles
}

impl Game {
    fn new() -> Self {
        Self {
            state: create_random_state(DEFAULT_GAME_SIZE),
            step: 0,
            size: DEFAULT_GAME_SIZE,
        }
    }
}

#[wasm_bindgen(start)]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#conways-game-of-life-through-time-canvas".into()),
                    ..default()
                }),
                ..default()
            }),
            CustomMaterialPlugin,
            NoCameraPlayerPlugin,
            EguiPlugin,
        ))
        .insert_resource(Game::new())
        .insert_resource(MovementSettings {
            sensitivity: 0.00005, // default: 0.00012
            speed: 36.0,          // default: 12.0
        })
        .add_systems(Startup, setup)
        .add_systems(Update, step)
        .add_systems(Update, (handle_reset, ui_system).before(step))
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        meshes.add(Cuboid::new(CELL_SIZE, STEP_HEIGHT, CELL_SIZE)),
        SpatialBundle::INHERITED_IDENTITY,
        InstanceMaterialData(Vec::new()),
        NoFrustumCulling,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5., DEFAULT_GAME_SIZE as f32 * 1.7),
            ..default()
        },
        FlyCam,
    ));
}

fn step(mut game: ResMut<Game>, mut instance_query: Query<&mut InstanceMaterialData>) {
    println!("S");
    game.step += 1;

    let mut instance = instance_query.single_mut();
    let mut next_tile_state = game.state.clone();

    let offset = (game.size / 2) as f32 * CELL_SIZE * -1.;
    for i in 0..game.size {
        for ii in 0..game.size {
            let check_row = |row: usize, check_center: bool| {
                let mut n_count = 0;
                if ii > 0 && game.state[row * game.size + ii - 1] {
                    n_count += 1;
                }
                if check_center && game.state[row * game.size + ii] {
                    n_count += 1;
                }
                if ii + 1 < game.size && game.state[row * game.size + ii + 1] {
                    n_count += 1;
                }
                n_count
            };
            let n_top_count = if i > 0 { check_row(i - 1, true) } else { 0 };
            let n_middle_count = check_row(i, false);
            let bottom_n = if i + 1 < game.size {
                check_row(i + 1, true)
            } else {
                0
            };
            let alive_n = n_top_count + n_middle_count + bottom_n;

            if game.state[i * game.size + ii] && alive_n < 2 {
                next_tile_state[i * game.size + ii] = false;
            }
            if game.state[i * game.size + ii] && (alive_n == 2 || alive_n == 3) {
                next_tile_state[i * game.size + ii] = true;
            }
            if game.state[i * game.size + ii] && alive_n > 3 {
                next_tile_state[i * game.size + ii] = false;
            }
            if !game.state[i * game.size + ii] && alive_n == 3 {
                next_tile_state[i * game.size + ii] = true;
            }

            if game.state[i * game.size + ii] {
                instance.push(InstanceData {
                    position: Vec3::new(
                        offset + (ii as f32 * CELL_SIZE),
                        game.step as f32 * STEP_HEIGHT,
                        offset + (i as f32 * CELL_SIZE),
                    ),
                    scale: 1.0,
                    color: Color::hsla(
                        ((ii * i) % (game.size.pow(2) / 4)) as f32 / (game.size.pow(2) / 4) as f32
                            * 360.,
                        1.0,
                        0.5,
                        1.,
                    )
                    .as_rgba_f32(),
                });
            }
        }
    }
    game.state = next_tile_state;
}

fn handle_reset(
    keys: Res<ButtonInput<KeyCode>>,
    mut instance_query: Query<&mut InstanceMaterialData>,
    mut game: ResMut<Game>,
) {
    let mut instance = instance_query.single_mut();
    if keys.just_pressed(KeyCode::KeyR) {
        instance.0 = Vec::new();
        game.state = create_random_state(game.size);
        game.step = 0;
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut game: ResMut<Game>,
    mut instance_query: Query<&mut InstanceMaterialData>,
) {
    let mut instance = instance_query.single_mut();
    let ctx = contexts.ctx_mut();

    egui::Window::new("side_panel").show(ctx, |ui| {
        ui.heading("Controls");

        let mut job = LayoutJob::default();
        job.append(
            "\nMove with ",
            0.0,
            TextFormat {
                color: Color32::WHITE,
                ..Default::default()
            },
        );
        job.append(
            "WASD + Space + LShift",
            0.0,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                ..Default::default()
            },
        );
        ui.label(job);

        let mut job = LayoutJob::default();
        job.append(
            "\nPress ",
            0.0,
            TextFormat {
                color: Color32::WHITE,
                ..Default::default()
            },
        );
        job.append(
            "ESC",
            0.0,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                ..Default::default()
            },
        );
        job.append(
            " to toggle mouse and keyboard capture",
            0.0,
            TextFormat {
                color: Color32::WHITE,
                ..Default::default()
            },
        );
        ui.label(job);

        let mut job = LayoutJob::default();
        job.append(
            "\nPress ",
            0.0,
            TextFormat {
                color: Color32::WHITE,
                ..Default::default()
            },
        );
        job.append(
            "r",
            0.0,
            TextFormat {
                color: Color32::LIGHT_GREEN,
                ..Default::default()
            },
        );
        job.append(
            " to reset the game\n",
            0.0,
            TextFormat {
                color: Color32::WHITE,
                ..Default::default()
            },
        );
        ui.label(job);

        if ui.button("Reset Game").clicked() {
            instance.0 = Vec::new();
            game.state = create_random_state(game.size);
            game.step = 0;
        }

        let before = game.size;
        ui.add(egui::Slider::new(&mut game.size, 4..=256).text("Game Size"));
        if game.size % 2 != 0 {
            game.size += 1;
        }
        if before != game.size {
            instance.0 = Vec::new();
            game.state = create_random_state(game.size);
            game.step = 0;
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add(egui::Hyperlink::from_label_and_url(
                "read my original blog post + code here",
                "https://silasmarvin.dev/conways-game-of-life-through-time",
            ));
        });
    });
}
