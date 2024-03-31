use bevy::{prelude::*, render::view::NoFrustumCulling};
use rand::Fill;

mod custom_material_plugin;

use custom_material_plugin::{CustomMaterialPlugin, InstanceData, InstanceMaterialData};

const GAME_SIZE: usize = 128;
const STEP_HEIGHT: f32 = 0.35;
const CELL_SIZE: f32 = 1.;

#[derive(Resource)]
struct GameState(Box<[[bool; GAME_SIZE]; GAME_SIZE]>);

fn create_random_state() -> Box<[[bool; GAME_SIZE]; GAME_SIZE]> {
    let mut tiles = Box::new([[true; GAME_SIZE]; GAME_SIZE]);
    let mut rng = rand::thread_rng();
    for i in 0..GAME_SIZE {
        tiles[i].try_fill(&mut rng).unwrap();
    }
    tiles
}

impl GameState {
    fn new() -> Self {
        Self(create_random_state())
    }
}

#[derive(Resource)]
struct GameStep(usize);

impl GameStep {
    fn new() -> Self {
        Self(0)
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, CustomMaterialPlugin))
        .insert_resource(GameState::new())
        .insert_resource(GameStep::new())
        .add_systems(Startup, setup)
        .add_systems(Update, handle_reset)
        .add_systems(Update, step.after(handle_reset))
        .add_systems(Update, move_camera)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        meshes.add(Cuboid::new(CELL_SIZE, STEP_HEIGHT, CELL_SIZE)),
        SpatialBundle::INHERITED_IDENTITY,
        InstanceMaterialData(Vec::new()),
        NoFrustumCulling,
    ));

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(
            0.0,
            GAME_SIZE as f32 / 5.,
            GAME_SIZE as f32 + (GAME_SIZE as f32 / 4.),
        )
        .looking_at(
            Vec3 {
                x: 0.,
                y: -1. * GAME_SIZE as f32 / 3.,
                z: 0.,
            },
            Vec3::Y,
        ),
        ..default()
    });
}

fn step(
    mut game_state: ResMut<GameState>,
    mut game_step: ResMut<GameStep>,
    mut instance_query: Query<&mut InstanceMaterialData>,
) {
    game_step.0 += 1;

    let mut instance = instance_query.single_mut();

    let mut next_tile_state = game_state.0.clone();
    let offset = (GAME_SIZE / 2) as f32 * CELL_SIZE * -1.;
    // z is i
    // x is ii
    for i in 0..GAME_SIZE {
        for ii in 0..GAME_SIZE {
            let check_row = |row: usize, check_center: bool| {
                let mut n_count = 0;
                if ii > 0 && game_state.0[row][ii - 1] {
                    n_count += 1;
                }
                if check_center && game_state.0[row][ii] {
                    n_count += 1;
                }
                if ii + 1 < GAME_SIZE && game_state.0[row][ii + 1] {
                    n_count += 1;
                }
                n_count
            };
            let n_top_count = if i > 0 { check_row(i - 1, true) } else { 0 };
            let n_middle_count = check_row(i, false);
            let bottom_n = if i + 1 < GAME_SIZE {
                check_row(i + 1, true)
            } else {
                0
            };
            let alive_n = n_top_count + n_middle_count + bottom_n;

            if game_state.0[i][ii] && alive_n < 2 {
                next_tile_state[i][ii] = false;
            }
            if game_state.0[i][ii] && (alive_n == 2 || alive_n == 3) {
                next_tile_state[i][ii] = true;
            }
            if game_state.0[i][ii] && alive_n > 3 {
                next_tile_state[i][ii] = false;
            }
            if !game_state.0[i][ii] && alive_n == 3 {
                next_tile_state[i][ii] = true;
            }

            if game_state.0[i][ii] {
                instance.push(InstanceData {
                    position: Vec3::new(
                        offset + (ii as f32 * CELL_SIZE),
                        game_step.0 as f32 * STEP_HEIGHT,
                        offset + (i as f32 * CELL_SIZE),
                    ),
                    scale: 1.0,
                    // color: Color::hsla((ii as f32 / GAME_SIZE as f32) * 360., 1., 0.5, 0.85)
                    // color: Color::hsla(
                    //     (ii as f32 / GAME_SIZE as f32) * 360.,
                    //     (i as f32 / GAME_SIZE as f32) * 0.3 + 0.7,
                    //     0.5,
                    //     0.85,
                    // )
                    // .as_rgba_f32(),
                    // color: Color::hsla(
                    //     // (ii as f32 / GAME_SIZE as f32) * 180. + (game_step.0 % 180) as f32,
                    //     (ii as f32 / GAME_SIZE as f32) * 360.,
                    //     (i as f32 / GAME_SIZE as f32) * 0.5 + 0.5,
                    //     0.5,
                    //     0.5,
                    // )
                    // .as_rgba_f32(),
                    color: Color::hsla(
                        // (ii as f32 / GAME_SIZE as f32) * 180. + (game_step.0 % 180) as f32,
                        // ((ii + i) % (GAME_SIZE / 2)) as f32 / (GAME_SIZE / 2) as f32 * 360.,
                        ((ii * i) % (GAME_SIZE.pow(2) / 4)) as f32 / (GAME_SIZE.pow(2) / 4) as f32
                            * 360.,
                        // ((ii * i) as f32 / (GAME_SIZE * GAME_SIZE) as f32) * 360.,
                        // (i as f32 / GAME_SIZE as f32) * 0.5 + 0.5,
                        1.0,
                        0.5,
                        1.,
                    )
                    .as_rgba_f32(),
                    // color: Color::hsla(
                    //     // (ii as f32 / GAME_SIZE as f32) * 180. + (game_step.0 % 180) as f32,
                    //     // ((ii + i) % (GAME_SIZE / 2)) as f32 / (GAME_SIZE / 2) as f32 * 360.,
                    //     // ((ii * i) % (GAME_SIZE.pow(2) / 4)) as f32 / (GAME_SIZE.pow(2) / 4) as f32
                    //     //     * 360.,
                    //     ((ii * GAME_SIZE) + i) as f32 / GAME_SIZE.pow(2) as f32 * 360.,
                    //     // ((ii * i) as f32 / (GAME_SIZE * GAME_SIZE) as f32) * 360.,
                    //     // (i as f32 / GAME_SIZE as f32) * 0.5 + 0.5,
                    //     1.0,
                    //     0.5,
                    //     1.,
                    // )
                    // .as_rgba_f32(),
                });
            }
        }
    }
    game_state.0 = next_tile_state;
}

fn move_camera(mut c_q: Query<(&Camera, &mut Transform)>, time: Res<Time>) {
    let (_camera, mut c_transform) = c_q.single_mut();
    c_transform.translation.y += STEP_HEIGHT;
    let y = c_transform.translation.y;
    c_transform.rotate_around(
        Vec3 { x: 0., y, z: 0. },
        Quat::from_rotation_y(time.delta_seconds() / 2.),
    );
}

fn handle_reset(
    keys: Res<ButtonInput<KeyCode>>,
    mut instance_query: Query<&mut InstanceMaterialData>,
    mut game_state: ResMut<GameState>,
) {
    let mut instance = instance_query.single_mut();
    if keys.just_pressed(KeyCode::KeyR) {
        instance.0 = Vec::new();
        game_state.0 = create_random_state();
    }
}
