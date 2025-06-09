use core::usize;

use bevy::prelude::*;

use crate::ball_type::Team;
use crate::game_state::MyGameState;
use crate::level::{LEVELS, LevelData, LevelSpawner, PlayerDirection};
use crate::physic::PhysicObject;
use crate::sound_manager::SoundManager;
use crate::text::{Size, Text, TextContent};
use crate::{fade_transition, level::Level};

#[derive(Resource, Default)]
pub struct WantedLevel(pub Option<usize>, pub Option<usize>);

#[derive(Resource, Default)]
pub struct CurrentLevel {
    level_index: Option<usize>,

    pub player_entity: Option<Entity>,
    pub player_ball_selected: Option<Entity>,
}

impl CurrentLevel {
    pub fn id(&self) -> Option<usize> {
        self.level_index.clone()
    }

    pub fn data(&self) -> Option<&'static LevelData> {
        self.level_index.map(|id| LEVELS.get(id)).flatten()
    }
}

#[derive(Resource, Default, Clone)]
pub struct InGameData {
    time_started_pressing_to_rotate: f32,
    player_start_press_to_fire_time: Option<f32>,
    last_ball_fire_time: f64,

    next_player_ball_to_use: usize,
    nb_ball_fired: usize,

    text_success_fail_added_time: Option<f64>,
    is_success: bool,

    time_since_level_start: f64,
    stabilized: bool,

    balls_text: Option<Entity>,
    levels_text: Option<Entity>,
}

pub fn init_state_in_game_system(app: &mut App) {
    app.init_resource::<WantedLevel>();
    app.init_resource::<CurrentLevel>();
    app.add_systems(OnEnter(MyGameState::InGame), in_game_enter)
        .add_systems(OnExit(MyGameState::InGame), in_game_exit)
        .add_systems(
            Update,
            (
                exec_load_level,
                check_stabilized,
                update_text,
                player_control,
                spawn_player_ball_update,
                detect_finish_level,
            )
                .chain()
                .run_if(in_state(MyGameState::InGame)),
        );
}

fn in_game_enter(
    mut commands: Commands,
    wanted_level: Option<Res<WantedLevel>>,
    mut sound_manager: SoundManager,
) {
    if wanted_level.is_none() {
        commands.insert_resource(WantedLevel(Some(0), Some(0)));
    }

    let balls_text = commands
        .spawn((
            Transform::from_xyz(-2., 4., 0.),
            children![(
                Transform::IDENTITY,
                Text {
                    size: Size::Small,
                    alignment: agb::display::object::TextAlignment::Right,
                    ..default()
                }
            )],
        ))
        .id()
        .into();
    // let levels_text = commands
    //     .spawn((
    //         Transform::from_xyz(2., 4., 0.),
    //         children![(
    //             Transform::IDENTITY,
    //             Text {
    //                 size: Size::Small,
    //                 alignment: agb::display::object::TextAlignment::Left,
    //                 ..default()
    //             }
    //         )],
    //     ))
    //     .id()
    //     .into();

    commands.insert_resource(InGameData {
        time_started_pressing_to_rotate: 0.0,
        player_start_press_to_fire_time: None,
        last_ball_fire_time: -666.666,
        next_player_ball_to_use: 0,
        nb_ball_fired: 0,
        balls_text,
        levels_text:None,
        ..Default::default()
    });

    sound_manager.change_main_sound(sound_manager.sound_list.main_menu_sound, 2);
}

