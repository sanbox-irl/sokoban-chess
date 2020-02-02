use super::{cardinals::CardinalPrime, ComponentBounds, InspectorParameters};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Velocity {
    pub intended_direction: Option<CardinalPrime>,
}

impl ComponentBounds for Velocity {
    fn entity_inspector(&mut self, ip: InspectorParameters<'_, '_>) {
        if let Some(new_direction) = super::imgui_system::typed_enum_selection_option_named(
            ip.ui,
            &self.intended_direction,
            "Intended Direction",
            ip.uid,
        ) {
            self.intended_direction = new_direction;
        }
    }
}
