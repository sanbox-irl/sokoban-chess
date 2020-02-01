use super::{
    component_utils::SpriteRunningData, imgui_system, sprite_resources::SpriteName,
    ComponentSerializedBounds, InspectorParameters, Sprite,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
#[serde(rename = "Sprite", default)]
pub struct SpriteSerialized {
    pub sprite_name: Option<SpriteName>,
    pub serialized_running_data: SpriteRunningData,
}

impl From<Sprite> for SpriteSerialized {
    fn from(o: Sprite) -> SpriteSerialized {
        SpriteSerialized {
            sprite_name: o.sprite_data.map(|sd| sd.sprite_name),
            serialized_running_data: o.running_data.clone(),
        }
    }
}

impl From<SpriteSerialized> for Sprite {
    fn from(o: SpriteSerialized) -> Sprite {
        Sprite {
            sprite_data: None,
            new_sprite: o.sprite_name,
            running_data: o.serialized_running_data.clone(),
        }
    }
}

impl ComponentSerializedBounds for SpriteSerialized {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(new_sprite) = imgui_system::typed_enum_selection_option(ip.ui, &self.sprite_name, ip.uid)
        {
            self.sprite_name = new_sprite;
        };

        self.serialized_running_data.inspect(ip.ui, ip.uid);
    }
}
