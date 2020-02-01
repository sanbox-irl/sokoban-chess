use super::{
    component_utils::{NameInspectorParameters, NameInspectorResult},
    imgui_system, Color, ComponentBounds, ComponentList, Entity, InspectorParameters,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Name {
    pub name: String,
    #[serde(skip)]
    pub open: bool,
    #[serde(skip)]
    pub color: Color,
}

impl Name {
    const INDENT_AMOUNT: f32 = 38.0;

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            open: false,
            color: Color::WHITE,
        }
    }

    pub fn inspect(
        &mut self,
        ui: &imgui::Ui<'_>,
        nip: NameInspectorParameters,
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
                if self.open == false {
                    '\u{f105}' // Right Chevron
                } else {
                    '\u{f107}' // Down Chevron
                }
            ));

            if imgui_system::left_clicked_item(ui) {
                self.open = !self.open;
            }
        } else {
            // this is here actually just to make the lower
            // same_line call work for both branches, for simplicity!
            ui.dummy([10.0, 0.0]);
        }
        res.show_children = self.open;

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
        ui.text_colored(self.color.into(), &imgui::im_str!("{}", self.name));

        // Manage the color...
        self.color = Color::WHITE;
        if (ui.is_item_hovered() && ui.is_mouse_down(imgui::MouseButton::Left)) || nip.being_inspected {
            self.color = Color::with_u8(252, 195, 108, 255); // nice orange
        }

        if imgui_system::left_clicked_item(ui) {
            res.inspect = true;
        }

        imgui_system::right_click_popup(ui, uid, || {
            ui.text("Hey there lassie, this isn't done yet. But Rename, Delete, and Clone will be here..");
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
}

impl ComponentBounds for Name {
    // Don't use this -- use the imgui function in the above impl
    fn entity_inspector(&mut self, _inspector_parameters: InspectorParameters<'_, '_>) {
        unimplemented!();
    }
}
