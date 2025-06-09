use bevy::prelude::*;
use log::info;


use crate::fade_transition;
use crate::game_state::MyGameState;
use crate::render::RepeatedSprite;
use crate::sound_manager::SoundManager;
use crate::sprite_loader::Sprites;

#[derive(Resource, Default, Clone, Copy)]
pub struct SplashScreenRuntimeData {
    entity: Option<Entity>,
    is_fading_out_of_splash_screen: bool,
    is_changing_to_main_menu: bool,
}

pub fn init_state_splash_screen_system(app: &mut App) {
    app.init_resource::<SplashScreenRuntimeData>();
    app.add_systems(OnEnter(MyGameState::SplashScreen), splash_screen_enter)
        .add_systems(OnExit(MyGameState::SplashScreen), splash_screen_exit)
        .add_systems(
            FixedUpdate,
            splash_screen_fixed_update.run_if(in_state(MyGameState::SplashScreen)),
        );
}

pub fn splash_screen_enter(
    mut commands: Commands,
    sprites: NonSend<Sprites>,
    mut runtime_data: ResMut<SplashScreenRuntimeData>,
    fade_external_data: ResMut<fade_transition::FadeExternalData>,
    mut sound_manager: SoundManager,
) {
    // info!("splash_screen_enter");

    runtime_data.entity = Some(
        commands
            .spawn((
                Transform::IDENTITY,
                children![(
                    Transform::from_xyz(88., 48., 0.),
                    sprites.bevy_logo.clone(),
                    RepeatedSprite::default()
                )],
            ))
            .id(),
    );

    sound_manager.change_main_sound(sound_manager.sound_list.main_menu_sound, 1);

    let fading_data = fade_external_data.into_inner();
    fading_data.request.request_valid = true;
    fading_data.request.is_fade_in = true;
    fading_data.request.speed = fade_transition::TransitionSpeed::Medium;
    fading_data.request.transition_type = fade_transition::FadeTransitionType::Horizontal;
}

pub fn splash_screen_exit(mut commands: Commands, runtime_data: Res<SplashScreenRuntimeData>) {
    // info!("splash_screen_exit");

    runtime_data
        .entity
        .map(|entity| commands.entity(entity).despawn());
}

pub fn splash_screen_fixed_update(
    time: Res<Time>,
    mut runtime_data: ResMut<SplashScreenRuntimeData>,
    fade_external_data: ResMut<fade_transition::FadeExternalData>,
    mut next_state: ResMut<NextState<MyGameState>>,
) {
    if runtime_data.is_changing_to_main_menu {
        return;
    }

    let fading_data = fade_external_data.into_inner();

    if runtime_data.is_fading_out_of_splash_screen {
        if fading_data.is_current_transitioning || fading_data.request.request_valid {
            return;
        }

        next_state.set(MyGameState::MainMenu);
        runtime_data.is_changing_to_main_menu = true;
    } else {
        if time.delta_secs() >= 0.015625 {
            fading_data.request.request_valid = true;
            fading_data.request.is_fade_in = false;
            fading_data.request.speed = fade_transition::TransitionSpeed::Fast;
            fading_data.request.transition_type = fade_transition::FadeTransitionType::Horizontal;

            runtime_data.is_fading_out_of_splash_screen = true;
            info!("Time: {}", time.delta_secs());
        }
    }
}
