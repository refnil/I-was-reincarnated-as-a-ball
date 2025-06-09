use agb::display::object::TextAlignment;
use bevy::prelude::*;

use super::super::text;
use crate::fade_transition;
use crate::game_state::MyGameState;
use crate::render::RepeatedSprite;
use crate::sound_manager::SoundManager;
use crate::sprite_loader::Sprites;
use crate::text::Size;
use crate::utils::{get_screen_center_position, get_screen_size};

#[derive(Resource, Default, Clone)]
pub struct CreditRuntimeData {
    is_transitioning_out: bool,
    request_sent_to_next_state: bool,

    text_to_quit_added: bool,
    text_thanks_added: bool,

    should_exit_button_visible: bool,
    exit_button_visibility_timer: f32,

    time_since_in_credit: f32,

    credit_text: Option<Entity>,
    thank_you_text: Option<Entity>,
    exit_text_entity: Option<Entity>,
    background: Option<Entity>,
}

pub fn game_state_credit(app: &mut App) {
    app.add_systems(OnEnter(MyGameState::Credit), credit_enter)
        .add_systems(OnExit(MyGameState::Credit), credit_exit)
        .add_systems(
            Update,
            (
                input_update,
                text_rolling_update,
                exit_button_visibility_update,
                transitioning_to_main_menu_update,
            )
                .chain()
                .run_if(in_state(MyGameState::Credit)),
        );
}

fn credit_enter(
    mut commands: Commands,
    mut fade_external_data: ResMut<fade_transition::FadeExternalData>,
    mut sound_manager: SoundManager,
    sprites: NonSend<Sprites>,
) {
    // info!("credit_enter");

    let background = commands
        .spawn((
            Transform::from_xyz(-7., -13., 0.),
            children![(
                Transform::IDENTITY,
                sprites.credit_background.clone(),
                RepeatedSprite {
                    cols: 4,
                    rows: 3,
                    x_size: 64,
                    y_size: 64,
                },
            )],
        ))
        .id()
        .into();

    let text_owner_axis_y_offset = 50.0;
    let between_text_offset = 35.0;

    let screen_size = get_screen_size();

    let y = screen_size.y + text_owner_axis_y_offset;

    let credit_text = commands
        .spawn((
            Transform::from_xyz(0.0, y, 0.0),
            text_list(
                -30.,
                between_text_offset,
                &[
                    "\u{E002}R and H ",
                    "\u{E002}CanariPack 8BIT ",
                    "\u{E002}QuinqueFive font  ",
                ],
            ),
        ))
        .id()
        .into();

    // Spawn our runtime data.
    commands.insert_resource(CreditRuntimeData {
        is_transitioning_out: false,
        request_sent_to_next_state: false,
        text_to_quit_added: false,
        text_thanks_added: false,
        should_exit_button_visible: true,
        exit_button_visibility_timer: 0.0,
        time_since_in_credit: 0.0,
        exit_text_entity: None,
        background,
        credit_text,
        thank_you_text: None,
    });

    sound_manager.change_main_sound(sound_manager.sound_list.credit_sound, 1);

    fade_external_data.request.request_valid = true;
    fade_external_data.request.is_fade_in = true;
    fade_external_data.request.speed = fade_transition::TransitionSpeed::Medium;
    fade_external_data.request.transition_type = fade_transition::FadeTransitionType::Vertical;
}

