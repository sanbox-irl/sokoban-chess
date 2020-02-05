use super::{
    component_utils::{
        EntityListInformation, NameEdit, NameInspectorParameters, NameInspectorResult,
    },
    imgui_system, Color, ComponentBounds, ComponentList, Entity, InspectorParameters,
};
use imgui::im_str;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
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

            if ui.is_item_deactivated_after_edit() {
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
        if imgui_system::left_clicked_item(ui) {
            res.inspect = true;
        }

        // Rename on Double Click
        if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
            eli.edit_name = NameEdit::First;
            res.serialize_name = Some(name.to_string());
        }

        // Clone and Delete will be here!
        imgui_system::right_click_popup(ui, uid, || {
            if ui.button(&im_str!("Clone##{}", uid), [0.0, 0.0]) {
                res.clone = true;
            }

            if ui.button(&im_str!("Delete##{}", uid), [0.0, 0.0]) {
                res.delete = true;
            }
        });

        res
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
}
