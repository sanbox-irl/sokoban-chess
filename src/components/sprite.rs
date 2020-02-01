use super::{
    component_utils::SpriteRunningData, imgui_system, sprite_resources::*, ComponentBounds,
    InspectorParameters,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, typename::TypeName, Default)]
pub struct Sprite {
    pub sprite_name: Option<SpriteName>,
    pub running_data: SpriteRunningData,
}

impl Sprite {
    // // GETTERS AND SETTERS!
    pub fn ensure_sprite(&mut self, name: SpriteName) {
        match self.sprite_name {
            Some(sp) => {
                if sp != name {
                    self.sprite_name = Some(name);
                }
            }
            None => self.sprite_name = Some(name),
        }
    }

    pub fn set_new_sprite(&mut self, name: SpriteName) {
        self.sprite_name = Some(name);
    }

    fn reset_animation(&mut self) {
        self.running_data.current_frame = 0;
        self.running_data.frame_time = 0.0;
        self.running_data.is_animating = true;
    }

    // pub fn origin(&self) -> Option<Origin> {
    //     if let Some(sprite_data) = &self.sprite_data {
    //         Some(sprite_data.origin)
    //     } else {
    //         None
    //     }
    // }
}

impl ComponentBounds for Sprite {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>) {
        let InspectorParameters { uid, ui, .. } = inspector_parameters;

        if let Some(new_sprite) =
            imgui_system::typed_enum_selection_option(ui, &self.sprite_name, uid)
        {
            self.reset_animation();
            self.sprite_name = new_sprite;
        };

        self.running_data.inspect(ui, uid);

        // CURRENT FRAME
        let mut frame = self.running_data.current_frame as i32;
        if ui
            .input_int(&imgui::im_str!("Current Frame##{}", uid), &mut frame)
            .build()
        {
            self.running_data.current_frame = frame as usize;
        }
    }
}
