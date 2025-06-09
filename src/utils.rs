use agb::display::{HEIGHT, WIDTH};
use bevy::math::Vec3;

pub fn get_screen_center_position() -> Vec3 {
    return get_screen_size() / 2.0;
}

pub fn get_screen_size() -> Vec3 {
    // GBA screen is : 240 × 160 px
    return Vec3::new(WIDTH as f32, HEIGHT as f32, 0.0);
}