fn exec_load_level(
    mut commands: Commands,
    mut load_level_request: ResMut<WantedLevel>,
    level_entities: Query<Entity, With<Level>>,
    mut next_state: ResMut<NextState<MyGameState>>,
    mut level_spawner: LevelSpawner,
    in_game_data: ResMut<InGameData>,
    fade_external_data: ResMut<fade_transition::FadeExternalData>,
    time: Res<Time>,
) {
    //info!("exec_load_level");

    if level_spawner.current_level.level_index == load_level_request.0
        && load_level_request.1 == None
    {
        return;
    }

    let fading_data = fade_external_data.into_inner();
    if fading_data.is_current_transitioning || fading_data.request.request_valid {
        return;
    }

    // info!("Despawning");

    // Despawn existing level
    for entity in level_entities {
        level_spawner.commands.entity(entity).despawn();
    }

    level_spawner.current_level.player_ball_selected = None;
    level_spawner.current_level.player_entity = None;

    let level_id = load_level_request.0;

    let cached_text_ball_entity = in_game_data.balls_text;
    let cached_text_levels_entity = in_game_data.levels_text;

    commands.insert_resource(InGameData {
        time_started_pressing_to_rotate: 0.0,
        player_start_press_to_fire_time: None,
        last_ball_fire_time: -666.666,
        next_player_ball_to_use: 0,
        nb_ball_fired: 0,
        text_success_fail_added_time: None,
        time_since_level_start: time.elapsed_secs_f64(),
        balls_text: cached_text_ball_entity,
        levels_text: cached_text_levels_entity,
        ..Default::default()
    });

    if let Some(level_to_load) = level_id.map(|id| LEVELS.get(id)).flatten() {
        // info!("Loading level {level_id}");
        level_spawner.spawn_initial(level_to_load);
        level_spawner.current_level.level_index = load_level_request.0;
        load_level_request.1 = None;

        fading_data.request.request_valid = true;
        fading_data.request.is_fade_in = true;
        fading_data.request.speed = fade_transition::TransitionSpeed::Fast;
        fading_data.request.transition_type = fade_transition::FadeTransitionType::Vertical;
    } else {
        // info!("Going to credit");
        level_spawner.current_level.level_index = None;
        next_state.set(MyGameState::Credit);
    }
}

fn get_ball_string(current_ball: usize, max_ball :usize) -> TextContent { 
    match (current_ball, max_ball) {
        (0,3) => TextContent::Ref("Balls: 0/3 "),
        (1,3) => TextContent::Ref("Balls: 1/3 "),
        (2,3) => TextContent::Ref("Balls: 2/3 "),
        (3,3) => TextContent::Ref("Balls: 3/3 "),

        (0,5) => TextContent::Ref("Balls: 0/5 "),
        (1,5) => TextContent::Ref("Balls: 1/5 "),
        (2,5) => TextContent::Ref("Balls: 2/5 "),
        (3,5) => TextContent::Ref("Balls: 3/5 "),
        (4,5) => TextContent::Ref("Balls: 4/5 "),
        (5,5) => TextContent::Ref("Balls: 5/5 "),

        _ => TextContent::No
    }
}

fn update_text(
    texts: Query<(Entity, &ChildOf, &Text)>,
    in_game_data: Res<InGameData>,
    current_level: Res<CurrentLevel>,
    mut commands: Commands,
    time: Res<Time>,
) {
    // Settings: For how long we will display the level title.
    let should_display_level_title =
        (time.elapsed_secs_f64() - in_game_data.time_since_level_start) < 2.5;
    //info!("\n - should_display_level_title: {} \n - elapsed_secs_f64: {} \n - in_game_data.time_since_level_start: {} \n - Delta: {}", should_display_level_title, time.elapsed_secs_f64(), in_game_data.time_since_level_start, (time.elapsed_secs_f64() - in_game_data.time_since_level_start));

    let mut is_level_finish = false;

    if let Some(time_when_adding_finish_text) = in_game_data.text_success_fail_added_time {
        is_level_finish = true;
    }

    for (entity, child_of, text) in texts {
        let parent = Some(child_of.parent());
        if parent == in_game_data.balls_text {
            let alignment = if is_level_finish || should_display_level_title {
                agb::display::object::TextAlignment::Center
            } else {
                agb::display::object::TextAlignment::Right
            };

            let new_text = if is_level_finish {
                if in_game_data.is_success {
                    TextContent::Ref("Success ! ")
                } else {
                    TextContent::Ref("Fail, No balls left :( ")
                }
            } else if should_display_level_title {
                TextContent::Ref(current_level.data().map_or("", |data|data.title))
            } else {
                let total_ball = current_level
                    .data()
                    .map_or(1, |level| level.player_balls.len());
                let ball_number = total_ball - in_game_data.nb_ball_fired;
                get_ball_string(ball_number, total_ball)
            };

            let Some(mut text_comp) = text.update(new_text) else {
                continue;
            };

            text_comp.alignment = alignment;
            // info!("Update ball with {new_text}");
            commands.entity(entity).insert(text_comp);
        } 
        // else if parent == in_game_data.levels_text
        //     && let Some(current_level) = current_level.id()
        // {
        //     // let new_text = if is_level_finish || should_display_level_title {
        //     //     ""
        //     // } else {
        //     //     let shown_level = current_level + 1;
        //     //     let total_level = LEVELS.len();
        //     //     &format!("Levels: {shown_level}/{total_level}")
        //     // };

        //     // let Some(new_comp) = text.update(new_text) else {
        //     //     continue;
        //     // };

        //     // // info!("Update levels with {new_text}");
        //     // commands.entity(entity).insert(new_comp);
        // }
    }
}

