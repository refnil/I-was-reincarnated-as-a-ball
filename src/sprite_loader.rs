use agb::display::{Priority, object::SpriteLoader};
use bevy::prelude::*;

use bevy_mod_gba::{Sprite, SpriteHandles};

pub struct SpriteLoaderPlugin;

impl Plugin for SpriteLoaderPlugin {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        // Unfortunately, we currently don't have a first-party abstraction for assets or rendering.
        // This means getting assets, and rendering them must be done somewhat manually.
        app.init_non_send_resource::<Sprites>();
    }
}

pub struct Sprites {
    pub fade_in_out: Sprite,
    pub bevy_logo: Sprite,
    pub selection_cursor_corner: Sprite,
    pub selection_cursor_center: Sprite,
    pub character_controller: Sprite,

    pub wall_top_left: Sprite,
    pub wall_top: Sprite,
    pub wall_top_right: Sprite,
    pub wall_left: Sprite,
    pub wall_right: Sprite,
    pub wall_bottom_left: Sprite,
    pub wall_bottom: Sprite,
    pub wall_bottom_right: Sprite,

    pub floor: Sprite,
    pub menu_background: Sprite,
    pub credit_background: Sprite,

    pub boy: Sprite,
    pub princess: Sprite,
    pub dog: Sprite,

    pub ghost: Sprite,
    pub green_blob: Sprite,
    pub red_blob: Sprite,
    pub snake: Sprite,
    pub tree: Sprite,
}

impl FromWorld for Sprites {
    fn from_world(world: &mut World) -> Self {
        static BLACKISH_GRAPHICS: &agb::display::object::Graphics = agb::include_aseprite!(
            "./assets/fade_in_out.aseprite",
            "./assets/bevy_logo.aseprite",
            "./assets/selection_cursor.aseprite",
            "./assets/character_controller.aseprite"
        );

        static WALLS: &agb::display::object::Graphics =
            agb::include_aseprite!("./assets/walls.aseprite");

        static FLOOR: &agb::display::object::Graphics =
            agb::include_aseprite!("./assets/floor.aseprite");

        static FLOOR2: &agb::display::object::Graphics =
            agb::include_aseprite!("./assets/floor2.aseprite");

        static CHARACTERS: &agb::display::object::Graphics = agb::include_aseprite!(
            "./assets/boy_ball.aseprite",
            "./assets/princess_ball.aseprite",
            "./assets/dog_ball.aseprite"
        );

        static ENEMIES: &agb::display::object::Graphics = agb::include_aseprite!(
            "./assets/enemy_ghost.aseprite",
            "./assets/enemy_green_blob.aseprite",
            "./assets/enemy_red_blob.aseprite",
            "./assets/enemy_snake.aseprite",
            "./assets/tree.aseprite"
        );

        let mut get_sprite = |graphics: &'static agb::display::object::Graphics,
                              tag: &str,
                              idx: usize,
                              priority: Priority| {
            // info!("Loading {tag}");
            let mut loader = world.get_non_send_resource_mut::<SpriteLoader>().unwrap();
            let vram_sprite = loader.get_vram_sprite(graphics.tags().get(tag).sprite(idx));

            let mut handles: Mut<'_, SpriteHandles> =
                world.get_non_send_resource_mut::<SpriteHandles>().unwrap();
            let mut sprite = Sprite::new(handles.add(vram_sprite));
            sprite.priority = priority;
            sprite
        };

        Sprites {
            fade_in_out: get_sprite(BLACKISH_GRAPHICS, "Fade", 0, Priority::P0),
            bevy_logo: get_sprite(BLACKISH_GRAPHICS, "Default", 0, Priority::P2),
            selection_cursor_corner: get_sprite(
                BLACKISH_GRAPHICS,
                "Selection_Corner",
                0,
                Priority::P1,
            ),
            selection_cursor_center: get_sprite(
                BLACKISH_GRAPHICS,
                "Selection_Center",
                0,
                Priority::P1,
            ),
            character_controller: get_sprite(
                BLACKISH_GRAPHICS,
                "PlayerController",
                0,
                Priority::P1,
            ),

            wall_top_left: get_sprite(WALLS, "Wall-Top-Left", 0, Priority::P3),
            wall_top: get_sprite(WALLS, "Wall-Top", 0, Priority::P3),
            wall_top_right: get_sprite(WALLS, "Wall-Top-Right", 0, Priority::P3),
            wall_left: get_sprite(WALLS, "Wall-Left", 0, Priority::P3),
            wall_right: get_sprite(WALLS, "Wall-Right", 0, Priority::P3),
            wall_bottom_left: get_sprite(WALLS, "Wall-Bottom-Left", 0, Priority::P3),
            wall_bottom: get_sprite(WALLS, "Wall-Bottom", 0, Priority::P3),
            wall_bottom_right: get_sprite(WALLS, "Wall-Bottom-Right", 0, Priority::P3),

            floor: get_sprite(FLOOR, "Floor", 0, Priority::P3),
            menu_background: get_sprite(FLOOR, "Menu", 0, Priority::P3),
            credit_background: get_sprite(FLOOR2, "Credit", 0, Priority::P3),

            boy: get_sprite(CHARACTERS, "boy", 0, Priority::P0),
            princess: get_sprite(CHARACTERS, "princess", 0, Priority::P0),
            dog: get_sprite(CHARACTERS, "dog", 0, Priority::P0),
            ghost: get_sprite(ENEMIES, "ghost", 0, Priority::P0),
            green_blob: get_sprite(ENEMIES, "green_blob", 0, Priority::P0),
            red_blob: get_sprite(ENEMIES, "red_blob", 0, Priority::P0),
            snake: get_sprite(ENEMIES, "snake", 0, Priority::P0),
            tree: get_sprite(ENEMIES, "tree", 0, Priority::P1),
        }
    }
}
