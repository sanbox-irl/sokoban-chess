use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, typename::TypeName, Hash)]
#[serde(default)]
pub struct SoundSource {
    pub sound_to_play: Option<SoundResource>,
    pub muted: bool,
}

use imgui::*;
impl ComponentBounds for SoundSource {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>) {
                let InspectorParameters { uid, ui, .. } = inspector_parameters;

        if let Some(sound_to_play_maybe) =
            imgui_system::typed_enum_selection_option(ui, &self.sound_to_play, uid)
        {
            self.sound_to_play = sound_to_play_maybe;
        }

        // MUTED
        ui.checkbox(&im_str!("Muted##{}", uid), &mut self.muted);
    }
}
