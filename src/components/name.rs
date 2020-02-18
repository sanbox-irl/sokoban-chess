use super::{
    component_utils::{
        EntityListInformation, NameInspectorParameters, NameInspectorResult, NameRequestedAction,
        PrefabStatus,
    },
    imgui_system, Color, ComponentBounds, ComponentList, Entity, InspectorParameters, SyncStatus,
};
use imgui::{im_str, MenuItem};
use regex::Regex;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Name {
    pub name: String,
}

lazy_static::lazy_static! {
    pub static ref YELLOW_WARNING_COLOR: Color = Color::with_u8(253, 229, 109, 255);
    pub static ref RED_WARNING_COLOR: Color = Color::with_u8(238, 93, 67, 255);
    pub static ref BASE_GREY_COLOR: Color = Color::with_u8(202, 205, 210, 255);
}

impl Name {
    const INDENT_AMOUNT: f32 = 38.0;
    const RIGHT_CHEVRON: char = '\u{f105}';
    const DOWN_CHEVRON: char = '\u{f107}';
    const ENTITY_ICON: char = '\u{f6d1}';
    const WARNING_ICON: char = '\u{f071}';

    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn inspect(
        name: &str,
        eli: &mut EntityListInformation,
        nip: &NameInspectorParameters,
        ui: &imgui::Ui<'_>,
        uid: &str,
    ) -> NameInspectorResult {
        let mut res = NameInspectorResult::new();
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
                    Name::RIGHT_CHEVRON
                } else {
                    Name::DOWN_CHEVRON
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
            match nip.prefab_status {
                PrefabStatus::None => Color::with_u8(225, 225, 225, 255),
                PrefabStatus::Prefab => Color::with_u8(188, 203, 222, 255),
                PrefabStatus::PrefabInstance => Color::with_u8(188, 203, 222, 255),
            }
            .into(),
            &imgui::im_str!("{}", Name::ENTITY_ICON),
        );
        ui.same_line(0.0);

