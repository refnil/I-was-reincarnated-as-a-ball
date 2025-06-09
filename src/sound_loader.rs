use bevy::prelude::*;

use agb::include_wav;

pub struct SoundLoaderPlugin;

impl Plugin for SoundLoaderPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        app.init_non_send_resource::<SoundList>();
    }
}

pub struct SoundList {
    pub main_menu_sound: &'static [u8],
    pub credit_sound: &'static [u8],
    // pub in_game_sound: &'static [u8],
    pub menu_cursor_change_sound: &'static [u8],
    pub menu_cursor_select: &'static [u8],
}

impl FromWorld for SoundList {
    fn from_world(_world: &mut World) -> Self {
        static main_menu_sound: &[u8] = include_wav!("./assets/sfx/main_menu.wav");
        static credit_sound: &[u8] = include_wav!("./assets/sfx/credit.wav");
        // static in_game_sound: &[u8] = include_wav!("./assets/sfx/in_game.wav");
        static menu_cursor_change_sound: &[u8] =
            include_wav!("./assets/sfx/menu_cursor_change.wav");
        static menu_cursor_select: &[u8] = include_wav!("./assets/sfx/menu_cursor_select.wav");

        SoundList {
            main_menu_sound,
            credit_sound,
            // in_game_sound,
            menu_cursor_change_sound,
            menu_cursor_select,
        }
    }
}
