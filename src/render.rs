use agb::{
    display::{
        affine::{AffineMatrix, AffineMatrixObject},
        object::{AffineMatrixInstance, AffineMode},
    },
    fixnum::Num,
};
use alloc::{borrow::ToOwned, boxed::Box};
use bevy::{
    app::{App, Last, Plugin},
    ecs::{
        component::Component,
        error::Result,
        system::{IntoSystem, NonSendMut, Query, RunSystemOnce, System},
    },
    math::Vec3,
    transform::components::{GlobalTransform, Transform},
};
pub use bevy_mod_gba::{Sprite, SpriteHandles};
use log::warn;

use crate::text::{TextQuery, render_text_object};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, render_objects_and_text);
    }

    fn finish(&self, app: &mut App) {
        let Some(display) = app
            .world_mut()
            .remove_non_send_resource::<agb::display::Display>()
        else {
            return;
        };

        let agb::display::Display { object, .. } = display;

        let object = Box::leak(Box::new(object));

        let (oam, sprite_loader) = object.get_unmanaged();

        app.insert_non_send_resource(oam);
        app.insert_non_send_resource(sprite_loader);
        app.insert_non_send_resource(SpriteHandles::new());
    }
}

/// Controls access to the underlying video hardware.
// #[derive(Deref, DerefMut)]
// pub struct VRam(VRamManager);

/// Provides access to [`Windows`](agb::display::window::Windows).
// #[derive(Resource, Deref, DerefMut)]
// pub struct WindowDist(agb::display::WindowDist);

// /// Provides access to [`Blend`](agb::display::blend::Blend).
// #[derive(Resource, Deref, DerefMut)]
// pub struct BlendDist(agb::display::BlendDist);

#[derive(Component)]
#[require(Transform)]
pub struct AffineSprite {
    pub enable: bool,
}

impl AffineSprite {
    pub fn enabled() -> Self {
        Self { enable: true }
    }
}

impl Default for AffineSprite {
    fn default() -> Self {
        Self { enable: false }
    }
}

#[derive(Component, Clone, Copy)]
#[require(AffineSprite)]
pub struct RepeatedSprite {
    pub rows: i32,
    pub cols: i32,
    pub x_size: i32,
    pub y_size: i32,
}

impl Default for RepeatedSprite {
    fn default() -> Self {
        Self {
            rows: 1,
            cols: 1,
            x_size: 64,
            y_size: 64,
        }
    }
}

type AffineMatrixElement = Num<i32, 8>;

pub fn render_objects_and_text(
    mut oam: NonSendMut<agb::display::object::OamUnmanaged<'static>>,
    sprites: Query<(
        &Sprite,
        &GlobalTransform,
        Option<&AffineSprite>,
        Option<&RepeatedSprite>,
    )>,
    sprite_assets: NonSendMut<SpriteHandles>,
    text_params: TextQuery,
) -> Result {
    let oam_iterator = &mut oam.iter();

    render_text_object(oam_iterator, text_params);

    for (sprite, transform, affine, repeated) in &sprites {
        let Some(handle) = sprite_assets.get(&sprite.handle) else {
            continue;
        };

        let mut obj = agb::display::object::ObjectUnmanaged::new(handle);

        if !sprite.visible {
            continue;
        }

        let repeated = repeated.cloned().unwrap_or_default();

        let Vec3 { x, y, .. } = transform.translation();

        for row in 0..repeated.rows {
            let y = y + (row * repeated.y_size) as f32;
            let y = y.clamp(i32::MIN as f32, i32::MAX as f32) as i32;

            for col in 0..repeated.cols {
                let x = x + (col * repeated.x_size) as f32;
                let x = x.clamp(i32::MIN as f32, i32::MAX as f32) as i32;

                if !(-64..240).contains(&x) && !(-64..160).contains(&y) {
                    continue;
                }

                let position = agb::fixnum::Vector2D { x, y };

                if let Some(affine) = affine {
                    if affine.enable {
                        let (_scale, rotation, _translation) =
                            transform.to_scale_rotation_translation();

                        // let adjusted_scale: Vector2D<Num<i32, 8>> =
                        //     Vector2D::new(Num::from_f32(scale.x), Num::from_f32(scale.y));
                        let affine_matrix =
                            AffineMatrix::from_rotation(AffineMatrixElement::from_f32(
                                rotation.to_euler(bevy::math::EulerRot::XYZ).2,
                            ));
                        //    * AffineMatrix::from_scale(adjusted_scale);

                        // info!("scale {scale:?} rotation {rotation:?} {adjusted_scale:?} {affine_matrix:?}");

                        let amo = AffineMatrixObject::try_from(affine_matrix)
                            .map_err(|_| "Cannot convert matrix")?;

                        obj.set_affine_matrix(AffineMatrixInstance::new(amo));
                        obj.show_affine(AffineMode::Affine);
                    } else {
                        obj.show();
                    }
                } else {
                    obj.show();
                }

                obj.set_position(position)
                    .set_hflip(sprite.horizontal_flipped)
                    .set_vflip(sprite.vertical_flipped)
                    .set_priority(sprite.priority)
                    .set_graphics_mode(sprite.graphics_mode);

                let Some(next) = oam_iterator.next() else {
                    warn!("Ran out of OAM slots!");
                    return Ok(());
                };

                next.set(&obj);
            }
        }
    }

    Ok(())
}