fn player_control(
    gamepad: Single<&Gamepad>,
    time: Res<Time>,
    mut runtime_data: ResMut<InGameData>,
    mut current_level: ResMut<CurrentLevel>,
    mut transforms: Query<&mut Transform>,
    mut physic_objects: Query<&mut PhysicObject>,
) {
    if current_level.data().is_none() {
        return;
    }

    if let Some(_time_when_adding_finish_text) = runtime_data.text_success_fail_added_time {
        let impulse_force_ratio = get_impulse_ratio_based_on_input(&time, &runtime_data);
        update_player_ball_selected_position(transforms, &current_level, impulse_force_ratio);
        return;
    }

    let mut rotation_left_button = GamepadButton::DPadRight;
    let mut rotation_right_button = GamepadButton::DPadRight;

    let mut rotation_angle_min = 0.0;
    let mut rotation_angle_max = 0.0;

    let max_side_angle = 0.2;

    let level_data = current_level.data().unwrap();
    match level_data.player_direction {
        PlayerDirection::Bottom => {
            rotation_left_button = GamepadButton::DPadLeft;
            rotation_right_button = GamepadButton::DPadRight;

            rotation_angle_min = -max_side_angle;
            rotation_angle_max = max_side_angle;
        }
        PlayerDirection::Top => {
            rotation_left_button = GamepadButton::DPadRight;
            rotation_right_button = GamepadButton::DPadLeft;

            rotation_angle_min = -(0.5 - max_side_angle);
            rotation_angle_max = 0.5 - max_side_angle;
        }
        PlayerDirection::Left => {
            rotation_left_button = GamepadButton::DPadUp;
            rotation_right_button = GamepadButton::DPadDown;

            rotation_angle_min = (-0.5 / 2.0) - max_side_angle;
            rotation_angle_max = (-0.5 / 2.0) + max_side_angle;
        }
        PlayerDirection::Right => {
            rotation_left_button = GamepadButton::DPadDown;
            rotation_right_button = GamepadButton::DPadUp;

            rotation_angle_min = (0.5 / 2.0) - max_side_angle;
            rotation_angle_max = (0.5 / 2.0) + max_side_angle;
        }
    }

    if gamepad.just_pressed(rotation_left_button) || gamepad.just_pressed(rotation_right_button) {
        runtime_data.time_started_pressing_to_rotate = time.elapsed_secs();
    }

    let time_since_pressing_to_rotate =
        time.elapsed_secs() - runtime_data.time_started_pressing_to_rotate;

    // Settings: Player controller rotation speed multiplier the longer you press.
    let rotation_speed_multiplier = remap(time_since_pressing_to_rotate, 0.0, 2.0, 1.0, 4.0);
    //info!("Time Since: {} | rotation_speed_multiplier: {}", time_since_pressing_to_rotate, rotation_speed_multiplier);

    // Settings: Player controller rotation speed.
    let player_rotation_speed = 0.1 * rotation_speed_multiplier;

    match level_data.player_direction {
        PlayerDirection::Top => {
            if gamepad.pressed(rotation_left_button) {
                if let Some(player_id) = current_level.player_entity {
                    let Ok(mut player_entity_transform) = transforms.get_mut(player_id) else {
                        return;
                    };

                    let rotation_change = player_rotation_speed * time.delta_secs();
                    let mut new_rotation =
                        player_entity_transform.rotation.to_euler(EulerRot::XYZ).2
                            + rotation_change;
                    if new_rotation > 0.5 {
                        new_rotation = -0.5 + (new_rotation - 0.5);
                    }

                    if new_rotation > rotation_angle_min && new_rotation < rotation_angle_max {
                        new_rotation = rotation_angle_min;
                    }

                    player_entity_transform.rotation =
                        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, new_rotation);
                }
            }

            if gamepad.pressed(rotation_right_button) {
                if let Some(player_id) = current_level.player_entity {
                    let Ok(mut player_entity_transform) = transforms.get_mut(player_id) else {
                        return;
                    };

                    let rotation_change = player_rotation_speed * time.delta_secs();
                    let mut new_rotation =
                        player_entity_transform.rotation.to_euler(EulerRot::XYZ).2
                            - rotation_change;
                    if new_rotation < -0.5 {
                        new_rotation = 0.5 - (new_rotation.abs() - 0.5);
                    }

                    if new_rotation > rotation_angle_min && new_rotation < rotation_angle_max {
                        new_rotation = rotation_angle_max;
                    }

                    player_entity_transform.rotation =
                        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, new_rotation);
                }
            }
        }
        PlayerDirection::Bottom | PlayerDirection::Right | PlayerDirection::Left => {
            if gamepad.pressed(rotation_left_button) {
                if let Some(player_id) = current_level.player_entity {
                    let Ok(mut player_entity_transform) = transforms.get_mut(player_id) else {
                        return;
                    };

                    let rotation_change = player_rotation_speed * time.delta_secs();
                    let mut new_rotation =
                        player_entity_transform.rotation.to_euler(EulerRot::XYZ).2
                            + rotation_change;
                    new_rotation = new_rotation.clamp(rotation_angle_min, rotation_angle_max);

                    player_entity_transform.rotation =
                        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, new_rotation);
                }
            }

            if gamepad.pressed(rotation_right_button) {
                if let Some(player_id) = current_level.player_entity {
                    let Ok(mut player_entity_transform) = transforms.get_mut(player_id) else {
                        return;
                    };

                    let rotation_change = player_rotation_speed * time.delta_secs();
                    let mut new_rotation =
                        player_entity_transform.rotation.to_euler(EulerRot::XYZ).2
                            - rotation_change;
                    new_rotation = new_rotation.clamp(rotation_angle_min, rotation_angle_max);

                    player_entity_transform.rotation =
                        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, new_rotation);
                }
            }
        }
    }

    if gamepad.just_pressed(GamepadButton::East) {
        runtime_data.player_start_press_to_fire_time = Some(time.elapsed_secs());
    } else if gamepad.just_released(GamepadButton::East) {
        if let Some(player_id) = current_level.player_entity {
            let Ok(player_entity_transform) = transforms.get_mut(player_id) else {
                return;
            };

            if let Some(player_ball_selected) = current_level.player_ball_selected {
                let player_forward = get_player_controller_forward(player_entity_transform);
                let impulse_force = get_impulse_force_based_on_input(&time, &runtime_data);
                let impulse = player_forward * impulse_force;

                //info!("\n - Impulse Ratio: {} | Impuse Force: {}",get_impulse_ratio_based_on_input(&time, &runtime_data),impulse_force);

                let ball_physic_object_result = physic_objects.get_mut(player_ball_selected);
                let mut ball_physic_object = ball_physic_object_result.unwrap();
                ball_physic_object.impulse = Vec2::new(impulse.x, impulse.y);
                ball_physic_object.enable = true;
                current_level.player_ball_selected = None;
                runtime_data.player_start_press_to_fire_time = None;
                runtime_data.last_ball_fire_time = time.elapsed_secs_f64();

                runtime_data.nb_ball_fired += 1;
            }
        }
    }

    let impulse_force_ratio = get_impulse_ratio_based_on_input(&time, &runtime_data);
    update_player_ball_selected_position(transforms, &current_level, impulse_force_ratio);
}

