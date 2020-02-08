use super::*;

pub fn entity_inspector(
    component_database: &mut ComponentDatabase,
    singleton_database: &mut SingletonDatabase,
    resources: &mut ResourcesDatabase,
    entities: &[Entity],
    ui_handler: &mut UiHandler<'_>,
) {
    let ui: &mut Ui<'_> = &mut ui_handler.ui;
    let mut remove_this_entity = None;

    for entity in ui_handler.stored_ids.iter() {
        let mut is_open = true;

        let name = {
            match component_database.names.get_mut(entity) {
                Some(name) => im_str!("{} (Scene Entity)##{}", &name.inner().name, entity),
                None => im_str!("{} (Scene Entity)", entity),
            }
        };

        let entity_window = imgui::Window::new(&name)
            .size([600.0, 800.0], imgui::Condition::FirstUseEver)
            .position([1200.0, 100.0], imgui::Condition::FirstUseEver)
            .menu_bar(true)
            .opened(&mut is_open);

        if let Some(entity_inspector_window) = entity_window.begin(ui) {
            let serialization_id = component_database
                .serialization_data
                .get(entity)
                .map(|sd| sd.inner().id);

            // This unsafety is not actually unsafe at all -- Rust doesn't yet realize
            // that this method, though it takes `component_database`, doesn't involve
            // the field .names within component_database. If we use names, then this would
            // become a lot trickier.
            let names_raw_pointer: *const ComponentList<Name> = &component_database.names;
            component_database.foreach_component_list_no_name_or_prefab(|component_list| {
                component_list.component_inspector(
                    entities,
                    unsafe { &*names_raw_pointer },
                    entity,
                    &resources.prefabs,
                    ui,
                    is_open,
                );
            });

            // Prefab
            component_database.prefab_markers.component_inspector(
                entities,
                &component_database.names,
                entity,
                &resources.prefabs,
                ui,
                is_open,
            );

            // Serialization
            if component_database.serialization_data.get(entity).is_none() {
                if let Some(id) = serialization_id {
                    if let Err(e) = serialization_util::entities::unserialize_entity(&id) {
                        error!("Couldn't unserialize! {}", e);
                    }
                }
            }

            // Menu bar funtimes!
            if let Some(menu_bar) = ui.begin_menu_bar() {
                if let Some(add_component_submenu) = ui.begin_menu(im_str!("Add Component"), true) {
                    // @update_components exception
                    let had_transform = component_database.transforms.get(entity).is_some();

                    // Prefab Marker, Name is omitted
                    component_database.foreach_component_list_no_name_or_prefab(|component_list| {
                        component_list.component_add_button(entity, ui)
                    });

                    if had_transform == false {
                        if let Some(new_transform) = component_database.transforms.get_mut(entity) {
                            scene_graph::add_to_scene_graph(
                                new_transform,
                                &component_database.serialization_data,
                            );
                        }
                    }

                    add_component_submenu.end(ui);
                }

                if let Some(serialization_submenu) = ui.begin_menu(
                    im_str!("Serialization"),
                    component_database.serialization_data.get(entity).is_some(),
                ) {
                    if let Some(comp) = component_database.serialization_data.get(entity) {
                        match comp
                            .inner()
                            .entity_inspector_serde(ui, entity, component_database)
                        {
                            Ok(Some(command)) => {
                                serialization_util::entities::process_serialized_command(
                                    entity,
                                    command,
                                    component_database,
                                    singleton_database,
                                    &resources.prefabs,
                                );
                            }
                            Ok(None) => {}
                            Err(e) => {
                                error!("{}", e);
                            }
                        };
                    }

                    serialization_submenu.end(ui);
                }

                if let Some(prefab_submenu) = ui.begin_menu(
                    im_str!("Create Prefab"),
                    component_database.prefab_markers.get(entity).is_none(),
                ) {
                    let mut prefab_to_instantiate: Option<uuid::Uuid> = None;

                    if imgui::MenuItem::new(im_str!("New Prefab")).build(ui) {
                        match prefab_system::create_blank_prefab(resources) {
                            Ok(uuid) => prefab_to_instantiate = Some(uuid),
                            Err(e) => error!("Couldn't create prefab: {}", e),
                        }
                    }

                    if let Some(overwrite_submenu) = ui.begin_menu(
                        im_str!("Overwrite Prefab"),
                        resources.prefabs.is_empty() == false,
                    ) {
                        for prefab in resources.prefabs.values() {
                            let name = match &prefab.name {
                                Some((name, _)) => im_str!("{}", &name.name),
                                None => im_str!("ID: {}", prefab.id),
                            };

                            if imgui::MenuItem::new(&name).build(ui) {
                                prefab_to_instantiate = Some(prefab.id);
                            }
                        }

                        overwrite_submenu.end(ui);
                    }

                    if let Some(prefab_to_instantiate) = prefab_to_instantiate {
                        // Create a serialized entity
                        let mut serialized_entity =
                            SerializedEntity::new(entity, component_database, singleton_database);
                        serialized_entity.id = prefab_to_instantiate;

                        // Add our Prefab Marker
                        component_database.prefab_markers.set(
                            entity,
                            Component::new(
                                entity,
                                PrefabMarker {
                                    id: prefab_to_instantiate,
                                },
                            ),
                        );

                        if let Err(e) =
                            serialization_util::prefabs::serialize_prefab(&serialized_entity)
                        {
                            error!("Error Creating Prefab: {}", e);
                        }

                        match serialization_util::prefabs::cycle_prefab(serialized_entity) {
                            Ok(prefab) => {
                                resources.prefabs.insert(prefab.id, prefab);
                            }
                            Err(e) => {
                                error!("We couldn't cycle the Prefab! It wasn't saved! {}", e)
                            }
                        }
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
