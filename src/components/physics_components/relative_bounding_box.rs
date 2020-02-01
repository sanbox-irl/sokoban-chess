use super::{math, ComponentBounds, Rect, Vec2, InspectorParameters};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct RelativeBoundingBox {
    pub rect: Rect,
}

impl RelativeBoundingBox {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            rect: Rect::new(min, max),
        }
    }

    pub fn into_rect(self, native_size: Vec2) -> Rect {
        Rect {
            min: self.rect.min.cwise_product(native_size),
            max: self.rect.max.cwise_product(native_size),
        }
    }
}

impl ComponentBounds for RelativeBoundingBox {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>) {
        let InspectorParameters { uid, ui, .. } = inspector_parameters;

        if self.rect.rect_inspector(ui, uid) {
            // Clamp the Min
            math::clamped(&mut self.rect.min.x, 0.0, 1.0);
            math::clamped(&mut self.rect.min.y, 0.0, 1.0);

            // Clamp the Max
            math::clamped(&mut self.rect.max.x, 0.0, 1.0);
            math::clamped(&mut self.rect.max.y, 0.0, 1.0);
        }
    }
}
