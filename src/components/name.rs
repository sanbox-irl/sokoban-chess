use super::{
    component_utils::{
        EntityListInformation, NameEdit, NameInspectorParameters, NameInspectorResult,
    },
    imgui_system, Color, ComponentBounds, ComponentList, Entity, InspectorParameters,
};
use imgui::im_str;
use regex::Regex;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Name {
    pub name: String,
}

impl Name {
    const INDENT_AMOUNT: f32 = 38.0;

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }

    pub fn inspect(
        name: &str,
        eli: &mut EntityListInformation,
        nip: NameInspectorParameters,
        ui: &imgui::Ui<'_>,
        uid: &str,
    ) -> NameInspectorResult {
        let mut res = NameInspectorResult::default();
        let depth = nip.depth as f32;

        if depth > 0.0 {
            ui.dummy([depth * Name::INDENT_AMOUNT, 0.0]);
            ui.same_line(0.0);
        }

        // The shape we're making looks like this:
        // > CLOSED:
        // [] NO CHILDREN:
        // 1 Open: That's the down arrow but I couldn't find the glyph
        // let gap_amount;

        if nip.has_children {
            ui.text(&imgui::im_str!(
                "{}",
                if eli.open == false {
                    '\u{f105}' // Right Chevron
                } else {
                    '\u{f107}' // Down Chevron
                }
            ));

            if imgui_system::left_clicked_item(ui) {
                eli.open = !eli.open;
            }
        } else {
            // this is here actually just to make the lower
            // same_line call work for both branches, for simplicity!
            ui.dummy([10.0, 0.0]);
        }
        res.show_children = eli.open;

        // Get us to the next evenish column:
        ui.same_line(
            ((ui.cursor_pos()[0] / (Name::INDENT_AMOUNT * (depth + 1.0))).floor() + 1.0)
                * (Name::INDENT_AMOUNT * (depth + 1.0)),
        );

        // Object Symbol:
        ui.text_colored(
            if nip.is_prefab {
                // Nice Blue
                Color::with_u8(188, 203, 222, 255)
            } else {
                // Nice Grey
                Color::with_u8(225, 225, 225, 255)
            }
            .into(),
            // the dice from FontAwesome
            &imgui::im_str!("{}", '\u{f6d1}'),
        );
        ui.same_line(0.0);

        // Actually Name:
        if eli.edit_name != NameEdit::NoEdit {
            let mut current_name = imgui::im_str!("{}", name);

            if ui
                .input_text(&imgui::im_str!("##New Name{}", uid), &mut current_name)
                .resize_buffer(true)
                .build()
            {
                res.serialize_name = Some(current_name.to_string());
            }

            let mut end_rename = false;

            if ui.is_item_deactivated_after_edit() {
                end_rename = true;
            }

            ui.same_line(0.0);
            if ui.button(&im_str!("Rename##{}", uid), [0.0, 0.0]) {
                end_rename = true;
            }

            if end_rename {
                eli.edit_name = NameEdit::NoEdit;
                res.serialize_name = Some(current_name.to_string());
            }
        } else {
            ui.text_colored(eli.color.into(), &imgui::im_str!("{}", name));
        }

        // Manage the color...
        eli.color = Color::with_u8(202, 205, 210, 255);
        if ui.is_item_hovered() {
            eli.color = Color::WHITE;
        }
        if nip.being_inspected {
            eli.color = Color::with_u8(252, 195, 108, 255); // nice orange
        }

        // Inspect on Single Click
        if imgui_system::left_clicked_item(ui) && eli.edit_name == NameEdit::NoEdit {
            res.inspect = true;
        }

        // Rename on Double Click
        let mut rename = false;
        if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
            rename = true;
        }

        // Clone and Delete will be here!
        imgui_system::right_click_popup(ui, uid, || {
            if ui.button(&im_str!("Rename##{}", uid), [0.0, 0.0]) {
                rename = true;
                ui.close_current_popup();
            }

            ui.same_line(0.0);

            if ui.button(&im_str!("Clone##{}", uid), [0.0, 0.0]) {
                res.clone = true;
                ui.close_current_popup();
            }

            ui.same_line(0.0);

            if ui.button(&im_str!("Delete##{}", uid), [0.0, 0.0]) {
                res.delete = true;
            }

            ui.same_line(0.0);

            if nip.is_serialized {
                if ui.button(&im_str!("Unserialize##{}", uid), [0.0, 0.0]) {
                    res.unserialize = true;
                }
            }

            ui.same_line(0.0);
            if ui.button(&im_str!("Console Dump##{}", uid), [0.0, 0.0]) {
                res.dump_into_console_log = true;
                ui.close_current_popup();
            }
        });

        if rename {
            eli.edit_name = NameEdit::First;
            res.serialize_name = Some(name.to_string());
        }

        res
    }

    pub fn update_name(&mut self, our_id: Entity, all_names: &ComponentList<Name>) {
        lazy_static::lazy_static! {
            static ref REGEX_PATTERN: Regex = Regex::new(r"\(\d*\d\)$").unwrap();
        }

        loop {
            if let Some(mat) = REGEX_PATTERN.find(&self.name) {
                // Add one to go over the "("...
                let byte_offset = mat.start() + 1;

                // add one!
                let integer = self.name.as_bytes()[byte_offset] + 1;
                unsafe {
                    self.name.as_mut_vec()[byte_offset] = integer;
                }
            } else {
                self.name += " (0)";
            }

            if all_names
                .iter()
                .any(|name| name.inner().name == self.name && name.entity_id() != our_id)
                == false
            {
                break;
            }
        }
    }

    pub fn get_name_quick(names: &ComponentList<Name>, id: &Entity) -> String {
        if let Some(name) = names.get(id) {
            name.inner().name.clone()
        } else {
            format!("Entity ID {}", id.index())
        }
    }

    pub fn get_name(names: Option<&ComponentList<Name>>, id: &Entity) -> String {
        if let Some(names) = names {
            if let Some(name) = names.get(id) {
                name.inner().name.clone()
            } else {
                format!("Entity ID {}", id.index())
            }
        } else {
            format!("Entity ID {}", id.index())
        }
    }
}

impl ComponentBounds for Name {
    // Don't use this -- use the imgui function in the above impl
    fn entity_inspector(&mut self, _inspector_parameters: InspectorParameters<'_, '_>) {
        unimplemented!();
    }

    fn is_serialized(&self, serialized_entity: &super::SerializedEntity, active: bool) -> bool {
        serialized_entity
            .name
            .as_ref()
            .map_or(false, |(c, a)| *a == active && c == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.name = Some((self.clone(), active));
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.name = None;
    }
}
