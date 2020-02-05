use super::{ComponentBounds, InspectorParameters};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, typename::TypeName)]
pub struct SceneSwitcher {
    pub target_scene: String,
}

impl ComponentBounds for SceneSwitcher {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        let mut scene_name = imgui::ImString::new(&self.target_scene);

        if ip
            .ui
            .input_text(&imgui::im_str!("Scene##{}", ip.uid), &mut scene_name)
            .resize_buffer(true)
            .build()
        {
            self.target_scene = scene_name.to_string();
        }
    }
}
