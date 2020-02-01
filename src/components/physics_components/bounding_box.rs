pub use super::*;

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

    pub fn with_sprite(this_sprite: &Component<Sprite>, bind_to_sprite: bool) -> Self {
        let rect = Self::rect_from_sprite(this_sprite).unwrap_or_default();
        Self { rect, bind_to_sprite }
    }

    pub fn rect_from_sprite(this_sprite: &Component<Sprite>) -> Option<Rect> {
        this_sprite.inner().sprite_data.as_ref().map(|sprite_data| {
            let rel_location = sprite_data.origin.sprite_location_relative(sprite_data.size);

            Rect::point_width(rel_location, sprite_data.size.into())
        })
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