fn spawn_player_ball_update(
    time: Res<Time>,
    mut transforms: Query<&mut Transform>,
    mut runtime_data: ResMut<InGameData>,
    mut level_spawner: LevelSpawner,
) {
    let elapsed_time_since_last_ball_fire =
        time.elapsed_secs_f64() - runtime_data.last_ball_fire_time;
    if elapsed_time_since_last_ball_fire < 1.0 {
        return;
    }

    let current_level = &level_spawner.current_level;
    if let Some(_player_ball_selected) = current_level.player_ball_selected {
        return;
    }

    // info!("spawn_player_ball_update_5");
    let Some(level_data) = current_level.data() else {
        return;
    };
    let level_nb_player_ball = level_data.player_balls.len();
    if runtime_data.next_player_ball_to_use >= level_nb_player_ball {
        return;
    }

    let target_position =
        get_ball_position_on_player_controller(&mut transforms, current_level, 0.0);

    // info!("New Player Ball Spawned!");
    level_spawner.spawn_player_ball_at_index(
        level_data,
        runtime_data.next_player_ball_to_use,
        target_position,
    );
    runtime_data.next_player_ball_to_use += 1;
}

fn update_player_ball_selected_position(
    mut transforms: Query<&mut Transform>,
    current_level: &CurrentLevel,
    impulse_ratio: f32,
) {
    let target_position =
        get_ball_position_on_player_controller(&mut transforms, current_level, impulse_ratio);

    if let Some(_player_ball_selected) = current_level.player_ball_selected {
        let Ok(mut player_ball_entity_transform) = transforms.get_mut(_player_ball_selected) else {
            return;
        };

        player_ball_entity_transform.translation = target_position;
    }
}