fn text_list(pos: f32, offset: f32, strs: &[&'static str]) -> impl Bundle {
    text_list_options(pos, offset, Size::Small, TextAlignment::Center, strs)
}

fn text_list_options(
    mut pos: f32,
    offset: f32,
    size: Size,
    alignment: TextAlignment,
    strs: &[&'static str],
) -> impl Bundle {
    Children::spawn(
        strs.iter()
            .map(|str| {
                let bundle = (
                    Transform::from_xyz(0.0, pos, 0.0),
                    text::Text {
                        size,
                        alignment,
                        text: text::TextContent::Ref(str)
                    },
                );
                pos += offset;
                bundle
            })
            .collect::<Vec<_>>(),
    )
}

fn credit_exit(mut commands: Commands, runtime_data: Res<CreditRuntimeData>) {
    // info!("credit_exit");

    let mut despawn = |entity: Option<Entity>| {
        entity
            .map(|entity| commands.get_entity(entity).ok())
            .flatten()
            .map(|mut entity| entity.despawn())
    };

    despawn(runtime_data.background);
    despawn(runtime_data.credit_text);
    despawn(runtime_data.exit_text_entity);
    despawn(runtime_data.thank_you_text);
}

fn input_update(
    mut commands: Commands,
    gamepad: Single<&Gamepad>,
    mut runtime_data: ResMut<CreditRuntimeData>,
    fade_external_data: ResMut<fade_transition::FadeExternalData>,
    mut sound_manager: SoundManager,
) {
    if runtime_data.is_transitioning_out {
        return;
    }

    if runtime_data.text_to_quit_added {
        if gamepad.just_pressed(GamepadButton::South) || 
            gamepad.just_pressed(GamepadButton::East) || 
            gamepad.just_pressed(GamepadButton::DPadUp) || 
            gamepad.just_pressed(GamepadButton::DPadDown) || 
            gamepad.just_pressed(GamepadButton::DPadLeft) ||
            gamepad.just_pressed(GamepadButton::DPadRight) || 
            gamepad.just_pressed(GamepadButton::Start) || 
            gamepad.just_pressed(GamepadButton::Select) || 
            gamepad.just_pressed(GamepadButton::LeftTrigger) || 
            gamepad.just_pressed(GamepadButton::RightTrigger) {
            sound_manager.play_sound_effect(sound_manager.sound_list.menu_cursor_select);

            runtime_data.is_transitioning_out = true;

            let fading_data = fade_external_data.into_inner();
            fading_data.request.request_valid = true;
            fading_data.request.is_fade_in = false;
            fading_data.request.speed = fade_transition::TransitionSpeed::Fast;
            fading_data.request.transition_type = fade_transition::FadeTransitionType::Vertical;

            runtime_data
                .credit_text
                .map(|entity| commands.get_entity(entity).ok())
                .flatten()
                .map(|mut entity| entity.despawn());
        }
    }
}

fn text_rolling_update(
    time: Res<Time>,
    mut commands: Commands,
    mut runtime_data: ResMut<CreditRuntimeData>,
    mut text_entities: Query<&mut Transform>,
) {
    runtime_data.time_since_in_credit += time.delta_secs();

    let screen_center = get_screen_center_position();
    let mut is_only_left_text_is_thanks = true;
    let mut is_thanks_text_reached_target = false;

    if let Some(entity) = runtime_data.thank_you_text
        && let Ok(mut transform) = text_entities.get_mut(entity)
    {
        // Thank you only
        transform.translation.y -= 25.0 * time.delta_secs();

        let thanks_target_end_position_y = screen_center.y - 20.0;

        if transform.translation.y < thanks_target_end_position_y {
            transform.translation.y = thanks_target_end_position_y;
            is_thanks_text_reached_target = true;
        }
    }

    if let Some(entity) = runtime_data.credit_text
        && let Ok(mut transform) = text_entities.get_mut(entity)
    {
        // Credit only

        let credit_texts_target_end_position_y = -50.0;
        if transform.translation.y < credit_texts_target_end_position_y {
            commands.entity(entity).despawn();
        } else {
            transform.translation.y -= 20.0 * time.delta_secs();
            is_only_left_text_is_thanks = false;
        }
    }

    let time_to_spawn_thanks = 10.0;
    if runtime_data.text_to_quit_added {
        return;
    } else if !runtime_data.text_thanks_added
        && runtime_data.time_since_in_credit >= time_to_spawn_thanks
    {
        let screen_size = get_screen_size();
        let text_owner_axis_y_offset = 10.0;

        runtime_data.thank_you_text = commands
            .spawn((
                Transform::from_xyz(0.0, screen_size.y + text_owner_axis_y_offset, 0.0),
                text_list(0., 0., &["\u{E002}Thank You For Playing "]),
            ))
            .id()
            .into();

        runtime_data.text_thanks_added = true;
        // info!("Added Text Thank You");
    } else if is_only_left_text_is_thanks && is_thanks_text_reached_target {
        let new_credit_text_owner = commands
            .spawn((
                Transform::from_xyz(0.0, 125.0, 0.0),
                text_list_options(
                    0.,
                    0.,
                    Size::Small,
                    TextAlignment::Right,
                    &["\u{E002}Press Any Input "],
                ),
            ))
            .id();

        runtime_data.text_to_quit_added = true;
        runtime_data.exit_text_entity = Some(new_credit_text_owner);
        // info!("Added Text To Quit");
    }
}

fn exit_button_visibility_update(
    time: Res<Time>,
    mut runtime_data: ResMut<CreditRuntimeData>,
    // all_texts: Query<&mut text::TextVisibility>,
    // all_children: Query<&mut Children, With<CreditTextOwner>>,
) {
    if runtime_data.exit_text_entity.is_none() {
        return;
    }

    runtime_data.exit_button_visibility_timer += time.delta_secs();
    let mut visibility_changed = false;

    if runtime_data.should_exit_button_visible {
        if runtime_data.exit_button_visibility_timer >= 0.8 {
            runtime_data.should_exit_button_visible = false;
            runtime_data.exit_button_visibility_timer = 0.0;
            visibility_changed = true;
        }
    } else if runtime_data.exit_button_visibility_timer >= 0.2 {
        runtime_data.should_exit_button_visible = true;
        runtime_data.exit_button_visibility_timer = 0.0;
        visibility_changed = true;
    }

    // if visibility_changed {
    //     let content = runtime_data.exit_text_entity.unwrap();
    //     for children in all_children.get(content).iter_mut() {
    //         for child in children.iter() {
    //             let text_result = all_texts.get_mut(child);
    //             if text_result.is_ok() {
    //                 let mut target_text = text_result.unwrap();
    //                 target_text.visible = runtime_data.should_exit_button_visible;
    //             }
    //         }
    //     }
    // }
}

fn transitioning_to_main_menu_update(
    mut next_state: ResMut<NextState<MyGameState>>,
    mut runtime_data: ResMut<CreditRuntimeData>,
    fade_external_data: ResMut<fade_transition::FadeExternalData>,
) {
    if !runtime_data.is_transitioning_out || runtime_data.request_sent_to_next_state {
        return;
    }

    let fading_data = fade_external_data.into_inner();
    if fading_data.is_current_transitioning || fading_data.request.request_valid {
        return;
    }

    next_state.set(MyGameState::MainMenu);
    runtime_data.request_sent_to_next_state = true;
}
