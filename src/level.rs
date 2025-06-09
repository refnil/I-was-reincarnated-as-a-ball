use bevy::{
    app::Plugin,
    ecs::{
        children,
        component::Component,
        system::{Commands, ResMut, SystemParam},
    },
    math::{Quat, Vec2, Vec3},
    prelude::*,
    transform::components::Transform,
};

use crate::render::AffineSprite;

use bevy::prelude::EulerRot;

use crate::game_state::game_state_in_game::CurrentLevel;
use crate::sprite_loader::Sprites;
use crate::{
    ball_type::{EnemyBall, PlayerBall},
    render::RepeatedSprite,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, _app: &mut bevy::app::App) {
    }
}

fn triangle(
    ball: EnemyBall,
    depth: usize,
    start: Vec2,
    half_side_step: Vec2,
    layer_step: Vec2,
) -> Vec<(EnemyBall, Vec2)> {
    let mut vec: Vec<(EnemyBall, Vec2)> = Vec::new();

    let mut row_start = start;

    for row in 0..depth {
        let mut pos = row_start;
        for _ in 0..row + 1 {
            vec.push((ball, pos));
            pos -= 2. * half_side_step;
        }

        row_start += half_side_step;
        row_start += layer_step;
    }

    vec
}

//start_pos: Vec2::new(120., 140.), // Bottom Center

//start_pos: Vec2::new(220., 80.),  // Right Center

// Top Center
// start_pos: Vec2::new(112., 32.),
// player_direction: PlayerDirection::Top,

// Left Center
// start_pos: Vec2::new(8., 70.),
// player_direction: PlayerDirection::Left,

