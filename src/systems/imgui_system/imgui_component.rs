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
                Some(name) => im_str!("{} (Scene Entity)", &name.inner().name),
                None => im_str!("{} (Scene Entity)", entity),
            }
        };

        let entity_window = imgui::Window::new(&name)
            .size([200.0, 400.0], imgui::Condition::FirstUseEver)
            .menu_bar(true)
            .opened(&mut is_open);

        if let Some(entity_inspector_window) = entity_window.begin(ui) {
            // @update_components:
            let is_prefab = component_database.prefab_markers.get(entity).is_some();

            macro_rules! component_inspector_quick {
                ( $( $x:ident ),* ) => {
                    $(
                        component_inspector(
                            &mut component_database.$x,
                            entities,
                            &component_database.names,
                            entity,
                            &resources.prefabs,
                            ui,
                            is_open,
                        );
                    )*
                };
            }

            let disabled_token = if is_prefab {
                Some(ui.push_style_var(imgui::StyleVar::Alpha(0.2)))
            } else {
                None
            };

            component_inspector_quick!(
                players,
                transforms,
                velocities,
                sprites,
                sound_sources,
                bounding_boxes,
                draw_rectangles,
                tilemaps,
                text_sources,
                follows,
                conversant_npcs,
                prefab_markers
            );

            if let Some(disabled_token) = disabled_token {
                disabled_token.pop(ui);
            }

            // graph nodes
            // if let Some(graph_node) = component_database.graph_nodes.get_mut(entity) {
            //     ui.separator();

            //     let mut comp_info = graph_node.construct_component_info();
            //     let name = imgui_utility::typed_text_ui::<GraphNode>();
            //     component_name_and_status(&name, ui, &mut comp_info);
            //     graph_node.take_component_info(&comp_info);

            //     use typename::TypeName;
            //     // DELETE OR INSPECT
            //     ui.tree_node(&imgui::ImString::new(name))
            //         .default_open(true)
            //         .build(|| {
            //             let inspector_parameters = InspectorParameters {
            //                 is_open,
            //                 uid: &format!("{}{}", graph_node.entity_id.to_string(), &GraphNode::type_name()),
            //                 ui,
            //                 entities,
            //                 entity_names: &component_database.names,
            //                 prefabs: &resources.prefabs,
            //             };
            //             let id = graph_node.entity_id;
            //             graph_node.inner_mut().specific_entity_inspector(
            //                 id,
            //                 inspector_parameters,
            //                 &component_database.serialization_data,
            //                 &mut component_database.transforms,
            //             );
            //         });
            // }

            // serialization
            if let Some(comp) = component_database.serialization_data.get_mut(entity) {
                ui.separator();
                let name = imgui_utility::typed_text_ui::<SerializationData>();
                let mut comp_info = comp.construct_component_info();

                ui.tree_node(&imgui::ImString::new(&name))
                    .default_open(true)
                    .frame_padding(false)
                    .build(|| {
                        if ui.button(im_str!("Stop Serialization"), [-1.0, 0.0]) {
                            comp_info.is_deleted = true;
                        }
                    });

                if comp_info.is_deleted {
                    if let Err(e) =
                        serialization_util::entities::unserialize_entity(&comp.inner().id)
                    {
                        error!("Couldn't unserialize! {}", e);
                    }
                    component_database.serialization_data.unset(entity);
                }
            }

            // Menu bar funtimes!
            if let Some(menu_bar) = ui.begin_menu_bar() {
                if let Some(add_component_submenu) = ui.begin_menu(im_str!("Add Component"), true) {
                    macro_rules! add_component_quick {
                        ( $( $x:ident ),* ) => {
                            $(
                                component_add_button(ui, &mut component_database.$x, entity);
                            )*
                        };
                    }

                    // ADD COMPONENT?
                    // @update_components
                    add_component_quick!(
                        players,
                        transforms,
                        velocities,
                        graph_nodes,
                        sprites,
                        sound_sources,
                        bounding_boxes,
                        draw_rectangles,
                        tilemaps,
                        text_sources,
                        follows,
                        conversant_npcs
                    );

                    // Prefab Marker, Name is omitted

                    // When we add a serialize button, we serialize the whole entity.
                    if component_add_button(ui, &mut component_database.serialization_data, entity)
                    {
                        serialization_util::entities::serialize_entity_full(
                            entity,
                            component_database,
                            singleton_database,
                        );
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
                        match prefabs::create_blank_prefab(resources) {
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
            remove_this_entity = Some(entity.clone());
        }
    }

    if let Some(entity) = remove_this_entity {
        ui_handler.stored_ids.remove(&entity);
    }
}

fn component_inspector<T: ComponentBounds + typename::TypeName>(
    component_list: &mut ComponentList<T>,
    entities: &[Entity],
    entity_names: &ComponentList<Name>,
    entity: &Entity,
    prefab_hashmap: &std::collections::HashMap<uuid::Uuid, SerializedEntity>,
    ui: &mut Ui<'_>,
    is_open: bool,
) {
    if let Some(comp) = component_list.get_mut(entity) {
        let delete_component =
            component_inspector_internal(comp, entities, entity_names, prefab_hashmap, ui, is_open);
        if delete_component {
            component_list.unset(entity);
        }
    }
}

fn component_inspector_internal<T: ComponentBounds + typename::TypeName>(
    comp: &mut Component<T>,
    entities: &[Entity],
    entity_names: &ComponentList<Name>,
    prefabs: &std::collections::HashMap<uuid::Uuid, SerializedEntity>,
    ui: &mut Ui<'_>,
    is_open: bool,
) -> bool {
    let mut delete = false;
    let name = imgui_utility::typed_text_ui::<T>();

    ui.tree_node(&imgui::ImString::new(&name))
        .default_open(true)
        .frame_padding(false)
        .build(|| {
            // COMPONENT INFO
            let mut comp_info = comp.construct_component_info();
            component_name_and_status(&name, ui, &mut comp_info);
            comp.take_component_info(&comp_info);

            // DELETE ENTITY
            if comp_info.is_deleted {
                delete = true;
            } else {
                let inspector_parameters = InspectorParameters {
                    is_open,
                    uid: &format!("{}{}", comp.entity_id.to_string(), &T::type_name()),
                    ui,
                    entities,
                    entity_names,
                    prefabs,
                };
                comp.inner_mut().entity_inspector(inspector_parameters);
            }
        });

    delete
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

fn component_add_button<T: ComponentBounds + typename::TypeName + Default>(
    ui: &mut Ui<'_>,
    component_list: &mut ComponentList<T>,
    entity: &Entity,
) -> bool {
    if imgui::MenuItem::new(&imgui::ImString::new(imgui_utility::typed_text_ui::<T>()))
        .enabled(component_list.get(entity).is_none())
        .build(ui)
    {
        component_list.set(entity, Component::new(entity, T::default()));
        true
    } else {
        false
    }
}
