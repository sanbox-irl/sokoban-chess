use super::{ComponentBounds, InspectorParameters, Vec2};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Velocity {
    pub velocity: Vec2,
}

impl ComponentBounds for Velocity {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        ip.ui.label_text(
            &imgui::im_str!("Velocity##{}", ip.uid),
            &imgui::im_str!("{}", self.velocity),
        );
    }
}
