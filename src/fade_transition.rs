use core::ops::{Index, IndexMut};

use bevy::prelude::*;


use super::sprite_loader;
use crate::{
    Sprite,
    render::RepeatedSprite,
    sprite_loader::Sprites,
    utils::{get_screen_center_position, get_screen_size},
};

#[derive(PartialEq, Clone, Copy)]
enum VisualPivotType {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum FadeTransitionType {
    #[default]
    Horizontal,

    Vertical,
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TransitionSpeed {
    #[default]
    Slow,
    Medium,
    Fast,
}

#[derive(Resource, Default, Clone)]
pub struct FadeRuntimeData {
    is_transitioning: bool,
    is_fading_in: bool,

    transition_type: FadeTransitionType,

    transition_speed: TransitionSpeed,

    current_fade_entities_offset: Vec2,

    top_left: Option<Entity>,
    top_right: Option<Entity>,
    bottom_right: Option<Entity>,
    bottom_left: Option<Entity>,
}

impl Index<VisualPivotType> for FadeRuntimeData {
    type Output = Option<Entity>;

    fn index(&self, index: VisualPivotType) -> &Self::Output {
        match index {
            VisualPivotType::BottomLeft => &self.bottom_left,
            VisualPivotType::TopLeft => &self.top_left,
            VisualPivotType::TopRight => &self.top_right,
            VisualPivotType::BottomRight => &self.bottom_right,
        }
    }
}

impl IndexMut<VisualPivotType> for FadeRuntimeData {
    fn index_mut(&mut self, index: VisualPivotType) -> &mut Self::Output {
        match index {
            VisualPivotType::BottomLeft => &mut self.bottom_left,
            VisualPivotType::TopLeft => &mut self.top_left,
            VisualPivotType::TopRight => &mut self.top_right,
            VisualPivotType::BottomRight => &mut self.bottom_right,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct FadeRequestData {
    pub request_valid: bool,
    pub is_fade_in: bool,
    pub speed: TransitionSpeed,

    pub transition_type: FadeTransitionType,
}

#[derive(Resource, Default, Clone)]
pub struct FadeExternalData {
    pub is_current_transitioning: bool,
    pub is_fading_in: bool,

    pub request: FadeRequestData,
}

pub fn init_fade_transition_system(app: &mut App) {
    app.init_resource::<FadeRuntimeData>();
    app.init_resource::<FadeExternalData>();
    app.add_systems(Startup, spawn_fade_entites)
        .add_systems(PostUpdate, (fade_transition_update,).chain());
}

fn spawn_fade_entites(
    mut commands: Commands,
    sprites: NonSend<Sprites>,
    mut runtime_data: ResMut<FadeRuntimeData>,
) {
    // info!("spawn_fade_entites");

    let mut create = |visual_pivot_type| {
        create_fade_entity(
            visual_pivot_type,
            &mut commands,
            &sprites,
            &mut runtime_data,
        )
    };

    create(VisualPivotType::TopLeft);
    create(VisualPivotType::TopRight);
    create(VisualPivotType::BottomRight);
    create(VisualPivotType::BottomLeft);
}

fn create_fade_entity(
    target_visual_pivot_type: VisualPivotType,
    commands: &mut Commands,
    game_sprites: &Sprites,
    runetime_data: &mut FadeRuntimeData,
) {
    let screen_center = get_screen_center_position();

    let mut visual_offset = Vec3::ZERO;

    if target_visual_pivot_type == VisualPivotType::TopRight {
        visual_offset = Vec3::new(-120.0, 0.0, 0.0);
    } else if target_visual_pivot_type == VisualPivotType::BottomRight {
        visual_offset = Vec3::new(-120.0, -80.0, 0.0);
    } else if target_visual_pivot_type == VisualPivotType::BottomLeft {
        visual_offset = Vec3::new(0.0, -80., 0.0);
    }

    runetime_data[target_visual_pivot_type] = Some(
        commands
            .spawn((
                Transform::from_xyz(screen_center.x, screen_center.y, 0.),
                children![create_new_fade_visual_with_param(
                    visual_offset,
                    game_sprites,
                )],
            ))
            .id(),
    );
}

fn create_new_fade_visual_with_param(
    position: Vec3,
    game_sprites: &sprite_loader::Sprites,
) -> impl Bundle {
    (
        Transform::from_translation(position),
        game_sprites.fade_in_out.clone(),
        RepeatedSprite {
            rows: 2,
            cols: 2,
            x_size: 56,
            y_size: 16,
        },
    )
}

fn set_full_fade_in(
    // q_parent: &mut Query<&Children>,
    // q_child: &mut Query<&mut Sprite, With<FadeEntity>>,
    external_data: &mut FadeExternalData,
    runtime_data: &mut FadeRuntimeData,
) {
    // info!("set_full_fade_in");

    // set_fade_visible(q_parent, q_child, false);

    runtime_data.is_fading_in = false;
    runtime_data.is_transitioning = false;
    runtime_data.current_fade_entities_offset = Vec2::ZERO;

    external_data.is_current_transitioning = false;
    external_data.is_fading_in = runtime_data.is_fading_in;
}

fn set_full_fade_out(
    // q_parent: &mut Query<&Children>,
    // q_child: &mut Query<&mut Sprite>,
    external_data: &mut FadeExternalData,
    runtime_data: &mut FadeRuntimeData,
) {
    // info!("set_full_fade_out");

    let screen_center = get_screen_center_position();

    // set_fade_visible(q_parent, q_child, true);

    runtime_data.is_fading_in = true;
    runtime_data.is_transitioning = false;
    runtime_data.current_fade_entities_offset = Vec2::new(screen_center.x, screen_center.y);

    external_data.is_current_transitioning = false;
    external_data.is_fading_in = runtime_data.is_fading_in;
}

fn start_fade_request(
    // mut q_parent: &mut Query<&Children>,
    // q_child: &mut Query<&mut Sprite>,
    external_data: &mut FadeExternalData,
    runtime_data: &mut FadeRuntimeData,
) {
    if !external_data.request.request_valid {
        return;
    }

    // info!("start_fade_request");

    runtime_data.is_fading_in = external_data.request.is_fade_in;
    external_data.is_fading_in = runtime_data.is_fading_in;

    runtime_data.transition_type = external_data.request.transition_type;
    runtime_data.transition_speed = external_data.request.speed;

    let screen_center = get_screen_center_position();
    if runtime_data.transition_type == FadeTransitionType::Vertical {
        runtime_data.current_fade_entities_offset.y = screen_center.y;
    } else {
        runtime_data.current_fade_entities_offset.x = screen_center.x;
    }

    if runtime_data.is_fading_in {
        set_full_fade_out( external_data, runtime_data);
    } else {
        set_full_fade_in(external_data, runtime_data);

        if runtime_data.transition_type == FadeTransitionType::Vertical {
            runtime_data.current_fade_entities_offset = Vec2::new(0.0, screen_center.y);
        } else {
            runtime_data.current_fade_entities_offset = Vec2::new(screen_center.x, 0.0);
        }
    }

    // set_fade_visible(q_parent, &mut q_child, true);

    runtime_data.is_transitioning = true;
    external_data.request.request_valid = false;
}

fn fade_transition_update(
    time: Res<Time>,
    mut transforms: Query<&mut Transform>,
    data: ResMut<FadeRuntimeData>,
    external: ResMut<FadeExternalData>,
) {
    let mut fade_runtime_data = data.into_inner();
    let mut fade_external_data = external.into_inner();

    if fade_external_data.request.request_valid {
        start_fade_request(
            &mut fade_external_data,
            &mut fade_runtime_data,
        );
    }

    if !fade_runtime_data.is_transitioning {
        fade_external_data.is_current_transitioning = fade_runtime_data.is_transitioning;
        return;
    }

    let screen_center = get_screen_center_position();

    let mut target_transition_speed = 1.0 * time.delta_secs();

    // Settings: Fade Speed.
    match fade_runtime_data.transition_speed {
        TransitionSpeed::Slow => target_transition_speed = 50.0 * time.delta_secs(),
        TransitionSpeed::Medium => target_transition_speed = 60.0 * time.delta_secs(),
        TransitionSpeed::Fast => target_transition_speed = 70.0 * time.delta_secs(),
    }

    if fade_runtime_data.transition_type == FadeTransitionType::Vertical {
        if fade_runtime_data.is_fading_in {
            fade_runtime_data.current_fade_entities_offset.x -= target_transition_speed;
            // info!(
            //     "Fading In | Vertical | Transition Offset: ({}, {})",
            //     fade_runtime_data.current_fade_entities_offset.x,
            //     fade_runtime_data.current_fade_entities_offset.y
            // );

            if fade_runtime_data.current_fade_entities_offset.x <= 0.0 {
                fade_runtime_data.current_fade_entities_offset.x = 0.0;
                fade_runtime_data.is_transitioning = false;

                // info!("Transition Over (0)");
            }
        } else {
            fade_runtime_data.current_fade_entities_offset.x += target_transition_speed;
            // info!(
            //     "Fading Out | Vertical | Transition Offset: ({}, {})",
            //     fade_runtime_data.current_fade_entities_offset.x,
            //     fade_runtime_data.current_fade_entities_offset.y
            // );

            if fade_runtime_data.current_fade_entities_offset.x >= screen_center.x {
                fade_runtime_data.current_fade_entities_offset.x = screen_center.x;
                fade_runtime_data.is_transitioning = false;

                // info!("Transition Over (1)");
            }
        }
    } else if fade_runtime_data.transition_type == FadeTransitionType::Horizontal {
        if fade_runtime_data.is_fading_in {
            fade_runtime_data.current_fade_entities_offset.y -= target_transition_speed;
            // info!(
            //     "Fading In | Horizontal | Transition Offset: ({}, {})",
            //     fade_runtime_data.current_fade_entities_offset.x,
            //     fade_runtime_data.current_fade_entities_offset.y
            // );

            if fade_runtime_data.current_fade_entities_offset.y <= 0.0 {
                fade_runtime_data.current_fade_entities_offset.y = 0.0;
                fade_runtime_data.is_transitioning = false;

                // info!("Transition Over (2)");
            }
        } else {
            fade_runtime_data.current_fade_entities_offset.y += target_transition_speed;
            // info!(
            //     "Fading Out | Horizontal | Transition Offset: ({}, {})",
            //     fade_runtime_data.current_fade_entities_offset.x,
            //     fade_runtime_data.current_fade_entities_offset.y
            // );

            if fade_runtime_data.current_fade_entities_offset.y >= screen_center.y {
                fade_runtime_data.current_fade_entities_offset.y = screen_center.y;
                fade_runtime_data.is_transitioning = false;

                // info!("Transition Over (3)");
            }
        }
    }

    update_translation_fade_entities(
        &fade_runtime_data,
        &mut transforms,
        VisualPivotType::BottomLeft,
    );
    update_translation_fade_entities(
        &fade_runtime_data,
        &mut transforms,
        VisualPivotType::BottomRight,
    );
    update_translation_fade_entities(
        &fade_runtime_data,
        &mut transforms,
        VisualPivotType::TopLeft,
    );
    update_translation_fade_entities(
        &fade_runtime_data,
        &mut transforms,
        VisualPivotType::TopRight,
    );

    fade_external_data.is_current_transitioning = fade_runtime_data.is_transitioning;
}

fn update_translation_fade_entities(
    fade_runtime_data: &FadeRuntimeData,
    transforms: &mut Query<&mut Transform>,
    vpt: VisualPivotType,
) {
    let mut transform = transforms.get_mut(fade_runtime_data[vpt].unwrap()).unwrap();

    let screen_size = get_screen_size();

    if vpt == VisualPivotType::TopLeft {
        transform.translation = screen_size
            - Vec3::new(
                fade_runtime_data.current_fade_entities_offset.x,
                fade_runtime_data.current_fade_entities_offset.y,
                0.0,
            );
    } else if vpt == VisualPivotType::TopRight {
        transform.translation.x = fade_runtime_data.current_fade_entities_offset.x;
        transform.translation.y = screen_size.y - fade_runtime_data.current_fade_entities_offset.y;
    } else if vpt == VisualPivotType::BottomRight {
        transform.translation = Vec3::new(
            fade_runtime_data.current_fade_entities_offset.x,
            fade_runtime_data.current_fade_entities_offset.y,
            0.0,
        );
    } else if vpt == VisualPivotType::BottomLeft {
        transform.translation.x = screen_size.x - fade_runtime_data.current_fade_entities_offset.x;
        transform.translation.y = fade_runtime_data.current_fade_entities_offset.y;
    }
}
