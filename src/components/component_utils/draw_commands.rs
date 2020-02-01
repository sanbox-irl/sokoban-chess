use super::{
    tilemap::Tilemap, Camera, ComponentList, DrawRectangle, Entity, RenderingUtility,
    ResourcesDatabase, Sprite, TextSource, Transform, Vec2,
};

#[derive(Default)]
pub struct DrawCommand<'a> {
    pub game_world: Option<GameWorldDrawCommands<'a>>,
    pub imgui: Option<ImGuiDrawCommands<'a>>,
}

pub struct GameWorldDrawCommands<'a> {
    pub text_sources: &'a ComponentList<TextSource>,
    pub sprites: &'a ComponentList<Sprite>,
    pub rects: &'a ComponentList<DrawRectangle>,
    pub tilemaps: &'a ComponentList<Tilemap>,
    pub transforms: &'a ComponentList<Transform>,
    pub rendering_utility: &'a mut RenderingUtility,
    pub camera_entity: &'a Entity,
    pub camera: &'a Camera,
    pub resources: &'a ResourcesDatabase,
}

pub struct ImGuiDrawCommands<'a> {
    pub draw_data: &'a imgui::DrawData,
    pub imgui_dimensions: Vec2,
}

impl<'a> DrawCommand<'a> {
    // pub fn take_game_world(
    //     &mut self,
    //     text_sources: &'a ComponentList<TextSource>,
    //     sprites: &'a ComponentList<Sprite>,
    //     rects: &'a ComponentList<DrawRectangle>,
    //     tilemaps: &'a ComponentList<Tilemap>,
    //     transforms: &'a ComponentList<Transform>,
    //     camera_entity: &'a Entity,
    //     camera: &'a Camera,
    //     rendering_utility: &'a mut RenderingUtility,
    //     window_size: Vec2,
    // ) {
    //     self.game_world = Some(GameWorldDrawCommands {
    //         sprites,
    //         text_sources,
    //         rects,
    //         tilemaps,
    //         transforms,
    //         camera,
    //         camera_entity,
    //         rendering_utility,
    //         window_size,
    //     });
    // }

    // pub fn take_imgui_commands(&mut self, draw_data: &'a imgui::DrawData, imgui_dimensions: Vec2) {
    //     self.imgui = Some(ImGuiDrawCommands {
    //         draw_data,
    //         imgui_dimensions,
    //     });
    // }
}
