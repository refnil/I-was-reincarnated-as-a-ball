use agb::display::object::ChangeColour;
use bevy::prelude::*;

use super::super::text;
use crate::text::TextContent;
use crate::Sprite;
use crate::fade_transition;
use crate::game_state::MyGameState;
use crate::game_state::game_state_in_game::WantedLevel;
use crate::render::RepeatedSprite;
use crate::sound_manager::SoundManager;
use crate::sprite_loader::Sprites;
use crate::utils::get_screen_center_position;

#[derive(Resource)]
pub struct MenuTextOwner {
    main_object_entity: Entity,
    start_game_cursor_entity: Entity,
    credit_cursor_entity: Entity,
}

#[derive(Resource, Default, Clone)]
pub struct MainMenuRuntimeData {
    is_selected_start_button: bool,

    should_selected_button_visible: bool,
    timer: f32,

    request_sent_to_next_state: bool,

    is_transitioning_out: bool,
    target_next_state: MyGameState,
}

pub fn init_state_main_menu_system(app: &mut App) {
    app.add_systems(OnEnter(MyGameState::MainMenu), main_menu_enter)
        .add_systems(OnExit(MyGameState::MainMenu), main_menu_exit)
        .add_systems(
            Update,
            (
                main_menu_input_update,
                main_menu_selection_cursor_animation_update,
                transitioning_to_next_state_update,
            )
                .chain()
                .run_if(in_state(MyGameState::MainMenu)),
        );
}

static start_game: &'static str = "\u{E002}Start Game ";
static title: &'static str = "\u{E002}I was reincarnated as a ball ";
static credit: &'static str = "Credit ";

fn main_menu_enter(
    mut commands: Commands,
    mut fade_external_data: ResMut<fade_transition::FadeExternalData>,
    sprites: NonSend<Sprites>,
    mut sound_manager: SoundManager,
) {
    // info!("main_menu_enter");

    let screen_center = get_screen_center_position();

    let mut start_game_cursor_owner = None;
    let mut credit_cursor_owner = None;

    let x = screen_center.x - 50.0;
    let y = screen_center.y + 25.;

    

    // Create the owner of the whole thing.
    let main_menu_owner = commands
        .spawn((
            Transform::from_xyz(x, y, 0.),
            children![
                (
                    Transform::from_xyz(-x, -y + 30., 0.),
                    text::Text {
                        size: text::Size::Medium,
                        alignment: agb::display::object::TextAlignment::Center,
                        text:TextContent::Ref(title),
                    },
                ),
                spawn_main_menu_target_text(Vec3::new(0.0, 0.0, 0.0), start_game),
                spawn_main_menu_target_text(Vec3::new(20.0, 20.0, 0.0), credit),
                (
                    Transform::from_xyz(-x - 16., -y - 16., 0.),
                    sprites.menu_background.clone(),
                    RepeatedSprite {
                        cols: 4,
                        rows: 3,
                        x_size: 64,
                        y_size: 64
                    }
                )
            ],
        ))
        .with_children(|related_commands| {
            // Selection Start Cursor
            start_game_cursor_owner = Some(
                related_commands
                    .spawn((
                        Transform::from_xyz(-7.0, -11.0, 0.0),
                        spawn_cursor(&sprites, 5),
                    ))
                    .id(),
            );

            // Selection Credit Cursor
            credit_cursor_owner = Some(
                related_commands
                    .spawn((Transform::from_xyz(8.0, 9., 0.0), spawn_cursor(&sprites, 3)))
                    .id(),
            );
        })
        .id();

    commands.insert_resource(MenuTextOwner {
        main_object_entity: main_menu_owner,
        start_game_cursor_entity: start_game_cursor_owner.unwrap(),
        credit_cursor_entity: credit_cursor_owner.unwrap(),
    });

    // Spawn our runtime data.
    commands.insert_resource(MainMenuRuntimeData {
        is_selected_start_button: true,
        should_selected_button_visible: true,
        timer: 0.0,
        request_sent_to_next_state: false,
        is_transitioning_out: false,
        target_next_state: MyGameState::InGame,
    });

    sound_manager.change_main_sound(sound_manager.sound_list.main_menu_sound, 1);

    // Fade In the Game.
    fade_external_data.request.request_valid = true;
    fade_external_data.request.is_fade_in = true;
    fade_external_data.request.speed = fade_transition::TransitionSpeed::Medium;
    fade_external_data.request.transition_type = fade_transition::FadeTransitionType::Vertical;
}

fn main_menu_exit(mut commands: Commands, menu_text_owner_entity: Res<MenuTextOwner>) {
    // info!("main_menu_exit");

    commands
        .entity(menu_text_owner_entity.main_object_entity)
        .despawn();
}

fn spawn_main_menu_target_text(position: Vec3, text_value: &'static str) -> impl Bundle {
    (
        Transform::from_translation(position),
        text::Text {
            size: text::Size::Small,
            ..text::Text::from(text_value)
        },
    )
}

