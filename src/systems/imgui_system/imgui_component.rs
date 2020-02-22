use super::{imgui_component_utils::*, *};
use imgui::{Condition, ImStr, ImString, MenuItem, StyleColor, StyleVar, Window};
use imgui_utility::imgui_str;

pub fn entity_inspector(ecs: &mut Ecs, resources: &mut ResourcesDatabase, ui_handler: &mut UiHandler<'_>) {
    let ui: &Ui<'_> = &ui_handler.ui;
    let mut remove_this_entity = None;
    let mut final_post_action: Option<ComponentInspectorPostAction> = None;


    let Ecs {
        component_database,
        singleton_database,
        entity_allocator: _,
        entities,
    } = ecs;

    let scene_is_prefab = scene_system::CURRENT_SCENE.lock().unwrap().is_prefab();

    for entity in ui_handler.stored_ids.iter() {
        let mut window_is_open = true;

        let window_name = {
            match component_database.names.get_mut(entity) {
                Some(name) => im_str!("{} (Scene Entity)###{}", &name.inner().name, entity),
                None => im_str!("{} (Scene Entity)", entity),
            }
        };

        let serialized_prefab = {
            let mut base_entity = SerializedEntity::default();

            if prefab_system::get_serialized_parent_prefab_from_inheritor(
                component_database.prefab_markers.get(entity),
                resources,
                &mut base_entity,
            ) {
                Some(base_entity)
            } else {
                None
            }
        };

        let names = &component_database.names;
        let serialized_entity = component_database
            .serialization_markers
            .get_mut(entity)
            .and_then(|se| {
                let base_entity = serialized_prefab.clone().unwrap_or_default();

                let cached_se: SerializedEntity = se
                    .inner_mut()
                    .cached_serialized_entity()
                    .cloned()
                    .unwrap_or_default();

                match prefab_system::load_override_into_prefab(base_entity, cached_se) {
                    Ok(se) => Some(se),
                    Err(e) => {
                        error!(
                            "We failed to override our prefab for {} because {}",
                            Name::get_name_quick(names, entity),
                            e
                        );
                        None
                    }
                }
            });

        let entity_window = Window::new(&window_name)
            .size([600.0, 800.0], Condition::FirstUseEver)
            .position([1200.0, 100.0], Condition::FirstUseEver)
            .menu_bar(true)
            .opened(&mut window_is_open);

        if let Some(entity_inspector_window) = entity_window.begin(ui) {
            // This unsafety is not actually unsafe at all -- Rust doesn't yet realize
            // that this method, though it takes `component_database`, doesn't involve
            // the field .names within component_database. If we use names, then this would
            // become a lot trickier.
            let names_raw_pointer: *const _ = &component_database.names;

            component_database.foreach_component_list_mut(NonInspectableEntities::PREFAB, |component_list| {
                if let Some(post_inspector) = component_list.component_inspector(
                    entity,
                    serialized_entity.as_ref(),
                    serialized_prefab.as_ref(),
                    entities,
                    unsafe { &*names_raw_pointer },
                    resources.prefabs(),
                    ui,
                    window_is_open,
                ) {
                    final_post_action = Some(post_inspector);
                }
            });

            let prefab_status = if scene_is_prefab {
                PrefabStatus::Prefab
            } else {
                if component_database.prefab_markers.get(entity).is_some() {
                    PrefabStatus::PrefabInstance
                } else {
                    PrefabStatus::None
                }
            };

            // Serialization Inspector
            if let Some(s_marker) = component_database.serialization_markers.get_mut(entity) {
                component_inspector_raw(
                    s_marker,
                    SyncStatus::Synced,
                    SyncStatus::Unsynced,
                    entities,
                    &component_database.names,
                    resources.prefabs(),
                    ui,
                    window_is_open,
                    |inner, ip| {
                        if inner.entity_inspector_results(ip) {
                            final_post_action = Some(ComponentInspectorPostAction::EntityCommands(
                                EntitySerializationCommand {
                                    id: inner.id,
                                    command_type: EntitySerializationCommandType::Overwrite,
                                },
                            ))
                        }
                    },
                );
            }

            // Menu bar funtimes!
            if let Some(menu_bar) = ui.begin_menu_bar() {
                let component_add_button_text = if prefab_status == PrefabStatus::PrefabInstance {
                    "Add Override"
                } else {
                    "Add Component"
                };

                if let Some(add_component_submenu) =
                    ui.begin_menu(&ImString::new(component_add_button_text), true)
                {
                    // @update_components exception
                    let had_transform = component_database.transforms.get(entity).is_some();

                    // Prefab Marker, Name, Graph Node is omitted
                    component_database.foreach_component_list_mut(
                        NonInspectableEntities::SERIALIZATION,
                        |component_list| component_list.component_add_button(entity, ui),
                    );

                    if had_transform == false {
                        if let Some(new_transform) = component_database.transforms.get_mut(entity) {
                            scene_graph::add_to_scene_graph(
                                new_transform,
                                &component_database.serialization_markers,
                            );
                        }
                    }

                    add_component_submenu.end(ui);
                }

                if let Some(serialization_submenu) = ui.begin_menu(
                    im_str!("Serialize"),
                    component_database.serialization_markers.get(entity).is_some(),
                ) {
                    if let Some(comp) = component_database.serialization_markers.get(entity) {
                        if let Some(post_inspector) = entity_serialization_options(
                            comp.inner(),
                            ui,
                            entity,
                            component_database,
                        ) {
                            final_post_action = Some(post_inspector);
                        };
                    }

                    serialization_submenu.end(ui);
                }

                if let Some(prefab_submenu) =
                    ui.begin_menu(im_str!("Create Prefab"), prefab_status == PrefabStatus::None)
                {
                    let mut new_prefab_to_create: Option<uuid::Uuid> = None;

                    if MenuItem::new(im_str!("New Prefab")).build(ui) {
                        match prefab_system::commit_blank_prefab(resources) {
                            Ok(uuid) => new_prefab_to_create = Some(uuid),
                            Err(e) => error!("Couldn't create prefab: {}", e),
                        }
                    }

                    if let Some(overwrite_submenu) = ui.begin_menu(
                        im_str!("Overwrite Prefab"),
                        resources.prefabs().is_empty() == false,
                    ) {
                        for prefab in resources.prefabs().values() {
                            let name = match &prefab.root_entity().name {
                                Some(sc) => im_str!("{}", &sc.inner.name),
                                None => im_str!("ID: {}", prefab.root_id()),
                            };

                            if MenuItem::new(&name).build(ui) {
                                new_prefab_to_create = Some(prefab.root_id());
                            }
                        }

                        overwrite_submenu.end(ui);
                    }

                    if let Some(prefab_to_create) = new_prefab_to_create {
                        prefab_system::load_entity_into_prefab(
                            entity,
                            prefab_to_create,
                            component_database,
                            singleton_database,
                            resources,
                        );
                    }

                    prefab_submenu.end(ui);
                }
                menu_bar.end(ui);
            }

            entity_inspector_window.end(ui);
        }

        if window_is_open == false {
            remove_this_entity = Some(*entity);
        }
    }

    // This happens when someone closes a window
    if let Some(entity) = remove_this_entity {
        ui_handler.stored_ids.remove(&entity);
    }

    if let Some(final_post_action) = final_post_action {
        match final_post_action {
            ComponentInspectorPostAction::Serialize => (),
            ComponentInspectorPostAction::StopSerializing => (),
            ComponentInspectorPostAction::Revert => (),
            ComponentInspectorPostAction::ApplyOverrideToParentPrefab => (),
            ComponentInspectorPostAction::EntityCommands(_) => (),
        }
    }
}