pub const LEVELS: &[LevelData] = &[
    // LevelData {
    //     title: "Shoot the slime ",
    //     player_balls: &[PlayerBall::Boy; 3],
    //     enemy_balls: &[(EnemyBall::GreenBlob, Vec2::new(112., 120.))],
    //     // Top Center
    //     start_pos: Vec2::new(104., 24.),
    //     player_direction: PlayerDirection::Top,
    //     angle_width: 90.,
    // },
    LevelData {
        title: "It is on the side ",
        player_balls: &[PlayerBall::Boy, PlayerBall::Dog, PlayerBall::Princess],
        enemy_balls: &[
            (EnemyBall::GreenBlob, Vec2::new(40., 120.)),
            (EnemyBall::Tree, Vec2::new(112., 75.)),
        ],
        // Top Center
        start_pos: Vec2::new(104., 24.),
        player_direction: PlayerDirection::Top,
        angle_width: 90.,
    },
    LevelData {
        title: "Hold the button! ",
        player_balls: &[PlayerBall::Boy, PlayerBall::Dog, PlayerBall::Princess],
        enemy_balls: &[(EnemyBall::Snake, Vec2::new(200., 40.))],
        // Left Center
        start_pos: Vec2::new(8., 70.),
        player_direction: PlayerDirection::Left,
        angle_width: 90.,
    },
    // LevelData {
    //     title: "They want revenge ",
    //     player_balls: &[PlayerBall::Boy; 3],
    //     enemy_balls: &[
    //         (EnemyBall::GreenBlob, Vec2::new(120., 120.)),
    //         (EnemyBall::GreenBlob, Vec2::new(100., 130.)),
    //         (EnemyBall::GreenBlob, Vec2::new(140., 130.)),
    //     ],
    //     // Top Center
    //     start_pos: Vec2::new(104., 24.),
    //     player_direction: PlayerDirection::Top,
    //     angle_width: 90.,
    // },
    // LevelData {
    //     title: "Bowling! ",
    //     player_balls: &[PlayerBall::Boy; 5],
    //     enemy_balls: &[
    //         (EnemyBall::Snake, Vec2::new(140.0, 69.0)),
    //         (EnemyBall::GreenBlob, Vec2::new(160.0, 79.0)),
    //         (EnemyBall::GreenBlob, Vec2::new(160.0, 59.0)),
    //         (EnemyBall::GreenBlob, Vec2::new(180.0, 89.0)),
    //         // (EnemyBall::GreenBlob, Vec2::new(180.0, 69.0)),
    //         (EnemyBall::GreenBlob, Vec2::new(180.0, 49.0)),
    //         // (EnemyBall::GreenBlob, Vec2::new(200.0, 99.0)),
    //         // (EnemyBall::GreenBlob, Vec2::new(200.0, 79.0)),
    //         // (EnemyBall::GreenBlob, Vec2::new(200.0, 59.0)),
    //         // (EnemyBall::GreenBlob, Vec2::new(200.0, 39.0)),
    //     ],
    //     // Left Center
    //     start_pos: Vec2::new(8., 70.),
    //     player_direction: PlayerDirection::Left,
    //     angle_width: 90.,
    // },
    LevelData {
        title: "Shoot the snake ",
        player_balls: &[PlayerBall::Boy, PlayerBall::Dog, PlayerBall::Princess],
        enemy_balls: &[
            (EnemyBall::Snake, Vec2::new(112., 120.)),
            (EnemyBall::Tree, Vec2::new(112., 75.)),
        ],
        // Top Center
        start_pos: Vec2::new(104., 24.),
        player_direction: PlayerDirection::Top,
        angle_width: 90.,
    },
    // LevelData {
    //     title: "They want revenge 2 ",
    //     player_balls: &[PlayerBall::Boy; 5],
    //     enemy_balls: &[
    //         (EnemyBall::RedBlob, Vec2::new(120., 100.)),
    //         (EnemyBall::RedBlob, Vec2::new(90., 120.)),
    //         (EnemyBall::RedBlob, Vec2::new(150., 120.)),
    //     ],
    //     // Top Center
    //     start_pos: Vec2::new(104., 24.),
    //     player_direction: PlayerDirection::Top,
    //     angle_width: 90.,
    // },
    // LevelData {
    //     title: "Ssssssss ",
    //     player_balls: &[PlayerBall::Boy; 4],
    //     enemy_balls: &[
    //         (EnemyBall::Snake, Vec2::new(120., 120.)),
    //         (EnemyBall::Snake, Vec2::new(100., 130.)),
    //         (EnemyBall::Snake, Vec2::new(140., 130.)),
    //     ],
    //     // Top Center
    //     start_pos: Vec2::new(104., 24.),
    //     player_direction: PlayerDirection::Top,
    //     angle_width: 90.,
    // },
    LevelData {
        title: "The BOSS ",
        player_balls: &[PlayerBall::Boy, PlayerBall::Dog, PlayerBall::Boy, PlayerBall::Dog, PlayerBall::Princess],
        enemy_balls: &[
            (EnemyBall::Tree, Vec2::new(70., 75.)),
            (EnemyBall::Ghost, Vec2::new(120., 130.)),
            (EnemyBall::GreenBlob, Vec2::new(110., 110.)),
            (EnemyBall::Snake, Vec2::new(130., 110.)),
        ],
        // Top Center
        start_pos: Vec2::new(104., 24.),
        player_direction: PlayerDirection::Top,
        angle_width: 90.,
    },
];

#[derive(Clone, Copy)]
pub enum PlayerDirection {
    Bottom,
    Top,
    Left,
    Right,
}

pub struct LevelData {
    pub title: &'static str,
    pub player_balls: &'static [PlayerBall],
    pub enemy_balls: &'static [(EnemyBall, Vec2)],
    pub start_pos: Vec2,
    pub angle_width: f32,
    pub player_direction: PlayerDirection,
}

#[derive(Component, Default)]
pub struct Level;

#[derive(SystemParam)]
pub struct LevelSpawner<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub sprites: NonSend<'w, Sprites>,
    pub current_level: ResMut<'w, CurrentLevel>,
}

impl<'w, 's> LevelSpawner<'w, 's> {
    pub fn spawn_enemies_ball(&mut self, level_data: &LevelData) {
        let LevelSpawner {
            commands, sprites, ..
        } = self;

        for (ball, pos) in level_data.enemy_balls {
            commands.spawn((
                Transform::from_translation(pos.extend(0.)),
                ball.to_bundle(&sprites),
            ));
        }
    }