fn get_ball_position_on_player_controller(
    transforms: &mut Query<&mut Transform>,
    current_level: &CurrentLevel,
    impulse_ratio: f32,
) -> Vec3 {
    if let Some(player_id) = current_level.player_entity {
        let Ok(player_entity_transform) = transforms.get_mut(player_id) else {
            return Vec3::ZERO;
        };

        // Settings: Ball offset to throw.
        let ball_front_player_offset_min_impulse = 3.0;
        let ball_front_player_offset_max_impulse = 15.0;

        let ratio_inverse = 1.0 - impulse_ratio;
        let ball_front_player_offset = f32::lerp(
            ball_front_player_offset_min_impulse,
            ball_front_player_offset_max_impulse,
            ratio_inverse,
        );

        let player_center_offset = Vec3::new(16.0, 16.0, 0.0);
        let ball_center_offset = Vec3::new(-8.0, -8.0, 0.0);
        let player_translation = player_entity_transform.translation;
        let player_translation_center = player_translation + player_center_offset;
        let player_forward = get_player_controller_forward(player_entity_transform);

        return player_translation_center
            + ball_center_offset
            + player_forward * ball_front_player_offset;
    }

    return Vec3::ZERO;
}

fn remap(value: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    let minMaxRange1 = max1 - min1;
    let valueRatioInRange = f32::clamp((value - min1) / minMaxRange1, 0.0, 1.0);

    return min2 + (max2 - min2) * valueRatioInRange;
}

fn remap_rotation(value: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    let minMaxRange1 = max1 - min1;
    let valueRatioInRange = f32::clamp((value - min1) / minMaxRange1, 0.0, 1.0);

    return (max2 - min2) * valueRatioInRange;
}

fn get_impulse_force_based_on_input(time: &Res<Time>, runtime_data: &ResMut<InGameData>) -> f32 {
    // Settings: Throw Impulse.
    let max_power = 190.0;
    let min_power = 80.0;

    let impulse_force_ratio = get_impulse_ratio_based_on_input(time, runtime_data);
    return f32::lerp(min_power, max_power, impulse_force_ratio);
}

