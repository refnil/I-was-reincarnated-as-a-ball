//! An example game written in the Bevy game engine and using the [`agb`] crate to allow running it
//! on the GameBoy Advance.

//! We declare our crate as `no_std`, as the GameBoy Advance doesn't have a port of the standard
//! library.
#![no_std]

//! We also declare the crate as not having a typical `main` function.
//! The `agb-gbafix` tool we use to generate our final ROM file expects an exported
//! function named `main` accepting no arguments and _never_ returning.
//! This is handled by [`main`].
#![no_main]

//! [`agb`] provides a global allocator, allowing us to use items from the [`alloc`] crate.

extern crate alloc;

use agb::display::object::TextAlignment;
use bevy::prelude::*;
use lib::{
    base::BasePlugin,
    render::{AffineSprite, RepeatedSprite},
    sprite_loader::Sprites,
    text::{Size, Text, TextVisibility},
};

/// Main entry point.
#[expect(unsafe_code)]
#[unsafe(export_name = "main")]
pub extern "C" fn main() -> ! {
    // We can use Bevy's `App` abstraction just like any other Bevy application.
    let mut app = App::new();

    app.add_plugins(BasePlugin);
    app.add_systems(Startup, setup_example);

    app.run();

    // Finally, we ensure this function never returns by entering an infinite loop if our app
    // ever exits.
    #[allow(clippy::empty_loop)]
    loop {}
}

fn setup_example(mut commands: Commands, sprites: NonSend<Sprites>) {
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0., 0., 90.)),
        sprites.princess.clone(),
        RepeatedSprite {
            rows: 2,
            cols: 3,
            x_size: 32,
            y_size: 32,
        },
        AffineSprite::enabled(),
    ));
}