pub fn entity_serialization_options(
    serialized_marker: &SerializationMarker,
    ui: &Ui<'_>,
    entity_id: &Entity,
    component_database: &ComponentDatabase,
) -> Option<ComponentInspectorPostAction> {
    let mut post_action = None;

    component_database.foreach_component_list(
        NonInspectableEntities::NAME | NonInspectableEntities::PREFAB | NonInspectableEntities::GRAPH_NODE,
        |component_list| {
            if let Some(post_inspector) = component_list.serialization_option(
                ui,
                entity_id,
                &component_database.serialization_markers,
            ) {
                post_action = Some(post_inspector);
            }
        },
    );

    ui.separator();

    // REVERT SAVE
    if ui.button(im_str!("Revert"), [0.0, 0.0]) {
        post_action = Some(ComponentInspectorPostAction::EntityCommands(
            EntitySerializationCommand {
                id: serialized_marker.id,
                command_type: EntitySerializationCommandType::Revert,
            },
        ));
    }

    // OVERWRITE
    ui.same_line(0.0);
    if ui.button(im_str!("Overwrite"), [0.0, 0.0]) {
        post_action = Some(ComponentInspectorPostAction::EntityCommands(
            EntitySerializationCommand {
                id: serialized_marker.id,
                command_type: EntitySerializationCommandType::Revert,
            },
        ));
    }

    post_action
}

