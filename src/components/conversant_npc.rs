use super::{
    component_utils::SerializableEntityReference, Color, ComponentBounds, Entity, InspectorParameters,
    SerializablePrefabReference,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct ConversantNPC {
    pub conversation_partner: SerializableEntityReference,
    pub initial_ui_prefab: SerializablePrefabReference,
    pub text_ui_prefab: SerializablePrefabReference,
    pub distance: f32,
    pub color_on_close: Color,
    pub color_on_far: Color,
    pub converse_with_input: bool,
    pub converse_text: String,

    #[serde(skip)]
    pub runtime_ui: Option<Entity>,
}

impl ComponentBounds for ConversantNPC {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        // pub ui_component_entity: SerializableEntityReference
        self.conversation_partner.inspect("Conversation Partner", &ip);

        // pub initial_ui_prefab: SerializablePrefabReference,
        self.initial_ui_prefab.inspect("Bang UI", &ip);
        // pub text_ui_prefab: SerializablePrefabReference,
        self.text_ui_prefab.inspect("Text UI", &ip);

        // pub distance: f32
        ip.ui
            .drag_float(&imgui::im_str!("Distance##{}", ip.uid), &mut self.distance)
            .build();

        // pub color_on_close: Color
        self.color_on_close.inspect(&ip.ui, "Color Close", ip.uid);

        // pub color_on_far: Color
        self.color_on_far.inspect(&ip.ui, "Color Far", ip.uid);

        // pub converse_with_input: bool
        ip.ui.checkbox(
            &imgui::im_str!("Converse With Input##{}", ip.uid),
            &mut self.converse_with_input,
        );
    }
}
