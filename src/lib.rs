//! An example game written in the Bevy game engine and using the [`agb`] crate to allow running it
//! on the GameBoy Advance.

//! We declare our crate as `no_std`, as the GameBoy Advance doesn't have a port of the standard
//! library.
#![no_std]

//! [`agb`] provides a global allocator, allowing us to use items from the [`alloc`] crate.
extern crate alloc;

use agb::display::{HEIGHT, WIDTH};
use bevy::prelude::*;
use bevy_mod_gba::Sprite;
use level::LevelPlugin;
use log::info;

use crate::{
    ball_type::BallPlugin,
    base::BasePlugin,
    physic::{PhysicConfig, PhysicPlugin},
};

pub mod ball_type;
pub mod base;
pub mod fade_transition;
pub mod game_state;
pub mod level;
pub mod physic;
pub mod render;
pub mod sound_loader;
pub mod sound_manager;
pub mod sprite_loader;
pub mod text;
pub mod utils;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, mut app: &mut App) {
        app.add_plugins((BasePlugin, LevelPlugin, PhysicPlugin, BallPlugin));

        app.insert_resource(PhysicConfig {
            boundary: Rect::new(16., 32., (WIDTH - 16) as f32, (HEIGHT - 16) as f32),
        });

        game_state::init_game_state_system(&mut app);
        fade_transition::init_fade_transition_system(&mut app);

        // app.add_systems(PreUpdate, entities_count);
    }
}

fn entities_count(world: &World) {
    let count = world.entities().len();
    let archetypes = world.archetypes().len();

    for archetype in world.archetypes().iter() {
        let size = archetype.len();

        info!("Archetype of {size}");
        for name in archetype
            .components()
            .map(|component_id| world.components().get_info(component_id).unwrap().name())
        {
            info!("  {name}");
        }
    }

    info!("Entities: {count} Archetype: {archetypes}");
}
