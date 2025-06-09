use bevy::prelude::*;


mod game_state_credit;
mod game_state_game_init;
pub mod game_state_in_game;

mod game_state_main_menu;
mod game_state_splash_screen;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Copy)]
pub enum MyGameState {
    #[default]
    GameInit,
    SplashScreen,
    MainMenu,
    InGame,
    Credit,
}

pub fn init_game_state_system(app: &mut App) {
    // info!("init_game_state_system");

    game_state_game_init::init_state_game_init_system(app);
    game_state_splash_screen::init_state_splash_screen_system(app);
    game_state_main_menu::init_state_main_menu_system(app);
    game_state_in_game::init_state_in_game_system(app);
    game_state_credit::game_state_credit(app);
}
