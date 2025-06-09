use bevy::prelude::*;

use crate::game_state::MyGameState;

pub fn init_state_game_init_system(app: &mut App) {
    app.insert_state(MyGameState::GameInit);

    app.add_systems(
        FixedUpdate,
        (game_init_fixed_update,)
            .chain()
            .run_if(in_state(MyGameState::GameInit)),
    );
}

pub fn game_init_fixed_update(mut next_state: ResMut<NextState<MyGameState>>) {
    // info!("init_game_fixed_update");
    next_state.set(MyGameState::SplashScreen);
}