    pub fn spawn_player_ball(
        &mut self,
        level_data: &LevelData,
        player_ball_index: usize,
        target_spawn_position: Vec3,
    ) {
        let LevelSpawner {
            commands,
            sprites,
            current_level,
        } = self;

        if player_ball_index >= level_data.player_balls.len() {
            // info!(
            //     "player_ball_index out of bound! player_ball_index: {} | player_balls len: {}",
            //     player_ball_index,
            //     level_data.player_balls.len()
            // );
            return;
        }

        let player_ball_info = level_data.player_balls[player_ball_index];
        current_level.player_ball_selected = Some(
            commands
                .spawn((
                    Transform::from_translation(target_spawn_position),
                    player_ball_info.to_bundle(&sprites, false),
                ))
                .id(),
        );
    }

    fn spawn_player_controller(&mut self, level_data: &LevelData) {
        let LevelSpawner {
            commands,
            sprites,
            current_level,
        } = self;


        let player_rotation = match level_data.player_direction {
            PlayerDirection::Bottom => {
               Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0)
            }
            PlayerDirection::Top => {
                Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.5)
            }
            PlayerDirection::Left => {
                 Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, -0.25)
            }
            PlayerDirection::Right => {
                 Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.25)
            }
        };

        // info!(
        //     "Player Angle: {}",
        //     player_rotation.to_euler(EulerRot::XYZ).2
        // );

        let mut player_transform = Transform::IDENTITY;
        player_transform.translation =
            Vec3::new(level_data.start_pos.x, level_data.start_pos.y, 0.0);
        player_transform.rotation = player_rotation;

        current_level.player_entity = Some(
            commands
                .spawn((
                    Level,
                    player_transform,
                    children![(
                        Transform::IDENTITY,
                        sprites.character_controller.clone(),
                        AffineSprite::enabled(),
                        RepeatedSprite::default(),
                    )],
                ))
                .id(),
        );
    }

    fn spawn_background(&mut self) {
        let LevelSpawner {
            commands, sprites, ..
        } = self;

        commands.spawn((
            Transform::IDENTITY,
            Level,
            children![
                // Top Left
                (
                    Transform::from_translation(Vec3::new(0., 16., 0.)),
                    sprites.wall_top_left.clone(),
                    RepeatedSprite::default()
                ),
                // Top Right
                (
                    Transform::from_translation(Vec3::new(224., 16., 0.)),
                    sprites.wall_top_right.clone(),
                    RepeatedSprite::default()
                ),
                // Top
                (
                    Transform::from_translation(Vec3::new(16., 16., 0.)),
                    sprites.wall_top.clone(),
                    RepeatedSprite {
                        cols: 13,
                        x_size: 16,
                        ..Default::default()
                    }
                ),
                // Left
                (
                    Transform::from_translation(Vec3::new(0., 32., 0.)),
                    sprites.wall_left.clone(),
                    RepeatedSprite {
                        rows: 7,
                        y_size: 16,
                        ..Default::default()
                    }
                ),
                // Right
                (
                    Transform::from_translation(Vec3::new(224., 32., 0.)),
                    sprites.wall_right.clone(),
                    RepeatedSprite {
                        rows: 7,
                        y_size: 16,
                        ..Default::default()
                    }
                ),
                // Bottom Left
                (
                    Transform::from_translation(Vec3::new(0., 144., 0.)),
                    sprites.wall_bottom_left.clone(),
                    RepeatedSprite::default()
                ),
                // Bottom Right
                (
                    Transform::from_translation(Vec3::new(224., 144., 0.)),
                    sprites.wall_bottom_right.clone(),
                    RepeatedSprite::default()
                ),
                // Bottom
                (
                    Transform::from_translation(Vec3::new(16., 144., 0.)),
                    sprites.wall_bottom.clone(),
                    RepeatedSprite {
                        cols: 13,
                        x_size: 16,
                        ..Default::default()
                    }
                ),
                // Floor
                (
                    Transform::from_translation(Vec3::new(0., 0., 0.)),
                    sprites.floor.clone(),
                    RepeatedSprite {
                        cols: 4,
                        x_size: 64,
                        rows: 3,
                        y_size: 64
                    }
                ),
            ],
        ));
    }
    pub fn spawn_initial(&mut self, level_data: &LevelData) {
        self.spawn_player_controller(level_data);
        self.spawn_background();
        self.spawn_enemies_ball(level_data);
    }

    pub fn spawn_player_ball_at_index(
        &mut self,
        level_data: &LevelData,
        player_ball_index: usize,
        target_spawn_position: Vec3,
    ) {
        self.spawn_player_ball(level_data, player_ball_index, target_spawn_position);
    }
}
