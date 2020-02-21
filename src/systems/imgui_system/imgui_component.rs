use super::*;
use failure::Fallible;
use imgui::{Condition, ImString, MenuItem, StyleColor, StyleVar, Window};
use imgui_utility::imgui_str;

pub fn entity_inspector(ecs: &mut Ecs, resources: &mut ResourcesDatabase, ui_handler: &mut UiHandler<'_>) {
    let ui: &Ui<'_> = &ui_handler.ui;
    let mut remove_this_entity = None;

    let Ecs {
        component_database,
        singleton_database,
        entity_allocator,
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

        let serialized_entity = if let Some(se) = component_database.serialization_markers.get_mut(entity) {
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
                        Name::get_name_quick(&component_database.names, entity),
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

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
                component_list.component_inspector(
                    entity,
                    serialized_entity.as_ref(),
                    serialized_prefab.as_ref(),
                    entities,
                    unsafe { &*names_raw_pointer },
                    resources.prefabs(),
                    ui,
                    window_is_open,
                );
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

            // Serialization
            let mut serialize_it = false;
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
                        serialize_it = inner.entity_inspector_results(ip);
                    },
                );
            }

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

                let serialization_menu_text = if prefab_status == PrefabStatus::PrefabInstance {
                    "Serialize Overrides"
                } else {
                    "Serialization"
                };

                if let Some(serialization_submenu) = ui.begin_menu(
                    &ImString::new(serialization_menu_text),
                    component_database.serialization_markers.get(entity).is_some(),
                ) {
                    if let Some(comp) = component_database.serialization_markers.get(entity) {
                        let id = comp.inner().id;
                        match entity_serialization_options(
                            comp.inner(),
                            ui,
                            entity,
                            prefab_status,
                            component_database,
                        ) {
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
                            Err(e) => {
                                error!("{}", e);
                            }
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

    if let Some(entity) = remove_this_entity {
        ui_handler.stored_ids.remove(&entity);
    }
}

pub fn entity_serialization_options(
    serialized_marker: &SerializationMarker,
    ui: &Ui<'_>,
    entity_id: &Entity,
    prefab_status: PrefabStatus,
    component_database: &ComponentDatabase,
) -> Fallible<(Option<ImGuiSerializationDataCommand>, bool)> {
    // If this is a prefab we're inspecting, we're gonna do some stuff here!
    // compile_error!("Hey Jack we're here right now! We need to reload prefabs after we've edited them when we're in prefab mode!");
    let mut reload_prefab = false;

    component_database.foreach_component_list(
        NonInspectableEntities::NAME | NonInspectableEntities::PREFAB | NonInspectableEntities::GRAPH_NODE,
        |component_list| match component_list.serialization_option(
            ui,
            entity_id,
            prefab_status,
            &component_database.serialization_markers,
        ) {
            Ok(serialization_delta) => {
                if serialization_delta == SerializationDelta::Updated && prefab_status == PrefabStatus::Prefab
                {
                    reload_prefab = true;
                }
            }

            Err(e) => {
                error!("Error in Serialization Option {}", e);
            }
        },
    );

    ui.spacing();

    let mut sc = None;
    // REVERT SAVE
    if ui.button(im_str!("Revert"), [0.0, 0.0]) {
        sc = Some(ImGuiSerializationDataCommand {
            id: serialized_marker.id,
            serialization_type: ImGuiSerializationDataType::Revert,
        });
    }

    // OVERWRITE
    ui.same_line(0.0);
    if ui.button(im_str!("Overwrite"), [0.0, 0.0]) {
        sc = Some(ImGuiSerializationDataCommand {
            id: serialized_marker.id,
            serialization_type: ImGuiSerializationDataType::Overwrite,
        });
    }

    if sc.is_some() && prefab_status == PrefabStatus::Prefab {
        reload_prefab = true;
    }

    Ok((sc, reload_prefab))
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
) -> bool
where
    T: ComponentBounds + Clone + typename::TypeName + std::fmt::Debug + 'static,
{
    let mut delete_this_component = false;
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
                imgui_system::right_click_popup(ui, uid, || {
                    imgui_utility::wrap_style_color_var(ui, StyleColor::Text, default_color, || {
                        MenuItem::new(&im_str!("Is Active##{}", uid)).build_with_ref(ui, &mut comp.is_active);
                        MenuItem::new(&im_str!("Delete##{}", uid))
                            .build_with_ref(ui, &mut delete_this_component);

                        ui.separator();

                        if MenuItem::new(&imgui_str("Serialize", uid))
                            .enabled(serialization_sync_status == SyncStatus::OutofSync)
                            .build(ui)
                        {
                            info!("Let's serialize!")
                        }

                        if MenuItem::new(&imgui_str("Stop Serializing", uid))
                            .enabled(serialization_sync_status.is_synced_at_all())
                            .build(ui)
                        {
                            info!("Let's serialize!")
                        }

                        if MenuItem::new(&imgui_str("Revert to Serialization", uid))
                            .enabled(serialization_sync_status == SyncStatus::Unsynced)
                            .build(ui)
                        {
                            info!("Let's revert!")
                        }

                        ui.separator();

                        if MenuItem::new(&imgui_str("Apply Overrides To Prefab", uid))
                            .enabled(prefab_sync_status == SyncStatus::Unsynced)
                            .build(ui)
                        {
                            info!("Applying my overrides to Dad!");
                        }

                        if MenuItem::new(&imgui_str("Revert to Prefab", uid))
                            .enabled(prefab_sync_status == SyncStatus::OutofSync)
                            .build(ui)
                        {
                            info!("Reverting to Prefab DAD!");
                        }
                    });
                });

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

            if comp.is_active == false {
                return;
            }

            let normal_text_color = ui.push_style_color(imgui::StyleColor::Text, default_color);

            // DELETE ENTITY
            if delete_this_component != true {
                let inspector_parameters = InspectorParameters {
                    is_open,
                    uid,
                    ui,
                    entities,
                    entity_names,
                    prefabs,
                };
                f(comp.inner_mut(), inspector_parameters);
            }

            normal_text_color.pop(ui);
        });
    text_color_token.pop(ui);
    if let Some(alpha_token) = alpha_controller {
        alpha_token.pop(ui);
    }

    delete_this_component
}

