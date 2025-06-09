use agb::fixnum::Num;
use bevy::prelude::*;

use bevy::ecs::system::SystemParam;

use crate::sound_loader::SoundList;
use agb::sound::mixer::SoundChannel;
use agb::sound::mixer::*;

pub struct SoundManagerPlugin {
    pub enable: bool,
}

impl Plugin for SoundManagerPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        app.insert_non_send_resource(SoundManagerRuntimeData {
            enable: self.enable,
            ..Default::default()
        });
    }
}

#[derive(Default)]
pub struct SoundManagerRuntimeData {
    pub enable: bool,
    pub main_sound_channel_id: Option<ChannelId>,
    pub current_main_theme: Option<&'static [u8]>,
}

#[derive(SystemParam)]
pub struct SoundManager<'w> {
    pub sound_list: NonSend<'w, SoundList>,
    mixer: NonSendMut<'w, Mixer<'static>>,
    manager_runtime_data: NonSendMut<'w, SoundManagerRuntimeData>,
}

impl<'w> SoundManager<'w> {
    pub fn change_main_sound(&mut self, target_sound: &'static [u8], playback: u32) {
        if !self.manager_runtime_data.enable {
            return;
        }

        // info!("change_main_sound");

        if let Some(current_player_main_theme) = self.manager_runtime_data.current_main_theme {
            if (current_player_main_theme as *const _) == (target_sound as *const _) {
                // info!("Early Return, already playing");
                return;
            }
        }

        if let Some(current_channel_id) = &self.manager_runtime_data.main_sound_channel_id {
            if let Some(main_sound_channel) = self.mixer.channel(current_channel_id) {
                main_sound_channel.stop();
                // info!("Sound is stopped");
            }
        }

        let mut channel = SoundChannel::new_high_priority(target_sound);
        channel.volume(Num::from_f32(0.5));
        channel.stereo();
        channel.should_loop();
        channel.playback(playback);

        self.manager_runtime_data.main_sound_channel_id = self.mixer.play_sound(channel);
        self.manager_runtime_data.current_main_theme = Some(target_sound);
        // info!("Starting new sound!");
    }

    pub fn play_sound_effect(&mut self, target_sound: &'static [u8]) {
        if !self.manager_runtime_data.enable {
            return;
        }

        let mut channel = SoundChannel::new(target_sound);
        channel.volume(Num::from_f32(0.5));
        channel.stereo();

        self.mixer.play_sound(channel);
        // info!("Play Sound Effect!");
    }
}