fn spawn_cursor(sprites: &Sprites, middle_size: i32) -> impl Bundle {
    children![
        // Left
        (
            Transform::from_xyz(0., 0., 0.),
            bevy_mod_gba::Sprite {
                handle: sprites.selection_cursor_corner.handle.clone(),
                horizontal_flipped: false,
                vertical_flipped: false,
                visible: true,
                priority: agb::display::Priority::P1,
                graphics_mode: agb::display::object::GraphicsMode::Normal,
            },
            RepeatedSprite::default(),
        ),
        // Right
        (
            Transform::from_xyz(((middle_size + 1) * 16) as f32, 0., 0.),
            bevy_mod_gba::Sprite {
                handle: sprites.selection_cursor_corner.handle.clone(),
                horizontal_flipped: true,
                vertical_flipped: false,
                visible: true,
                priority: agb::display::Priority::P1,
                graphics_mode: agb::display::object::GraphicsMode::Normal,
            },
            RepeatedSprite::default(),
        ),
        // Center
        (
            Transform::from_xyz(16.0, 0., 0.),
            sprites.selection_cursor_center.clone(),
            RepeatedSprite {
                cols: middle_size,
                x_size: 16,
                y_size: 16,
                ..Default::default()
            },
        )
    ]
}

fn main_menu_input_update(
    mut runtime_data: ResMut<MainMenuRuntimeData>,
    gamepad: Single<&Gamepad>,
    fade_external_data: ResMut<fade_transition::FadeExternalData>,
    mut sound_manager: SoundManager,
) {
    if runtime_data.is_transitioning_out {
        return;
    }

    let fading_data = fade_external_data.into_inner();
    if fading_data.is_current_transitioning || fading_data.request.request_valid {
        return;
    }

    if gamepad.pressed(GamepadButton::DPadUp) {
        if !runtime_data.is_selected_start_button {
            sound_manager.play_sound_effect(sound_manager.sound_list.menu_cursor_change_sound);

            runtime_data.is_selected_start_button = true;
            runtime_data.should_selected_button_visible = true;
            runtime_data.timer = 0.0;

            // info!("display START buttom");
        }
    }

    if gamepad.pressed(GamepadButton::DPadDown) {
        if runtime_data.is_selected_start_button {
            sound_manager.play_sound_effect(sound_manager.sound_list.menu_cursor_change_sound);

            runtime_data.is_selected_start_button = false;
            runtime_data.should_selected_button_visible = true;
            runtime_data.timer = 0.0;

            // info!("display CREDIT buttom");
        }
    }

    if gamepad.just_pressed(GamepadButton::East) {
        runtime_data.is_transitioning_out = true;

        if runtime_data.is_selected_start_button {
            runtime_data.target_next_state = MyGameState::InGame;
        } else {
            runtime_data.target_next_state = MyGameState::Credit;
        }

        sound_manager.play_sound_effect(sound_manager.sound_list.menu_cursor_select);

        fading_data.request.request_valid = true;
        fading_data.request.is_fade_in = false;
        fading_data.request.speed = fade_transition::TransitionSpeed::Medium;
        fading_data.request.transition_type = fade_transition::FadeTransitionType::Vertical;
    }
}

fn main_menu_selection_cursor_animation_update(
    mut runtime_data: ResMut<MainMenuRuntimeData>,
    menu_text_owner: Res<MenuTextOwner>,
    mut entities: Query<&mut Children>,
    mut all_sprites: Query<&mut Sprite>,
    time: Res<Time>,
) {
    runtime_data.timer += time.delta_secs();

    if runtime_data.should_selected_button_visible {
        if runtime_data.timer >= 0.5 {
            runtime_data.should_selected_button_visible = false;
            runtime_data.timer = 0.0;
        }
    } else {
        if runtime_data.timer >= 0.2 {
            runtime_data.should_selected_button_visible = true;
            runtime_data.timer = 0.0;
        }
    }

    let mut credit_cursor_entity_owner = entities.get_mut(menu_text_owner.credit_cursor_entity);
    for children in credit_cursor_entity_owner.iter_mut() {
        for child in children.iter() {
            let sprite_result = all_sprites.get_mut(child);
            if sprite_result.is_ok() {
                let mut sprite = sprite_result.unwrap();
                sprite.visible = runtime_data.is_selected_start_button == false
                    && runtime_data.should_selected_button_visible;
            }
        }
    }

    let mut start_game_cursor_entity_owner =
        entities.get_mut(menu_text_owner.start_game_cursor_entity);
    for children in start_game_cursor_entity_owner.iter_mut() {
        for child in children.iter() {
            let sprite_result = all_sprites.get_mut(child);
            if sprite_result.is_ok() {
                let mut sprite = sprite_result.unwrap();
                sprite.visible = runtime_data.is_selected_start_button == true
                    && runtime_data.should_selected_button_visible;
            }
        }
    }
}

fn transitioning_to_next_state_update(
    mut next_state: ResMut<NextState<MyGameState>>,
    mut load_level: ResMut<WantedLevel>,
    mut runtime_data: ResMut<MainMenuRuntimeData>,
    fade_external_data: ResMut<fade_transition::FadeExternalData>,
) {
    if !runtime_data.is_transitioning_out || runtime_data.request_sent_to_next_state {
        return;
    }

    let fading_data = fade_external_data.into_inner();
    if fading_data.is_current_transitioning || fading_data.request.request_valid {
        return;
    }

    next_state.set(runtime_data.target_next_state);
    load_level.0 = Some(0);
    runtime_data.request_sent_to_next_state = true;
}