pub fn component_inspector_raw<T>(
    comp: &mut Component<T>,
    serialization_sync_status: SyncStatus,
    prefab_sync_status: SyncStatus,
    entities: &[Entity],
    entity_names: &ComponentList<Name>,
    prefabs: &PrefabMap,
    ui: &Ui<'_>,
    is_open: bool,
    mut f: impl FnMut(&mut T, InspectorParameters<'_, '_>),
) -> Option<ComponentInspectorListAction>
where
    T: ComponentBounds + Clone + typename::TypeName + std::fmt::Debug + 'static,
{
    let mut requested_action = None;
    let scene_mode = scene_system::current_scene_mode();

    let name = super::imgui_system::typed_text_ui::<T>();
    let uid = &format!("{}{}", comp.entity_id(), &T::type_name());

    let alpha_controller = if comp.is_active == false {
        Some(ui.push_style_var(imgui::StyleVar::Alpha(0.6)))
    } else {
        None
    };

    let default_color = ui.style_color(imgui::StyleColor::Text);
    let text_color_token = ui.push_style_color(
        imgui::StyleColor::Text,
        serialization_sync_status.imgui_color(scene_mode),
    );
    ui.tree_node(&imgui::ImString::new(&name))
        .default_open(true)
        .frame_padding(false)
        .build(|| {
            imgui_utility::wrap_style_var(ui, StyleVar::Alpha(1.0), || {
                requested_action = component_inspector_right_click(
                    ui,
                    uid,
                    &mut comp.is_active,
                    default_color,
                    serialization_sync_status,
                    prefab_sync_status,
                );

                // Handle the Warning!
                match serialization_sync_status {
                    SyncStatus::Unsynced => {
                        if scene_mode == SceneMode::Draft {
                            imgui_utility::help_marker_generic(
                                ui,
                                imgui_utility::WARNING_ICON,
                                format!("{} is not committed to the Scene!", name),
                            )
                        }
                    }
                    SyncStatus::Headless => imgui_utility::help_marker_generic(
                        ui,
                        imgui_utility::WARNING_ICON,
                        format!(
                            "{} is Headless! We thought we had a serialization, but we don't!",
                            name
                        ),
                    ),
                    SyncStatus::OutofSync => imgui_utility::help_marker_generic(
                        ui,
                        imgui_utility::WARNING_ICON,
                        format!("{} is out of Sync with its Serialization!", name),
                    ),
                    SyncStatus::Synced => {}
                };
            });

            if comp.is_active {
                imgui_system::wrap_style_color_var(ui, imgui::StyleColor::Text, default_color, || {
                    let inspector_parameters = InspectorParameters {
                        is_open,
                        uid,
                        ui,
                        entities,
                        entity_names,
                        prefabs,
                    };
                    f(comp.inner_mut(), inspector_parameters);
                });
            }
        });

    text_color_token.pop(ui);
    if let Some(alpha_token) = alpha_controller {
        alpha_token.pop(ui);
    }

    requested_action
}

fn component_inspector_right_click(
    ui: &Ui<'_>,
    uid: &str,
    is_active: &mut bool,
    default_color: ImColor,
    serialization_sync_status: SyncStatus,
    prefab_sync_status: SyncStatus,
) -> Option<ComponentInspectorListAction> {
    let mut requested_action = None;

    imgui_system::right_click_popup(ui, uid, || {
        imgui_utility::wrap_style_color_var(ui, StyleColor::Text, default_color, || {
            MenuItem::new(&im_str!("Is Active##{}", uid)).build_with_ref(ui, is_active);

            if MenuItem::new(&im_str!("Delete##{}", uid)).build(ui) {
                requested_action = Some(ComponentInspectorListAction::Delete);
            }

            ui.separator();

            if MenuItem::new(&imgui_str("Serialize", uid))
                .enabled(serialization_sync_status == SyncStatus::OutofSync)
                .build(ui)
            {
                requested_action = Some(ComponentInspectorListAction::ComponentInspectorPostAction(
                    ComponentInspectorPostAction::Serialize,
                ));
            }

            if MenuItem::new(&imgui_str("Stop Serializing", uid))
                .enabled(serialization_sync_status.is_synced_at_all())
                .build(ui)
            {
                requested_action = Some(ComponentInspectorListAction::ComponentInspectorPostAction(
                    ComponentInspectorPostAction::StopSerializing,
                ));
            }

            if MenuItem::new(&imgui_str("Revert to Serialization", uid))
                .enabled(serialization_sync_status == SyncStatus::Unsynced)
                .build(ui)
            {
                requested_action = Some(ComponentInspectorListAction::ComponentInspectorPostAction(
                    ComponentInspectorPostAction::Revert,
                ));
            }

            ui.separator();

            if MenuItem::new(&imgui_str("Apply Overrides To Prefab", uid))
                .enabled(prefab_sync_status == SyncStatus::Unsynced)
                .build(ui)
            {
                requested_action = Some(ComponentInspectorListAction::ComponentInspectorPostAction(
                    ComponentInspectorPostAction::Serialize,
                ));
            }

            if MenuItem::new(&imgui_str("Revert to Prefab", uid))
                .enabled(prefab_sync_status == SyncStatus::OutofSync)
                .build(ui)
            {
                requested_action = Some(ComponentInspectorListAction::RevertToParentPrefab);
            }
        });
    });

    requested_action
}

