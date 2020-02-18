use super::*;
use failure::Fallible;
use imgui::{Condition, ImString, MenuItem, Window};

pub fn entity_inspector(ecs: &mut Ecs, resources: &mut ResourcesDatabase, ui_handler: &mut UiHandler<'_>) {
    let ui: &mut Ui<'_> = &mut ui_handler.ui;
    let mut remove_this_entity = None;

    let Ecs {
        component_database,
        singleton_database,
        entity_allocator,
        entities,
    } = ecs;

    let scene_is_prefab = scene_system::CURRENT_SCENE.lock().unwrap().is_prefab();

    for entity in ui_handler.stored_ids.iter() {
        let mut is_open = true;

        let name = {
            match component_database.names.get_mut(entity) {
                Some(name) => im_str!("{} (Scene Entity)###{}", &name.inner().name, entity),
                None => im_str!("{} (Scene Entity)", entity),
            }
        };

        let entity_window = Window::new(&name)
            .size([600.0, 800.0], Condition::FirstUseEver)
            .position([1200.0, 100.0], Condition::FirstUseEver)
            .menu_bar(true)
            .opened(&mut is_open);

        if let Some(entity_inspector_window) = entity_window.begin(ui) {
            // This unsafety is not actually unsafe at all -- Rust doesn't yet realize
            // that this method, though it takes `component_database`, doesn't involve
            // the field .names within component_database. If we use names, then this would
            // become a lot trickier.
            let names_raw_pointer: *const ComponentList<Name> = &component_database.names;
            component_database.foreach_component_list_mut(NonInspectableEntities::PREFAB, |component_list| {
                component_list.component_inspector(
                    entities,
                    unsafe { &*names_raw_pointer },
                    entity,
                    resources.prefabs(),
                    ui,
                    is_open,
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
            component_database.serialization_markers.component_inspector_raw(
                entities,
                &component_database.names,
                entity,
                resources.prefabs(),
                ui,
                is_open,
                |inner, ip| {
                    serialize_it = inner.entity_inspector_results(ip);
                },
            );

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

        if is_open == false {
            remove_this_entity = Some(*entity);
        }
    }

    if let Some(entity) = remove_this_entity {
        ui_handler.stored_ids.remove(&entity);
    }
}

pub fn entity_serialization_options(
    serialized_marker: &SerializationMarker,
    ui: &mut Ui<'_>,
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

// @techdebt this is weirdly public, maybe we put it in the
// utilities. It's shared between Components and Prefabs!
pub fn component_name_and_status(name: &str, ui: &mut Ui<'_>, component_info: &mut ComponentInfo) {
    // NAME
    let two_thirds_size = ui.window_size()[0] * (2.0 / 3.0);
    ui.same_line(two_thirds_size);
    ui.checkbox(&im_str!("##Active{}", name), &mut component_info.is_active);
    ui.same_line(two_thirds_size + 25.0);

    let label = &im_str!("Delete##{}", name);
    let base_size: Vec2 = ui.calc_text_size(label, true, 0.0).into();
    let size = base_size + Vec2::new(13.0, 6.5);
    if ui.button(label, size.into()) {
        component_info.is_deleted = true;
    }

    ui.spacing();
}

impl<T> ComponentList<T>
where
    T: ComponentBounds + Clone + typename::TypeName + 'static,
{
    pub fn component_inspector_raw<F>(
        &mut self,
        entities: &[Entity],
        entity_names: &ComponentList<Name>,
        entity: &Entity,
        prefab_hashmap: &PrefabMap,
        ui: &mut Ui<'_>,
        is_open: bool,
        mut f: F,
    ) where
        F: FnMut(&mut T, InspectorParameters<'_, '_>),
    {
        if let Some(comp) = self.get_mut(entity) {
            let delete_component = {
                let mut delete = false;
                let name = super::imgui_system::typed_text_ui::<T>();

                ui.tree_node(&imgui::ImString::new(&name))
                    .default_open(true)
                    .frame_padding(false)
                    .build(|| {
                        // COMPONENT INFO
                        let mut comp_info = comp.construct_component_info();
                        super::imgui_system::component_name_and_status(&name, ui, &mut comp_info);
                        comp.take_component_info(&comp_info);

                        // DELETE ENTITY
                        if comp_info.is_deleted {
                            delete = true;
                        } else {
                            let inspector_parameters = InspectorParameters {
                                is_open,
                                uid: &format!("{}{}", comp.entity_id(), &T::type_name()),
                                ui,
                                entities,
                                entity_names,
                                prefabs: prefab_hashmap,
                            };
                            f(comp.inner_mut(), inspector_parameters);
                        }
                    });

                delete
            };
            if delete_component {
                self.unset(entity);
            }
        }
    }

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