        // Actually Name:
        match &mut eli.edit_name {
            Some(editable_name) => {
                let mut current_name = imgui::im_str!("{}", editable_name);

                if ui
                    .input_text(&imgui::im_str!("##New Name{}", uid), &mut current_name)
                    .resize_buffer(true)
                    .build()
                {
                    *editable_name = current_name.to_string();
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
                    res.requested_action =
                        Some(NameRequestedAction::ChangeName(eli.edit_name.take().unwrap()));
                }
            }
            None => {
                ui.text_colored(eli.color.into(), &imgui::im_str!("{}", name));

                // Inspect on Single Click
                if imgui_system::left_clicked_item(ui) {
                    res.requested_action = Some(NameRequestedAction::ToggleInspect);
                }

                // Rename on Double Click
                imgui_system::right_click_popup(ui, uid, || {
                    if MenuItem::new(&im_str!("Rename##{}", uid)).build(ui) {
                        eli.edit_name = Some(name.to_string());
                        ui.close_current_popup();
                    }

                    if MenuItem::new(&im_str!("Clone##{}", uid)).build(ui) {
                        res.requested_action = Some(NameRequestedAction::Clone);
                        ui.close_current_popup();
                    }

                    if MenuItem::new(&im_str!("Delete##{}", uid)).build(ui) {
                        res.requested_action = Some(NameRequestedAction::Delete);
                    }

                    ui.separator();

                    ui.menu(im_str!("Serialization"), nip.serialization_status.is_synced_at_all(), || {
                        if imgui_system::help_menu_item(ui, &im_str!("Serialize Entity##{}", uid), "This is the exact same thing as Overwriting the Entity in the Component Inspect. It completely overwrites the comitted entity.") {
                            if nip.serialization_status == SyncStatus::OutofSync {
                                res.requested_action = Some(NameRequestedAction::Serialize);
                                ui.close_current_popup();
                            }
                        }
                        if imgui_system::help_menu_item(ui, &im_str!("Stop Serializing Entity##{}", uid), "Stops serializing the entity from the scene -- ie, it won't be here when you reload the scene.") {
                            res.requested_action = Some(NameRequestedAction::Unserialize);
                            ui.close_current_popup();
                        }
                    });

                    ui.separator();

                    ui.menu(&im_str!("Prefab"), true, || {
                        match nip.prefab_status {
                            PrefabStatus::None => {
                                if MenuItem::new(&im_str!("Promote to Prefab##{}", uid)).build(ui) {
                                    res.requested_action = Some(NameRequestedAction::PromoteToPrefab);
                                    ui.close_current_popup();
                                }
                            }
                            prefab_kind => {
                                if MenuItem::new(&im_str!("Unpack Prefab##{}", uid))
                                    .enabled(prefab_kind == PrefabStatus::PrefabInstance)
                                    .build(ui)
                                {
                                    res.requested_action = Some(NameRequestedAction::UnpackPrefab);
                                    ui.close_current_popup();
                                }
                            }
                        }

                        if MenuItem::new(&im_str!("Go To Prefab##{}", uid))
                            .enabled(nip.prefab_status != PrefabStatus::None)
                            .build(ui)
                        {
                            res.requested_action = Some(NameRequestedAction::GoToPrefab);
                            ui.close_current_popup();
                        }
                    });

                    ui.separator();

                    ui.menu(&im_str!("Log to Console"), true, || {
                        if MenuItem::new(&im_str!("Log Entity##{}", uid)).build(ui) {
                            res.requested_action = Some(NameRequestedAction::LogEntity);
                            ui.close_current_popup();
                        }

                        if MenuItem::new(&im_str!("Log Serialized Entity##{}", uid))
                            .enabled(nip.serialization_status.is_synced_at_all())
                            .build(ui)
                        {
                            res.requested_action = Some(NameRequestedAction::LogSerializedEntity);
                            ui.close_current_popup();
                        }

                        if MenuItem::new(&im_str!("Log Prefab##{}", uid))
                            .enabled(nip.prefab_status != PrefabStatus::None)
                            .build(ui)
                        {
                            res.requested_action = Some(NameRequestedAction::LogPrefab);
                            ui.close_current_popup();
                        }
                    });
                });

                // Manage the color...
                eli.color = *BASE_GREY_COLOR;
                if ui.is_item_hovered() {
                    eli.color = Color::WHITE;
                }
                if nip.being_inspected {
                    eli.color = *YELLOW_WARNING_COLOR;
                }

                if matches!(
                    nip.serialization_status,
                    SyncStatus::OutofSync | SyncStatus::Headless
                ) {
                    ui.same_line(0.0);
                    ui.text_colored(
                        if nip.serialization_status == SyncStatus::OutofSync {
                            *YELLOW_WARNING_COLOR
                        } else {
                            *RED_WARNING_COLOR
                        }
                        .into(),
                        im_str!("{}", Name::WARNING_ICON),
                    );
                }
            }
        }

        res
    }

    /// This function is comically dangerous for what it does. First, 'all_names' passed in will
    /// almost certainly have to be a raw pointer, since our self will be in all_names, but we need
    /// to mutate ourselves. We could have done this in two stages, but this seemed better for simplicity.
    /// Additionally, the unsafety in this is simply to write directly into the internal byte buffer of the vec
    /// So long as no patterns are matched by our regex which are of a *different* byte offset amount, we should
    /// be okay.
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
        let inner: Option<&str> = names.get(id).map(|nc| nc.inner().name.as_str());
        Name::get_name_even_quicklier(inner, &id)
    }

    pub fn get_name_even_quicklier<S: ToString>(maybe_name: Option<&str>, default: S) -> String {
        if let Some(name) = maybe_name {
            name.to_string()
        } else {
            default.to_string()
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
            .map_or(false, |s| s.active == active && &s.inner == self)
    }

    fn commit_to_scene(
        &self,
        se: &mut super::SerializedEntity,
        active: bool,
        _: &super::ComponentList<super::SerializationMarker>,
    ) {
        se.name = Some(super::SerializedComponent {
            inner: self.clone(),
            active,
        });
    }

    fn uncommit_to_scene(&self, se: &mut super::SerializedEntity) {
        se.name = None;
    }
}