impl<T> ComponentList<T>
where
    T: ComponentBounds + Clone + typename::TypeName + std::fmt::Debug + 'static,
{
    pub fn serialization_option_raw(
        &self,
        ui: &imgui::Ui<'_>,
        entity_id: &Entity,
        serialized_markers: &ComponentList<SerializationMarker>,
    ) -> Option<ComponentInspectorPostAction> {
        lazy_static::lazy_static! {
            static ref SERIALIZE: &'static ImStr = im_str!("Serialize");
            static ref DESERIALIZE: &'static ImStr = im_str!("Stop Serializing");
            static ref REVERT: &'static ImStr = im_str!("Revert");
        }

        let type_name = ImString::new(imgui_system::typed_text_ui::<T>());
        let component_exists = self.get(entity_id).is_some();
        let mut output = None;

        if let Some(my_serialization_marker) = serialized_markers.get(entity_id) {
            if let Some(serde_menu) = ui.begin_menu(&type_name, component_exists) {
                if let Some(component) = self.get(entity_id) {
                    // Serialize
                    if MenuItem::new(&SERIALIZE).build(ui) {
                        output = Some(ComponentInspectorPostAction::Serialize);
                    }

                    // Deserialize
                    if MenuItem::new(&DESERIALIZE).build(ui) {
                        output = Some(ComponentInspectorPostAction::StopSerializing);
                    }

                    // Revert
                    if MenuItem::new(&REVERT).build(ui) {
                        output = Some(ComponentInspectorPostAction::ApplyOverrideToParentPrefab);
                    }
                }
                serde_menu.end(ui);
            }
        }

        output
    }
}

/*

let serialized_entity = serialization_util::entities::load_committed_entity(
    &my_serialization_marker.inner(),
)?;

if let Some(mut serialized_entity) = serialized_entity {
    component.inner().commit_to_scene(
        &mut serialized_entity,
        component.is_active,
        serialized_markers,
    );
    serialization_util::entities::commit_entity_to_scene(serialized_entity)?;
    serialization_delta = SerializationDelta::Updated;
} else {
    error!(
        "Couldn't find a Serialized Entity for {}. Check the YAML?",
        entity_id
    );
}

*/

/*
let serialized_entity = serialization_util::entities::load_committed_entity(
    &my_serialization_marker.inner(),
)?;

if let Some(mut serialized_entity) = serialized_entity {
    component.inner().uncommit_to_scene(&mut serialized_entity);

    serialization_util::entities::commit_entity_to_scene(serialized_entity)?;
    serialization_delta = SerializationDelta::Updated;
} else {
    error!(
        "Couldn't find a Serialized Entity for {}. Check the YAML?",
        entity_id
    );
}
*/

/*
Ok((command, reload_prefab)) => {
if let Some(command) = command {
    serialization_util::entities::process_serialized_command(
        entity,
        command,
        component_database,
        singleton_database,
        entities,
        entity_allocator,
        resources,
    );
}

if reload_prefab {
    let prefab = resources.prefabs_mut().unwrap().get_mut(&id).unwrap();

    match serialization_util::entities::load_entity_by_id(&id) {
        Result::Ok(new_prefab) => {
            if let Some(new_prefab) = new_prefab {
                prefab.members.insert(id, new_prefab);
            } else {
                error!("We tried to reload Prefab with UUID {} but we couldn't find it. Did the file get deleted?", id);
            }
        }
        Result::Err(e) => {
            error!("Couldn't reload the prefab that we just edited. The current application is out of date! {}", e);
        }
    }
}
}

*/

/*
// THIS IS BEGIN SERIALIZATION BUTTON IN THE SERIALIZED MARKER INSPECTOR
if serialize_it {
    serialization_util::entities::serialize_entity_full(
        entity,
        component_database
            .serialization_markers
            .get(entity)
            .as_ref()
            .unwrap()
            .inner()
            .id,
        component_database,
        singleton_database,
        resources,
    );
}

*/
