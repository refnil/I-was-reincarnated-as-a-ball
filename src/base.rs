use agb::sound::mixer::Frequency;
use bevy::app::Plugin;

use bevy::{
    app::PanicHandlerPlugin,
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    input::{
        InputSystem,
        gamepad::{gamepad_connection_system, gamepad_event_processing_system},
    },
    prelude::*,
    state::app::StatesPlugin,
    time::TimePlugin,
};
use bevy_mod_gba::{
    AgbDmaPlugin, AgbInputPlugin, AgbLogPlugin, AgbRunnerPlugin, AgbSoundPlugin, AgbTimePlugin,
    AgbUnpackPlugin,
};

use crate::render::RenderPlugin;
use crate::sound_loader::SoundLoaderPlugin;
use crate::sound_manager::SoundManagerPlugin;
use crate::sprite_loader::SpriteLoaderPlugin;
use crate::text::TextPlugin;

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.edit_schedule(Last, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        });

        // The first step is to add the `AgbPlugin`.
        // This sets up integration between Bevy and the `agb` abstraction over the GameBoy Advance.
        // This _must_ be done first, as it also sets up `Instant` for us.
        // Otherwise, the `TimePlugin` will fail to initialize.

        app.add_plugins((
            AgbUnpackPlugin,
            AgbLogPlugin,
            AgbInputPlugin,
            AgbRunnerPlugin,
            AgbTimePlugin,
            AgbSoundPlugin {
                enable_dmg: true,
                mixer_frequency: Some(Frequency::Hz32768),
            },
            AgbDmaPlugin,
            // No render plugin
        ));

        // Next we can add any Bevy plugins we like.
        // TODO: Used `DefaultPlugins` instead of this explicit list.
        // `DefaultPlugins` includes `InputPlugin` which is problematic on the GameBoy Advance. See below.
        app.add_plugins((
            PanicHandlerPlugin,
            TimePlugin,
            TransformPlugin,
            StatesPlugin,
        ));

        // TODO: Type registration information from `InputPlugin` causes an OOM error.
        // So we manually register the parts of this plugin that we need and ignore the rest.
        app.add_systems(
            PreUpdate,
            (
                gamepad_connection_system,
                gamepad_event_processing_system.after(gamepad_connection_system),
            )
                .in_set(InputSystem),
        );

        // Setup our "custom" rendering stack
        app.add_plugins((
            RenderPlugin,
            SpriteLoaderPlugin,
            SoundLoaderPlugin,
            SoundManagerPlugin { enable: true },
            TextPlugin,
        ));
    }
}
