use core::fmt::Write;

use agb::display::{
    WIDTH,
    object::{self, ChangeColour, OamIterator, ObjectTextRender, PaletteVram, TextAlignment},
    palette16::Palette16,
};
use alloc::string::String;
use bevy::{
    app::{Last, Plugin},
    ecs::{
        component::Component,
        error::Result,
        observer::Trigger,
        schedule::IntoScheduleConfigs,
        system::{NonSendMut, Query, SystemParam},
        world::{OnInsert, OnRemove},
    },
    prelude::*,
    transform::components::GlobalTransform,
};

use crate::render::render_objects_and_text;

pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_non_send_resource::<TextRenderers>();

        app.add_observer(on_text_insert);
        app.add_observer(on_handle_remove);

        app.add_systems(Last, update_text.before(render_objects_and_text));
    }
}

#[derive(Component, Default)]
struct TextHandle {
    id: Option<usize>,
}

impl TextHandle {
    fn set(&mut self, renderer: ObjectTextRender<'static>, renderers: &mut TextRenderers) {
        if let Some(id) = self.id {
            renderers.renderers[id] = Some(renderer);
        } else {
            self.id = Some(renderers.add(renderer));
        }
    }
    fn remove(&mut self, renderers: &mut TextRenderers) {
        if let Some(id) = self.id {
            renderers.renderers[id] = None;
        }
        self.id = None;
    }
}

#[derive(Component)]
pub struct TextVisibility {
    pub visible: bool,
}

impl Default for TextVisibility {
    fn default() -> Self {
        Self { visible: true }
    }
}

#[derive(Default, Copy, Clone)]
pub enum Size {
    Small,
    #[default]
    Medium,
    // Large,
}

#[derive(Default, Clone)]
pub enum TextContent {
    // Owned(String),
    Ref(&'static str),
    #[default]
    No
}

impl TextContent {
    pub fn as_ref(&self) -> Option<&str> {
        match self {
            TextContent::Ref(reference) => Some(*reference),
            // TextContent::Owned(string) => Some(string.as_ref()),
            TextContent::No => None
        }
    }
}

#[derive(Component, Default, Clone)]
#[require(TextHandle, TextVisibility)]
#[component(immutable)]
pub struct Text {
    pub text: TextContent,
    pub alignment: TextAlignment,
    pub size: Size,
}

impl From<&'static str> for Text {
    fn from(value: &'static str) -> Self {
        Text {
            text: TextContent::Ref(value),
            ..Default::default()
        }
    }
}

impl Text {
    pub fn with_content(&self, content: TextContent) -> Self {
        let mut new = self.clone();
        new.text = content;
        new
    }

    pub fn update(&self, content: TextContent) -> Option<Self> {
        if self.text.as_ref() == content.as_ref() {
            None
        } else {
            Some(self.with_content(content))
        }
    }
}

fn on_text_insert(
    trigger: Trigger<OnInsert, Text>,
    mut text_components: Query<(&Text, &mut TextHandle)>,
    mut renderers: NonSendMut<TextRenderers>,
) -> Result {
    let (text, mut text_handle) = text_components.get_mut(trigger.target())?;

    // info!("on_text_insert");
    if let Some(content) = &text.text.as_ref() {
        static SMALL: agb::display::Font =
            agb::include_font!("./assets/QuinqueFive_Font_1_15/QuinqueFive.ttf", 8);
        static MEDIUM: agb::display::Font =
            agb::include_font!("./assets/QuinqueFive_Font_1_15/QuinqueFive.ttf", 12);
        // static LARGE: agb::display::Font =
        //     agb::include_font!("./assets/QuinqueFive_Font_1_15/QuinqueFive.ttf", 20);

        let (font, size) = match text.size {
            Size::Small => (&SMALL, object::Size::S16x16),
            Size::Medium => (&MEDIUM, object::Size::S16x16),
            // Size::Large => (&LARGE, object::Size::S32x32),
        };

        let mut writer = ObjectTextRender::new(font, size, renderers.palette_vram.clone());

        writer.write_str(content)?;
        writer.layout((WIDTH, 40), text.alignment, 4);

        writer.next_letter_group();

        text_handle.set(writer, &mut renderers);
    } else {
        text_handle.remove(&mut renderers);
    }

    Ok(())
}

fn on_handle_remove(
    trigger: Trigger<OnRemove, TextHandle>,
    mut text_components: Query<&mut TextHandle>,
    mut renderers: NonSendMut<TextRenderers>,
) -> Result {
    // info!("on_handle_remove");
    let mut handle = text_components.get_mut(trigger.target())?;
    handle.remove(&mut renderers);
    Ok(())
}

const RENDERER_COUNT: usize = 5;

struct TextRenderers {
    renderers: [Option<ObjectTextRender<'static>>; RENDERER_COUNT],
    visible: [bool; RENDERER_COUNT],
    palette_vram: PaletteVram,
}

impl Default for TextRenderers {
    fn default() -> Self {
        let mut palette = [0x0; 16];
        palette[1] = 0xFF_FF; // White
        palette[2] = 0xf800; // Blue
        let palette: Palette16 = Palette16::new(palette);

        Self {
            renderers: Default::default(),
            visible: Default::default(),
            palette_vram: PaletteVram::new(&palette).unwrap(),
        }
    }
}

impl TextRenderers {
    fn add(&mut self, renderer: ObjectTextRender<'static>) -> usize {
        for i in 0..RENDERER_COUNT {
            let at_i = &mut self.renderers[i];
            if at_i.is_none() {
                at_i.replace(renderer);
                return i;
            }
        }

        unreachable!("Too many text. The limit is {RENDERER_COUNT}.");
    }
}

#[derive(SystemParam)]
pub struct TextQuery<'w> {
    renderers: NonSendMut<'w, TextRenderers>,
}

fn update_text(
    text: Query<(&TextHandle, &TextVisibility, Option<&GlobalTransform>)>,
    mut renderers: NonSendMut<TextRenderers>,
) {
    for (handle, visibility, transform) in text {
        if let Some(id) = handle.id {
            if let Some(renderer) = &mut renderers.renderers[id] {
                renderer.next_letter_group();
                renderer.update(
                    transform
                        .map(|t| t.translation())
                        .map(|t| (t.x as i32, t.y as i32))
                        .unwrap_or((0, 0)),
                );
            }
            renderers.visible[id] = visibility.visible;
        }
    }
}

pub fn render_text_object(mut oam_iterator: &mut OamIterator, mut renderers: TextQuery) {
    for i in 0..RENDERER_COUNT {
        if renderers.renderers.visible[i]
            && let Some(renderer) = &mut renderers.renderers.renderers[i]
        {
            renderer.commit(&mut oam_iterator);
        }
    }
}