fn get_impulse_ratio_based_on_input(time: &Res<Time>, runtime_data: &ResMut<InGameData>) -> f32 {
    let mut impulse_force_ratio = 0.0;

    if let Some(time_since_press) = runtime_data.player_start_press_to_fire_time {
        let elapsed_secs = time.elapsed_secs();
        let press_elapsed_time = elapsed_secs - time_since_press;

        // Settings: Throw Time.
        let max_power_time = 1.0;
        let min_power_time = 0.0;

        impulse_force_ratio = remap(press_elapsed_time, min_power_time, max_power_time, 0.0, 1.0);
        /*info!(
            "\n -- elapsed_secs: {} | time_since_press: {} | press_elapsed_time: {} | impulse_force_ratio: {}",
            elapsed_secs, time_since_press, press_elapsed_time, impulse_force_ratio
        );*/
    }
    return impulse_force_ratio;
}

fn get_player_controller_translation(
    transforms: Query<&mut Transform>,
    current_level: &CurrentLevel,
) -> Vec3 {
    if let Some(player_id) = current_level.player_entity {
        let Ok(player_entity_transform) = transforms.get(player_id) else {
            return Vec3::ZERO;
        };

        return player_entity_transform.translation;
    }

    return Vec3::ZERO;
}

fn get_player_controller_forward(player_entity_transform: Mut<Transform>) -> Vec3 {
    let player_rotation = player_entity_transform.rotation.to_euler(EulerRot::XYZ).2;
    let player_rotation_converted = remap_rotation(player_rotation, -0.5, 0.5, 3.1416, -3.1416);

    let direction_forward = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, player_rotation_converted)
        .mul_vec3(Vec3::new(0.0, 1.0, 0.0));
    return direction_forward;
}

fn detect_finish_level(
    time: Res<Time>,
    enemies: Query<(Entity, &Team)>,
    current_level: Res<CurrentLevel>,
    mut wanted_level: ResMut<WantedLevel>,
    mut in_game_data: ResMut<InGameData>,
    fade_external_data: ResMut<fade_transition::FadeExternalData>,
) {
    if let Some(time_when_adding_finish_text) = in_game_data.text_success_fail_added_time {
        let elapsed_time_since_text_there = time.elapsed_secs_f64() - time_when_adding_finish_text;
        // Settings: Time after end level transition.
        if elapsed_time_since_text_there < 3.0 {
            return;
        }

        let fading_data = fade_external_data.into_inner();
        if fading_data.is_current_transitioning
            || !fading_data.is_fading_in
            || fading_data.request.request_valid
        {
            return;
        }

        if in_game_data.is_success {

            wanted_level.0 = Some(
                current_level
                    .level_index
                    .map(|id| id + 1)
                    .unwrap_or(usize::MAX),
            );

            // info!("Victory! next level is {next_level}");
        } else {
            wanted_level.1 = Some(wanted_level.1.map_or(0, |counter| counter + 1));
        }

        fading_data.request.request_valid = true;
        fading_data.request.is_fade_in = false;
        fading_data.request.speed = fade_transition::TransitionSpeed::Medium;
        fading_data.request.transition_type = fade_transition::FadeTransitionType::Vertical;
        return;
    }

    let enemy_count = enemies.iter().filter(|(_, t)| t.is_enemy()).count();
    if enemy_count == 0 {
        in_game_data.text_success_fail_added_time = Some(time.elapsed_secs_f64());
        in_game_data.is_success = true;
    } else if in_game_data.stabilized
        && in_game_data.nb_ball_fired == current_level.data().unwrap().player_balls.len()
    {
        in_game_data.text_success_fail_added_time = Some(time.elapsed_secs_f64());
        in_game_data.is_success = false;
    }
}

fn in_game_exit(
    mut commands: Commands,
    level_entities: Query<Entity, With<Level>>,
    in_game_data: ResMut<InGameData>,
) {
    // info!("in_game_exit");

    // Despawn existing level
    for entity in level_entities {
        commands.entity(entity).despawn();
    }

    if let Some(text_entity) = in_game_data.balls_text {
        commands.entity(text_entity).despawn();
    }

    if let Some(text_entity) = in_game_data.levels_text {
        commands.entity(text_entity).despawn();
    }

    commands.insert_resource(CurrentLevel {
        level_index: None,
        player_entity: None,
        ..Default::default()
    });
}

fn check_stabilized(mut in_game_data: ResMut<InGameData>, objects: Query<&PhysicObject>) {
    in_game_data.stabilized = objects.iter().all(|po| po.velocity == Vec2::ZERO);
}
