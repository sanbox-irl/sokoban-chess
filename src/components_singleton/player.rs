use super::{imgui_system, ComponentBounds, InspectorParameters, SpriteName};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Player {
    pub move_speed: f32,
    pub friction_moving: f32,
    pub friction_standstill: f32,
    pub standing_sprite: Option<SpriteName>,
    pub walking_sprite: Option<SpriteName>,
}

impl ComponentBounds for Player {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        ip.ui
            .input_float(&imgui::im_str!("Move Speed##{}", ip.uid), &mut self.move_speed)
            .build();

        ip.ui
            .input_float(
                &imgui::im_str!("Friction Moving##{}", ip.uid),
                &mut self.friction_moving,
            )
            .build();

        ip.ui
            .input_float(
                &imgui::im_str!("Friction Standstill##{}", ip.uid),
                &mut self.friction_standstill,
            )
            .build();

        if let Some(new_sprite) = imgui_system::typed_enum_selection_option_named(
            ip.ui,
            &self.walking_sprite,
            "Walking Sprite",
            ip.uid,
        ) {
            self.walking_sprite = new_sprite;
        };

        if let Some(new_sprite) = imgui_system::typed_enum_selection_option_named(
            ip.ui,
            &self.walking_sprite,
            "Standing Sprite",
            ip.uid,
        ) {
            self.standing_sprite = new_sprite;
        };
    }
}
