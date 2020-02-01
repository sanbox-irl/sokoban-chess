use super::{ComponentBounds, InspectorParameters, Rect, Vec2};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct BoundingBox {
    pub rect: Rect,
    pub bind_to_sprite: bool,
}

impl BoundingBox {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            rect: Rect::new(min, max),
            bind_to_sprite: false,
        }
    }
}

impl ComponentBounds for BoundingBox {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>) {
        let InspectorParameters { uid, ui, .. } = inspector_parameters;

        self.rect.rect_inspector(ui, uid);
        ui.checkbox(
            &imgui::im_str!("Bind to Sprite##{}", uid),
            &mut self.bind_to_sprite,
        );
    }
}