impl<T> ComponentList<T>
where
    T: ComponentBounds + Clone + typename::TypeName + std::fmt::Debug + 'static,
{
    pub fn serialization_option_raw(
        &self,
        ui: &imgui::Ui<'_>,
        entity_id: &Entity,
        prefab_status: PrefabStatus,
        serialized_markers: &ComponentList<SerializationMarker>,
    ) -> failure::Fallible<SerializationDelta> {
        lazy_static::lazy_static! {
            static ref DEFAULT_SERIALIZE_TEXT: ImString = ImString::new("Serialize");
            static ref DEFAULT_DESERIALIZE_TEXT: ImString = ImString::new("Deserialize");
            static ref PREFAB_SERIALIZE_TEXT: ImString = ImString::new("Serialize Override");
            static ref PREFAB_DESERIALIZE_TEXT: ImString = ImString::new("Deserialize Override");
        }

        let type_name = ImString::new(imgui_system::typed_text_ui::<T>());
        let component_exists = self.get(entity_id).is_some();

        let mut serialization_delta = SerializationDelta::Unchanged;

        if let Some(my_serialization_marker) = serialized_markers.get(entity_id) {
            if let Some(serde_menu) = ui.begin_menu(&type_name, component_exists) {
                if let Some(component) = self.get(entity_id) {
                    // SERIALIZE
                    if MenuItem::new(if prefab_status == PrefabStatus::PrefabInstance {
                        &PREFAB_SERIALIZE_TEXT
                    } else {
                        &DEFAULT_SERIALIZE_TEXT
                    })
                    .build(ui)
                    {
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
                    }

                    // DESERIALIZE
                    if MenuItem::new(if prefab_status == PrefabStatus::PrefabInstance {
                        &PREFAB_DESERIALIZE_TEXT
                    } else {
                        &DEFAULT_DESERIALIZE_TEXT
                    })
                    .build(ui)
                    {
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
                    }
                }
                serde_menu.end(ui);
            }
        }
        Ok(serialization_delta)
    }
}
