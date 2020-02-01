use super::{imgui_system, sprite_resources::SpriteName, ComponentBounds, InspectorParameters};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Player {
    pub standing_sprite: Option<SpriteName>,
    #[serde(skip)]
    pub active: bool,
}

impl ComponentBounds for Player {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(new_sprite) = imgui_system::typed_enum_selection_option_named(
            ip.ui,
            &self.standing_sprite,
            "Standing Sprite",
            ip.uid,
        ) {
            self.standing_sprite = new_sprite;
        };

        ip.ui
            .checkbox(&imgui::im_str!("Active##{}", ip.uid), &mut self.active);
    }
}
