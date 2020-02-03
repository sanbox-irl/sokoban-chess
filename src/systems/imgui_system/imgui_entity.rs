use super::*;

pub fn entity_list(
    ecs: &mut Ecs,
    resources: &mut ResourcesDatabase,
    ui_handler: &mut UiHandler<'_>,
) {
    let mut open = true;

    // Top menu bar!
    let entity_window = imgui::Window::new(&im_str!("Entity List"))
        .size([200.0, 400.0], imgui::Condition::FirstUseEver)
        .menu_bar(true)
        .opened(&mut open);

    let mut entity_to_clone = None;
    let mut entity_to_delete = None;

    if let Some(entity_inspector_window) = entity_window.begin(&ui_handler.ui) {
        // Top menu bar!
        if let Some(menu_bar) = ui_handler.ui.begin_menu_bar() {
            let ui: &Ui<'_> = &ui_handler.ui;
            // BLANK ENTITY
            if imgui::MenuItem::new(im_str!("Create Blank Entity")).build(ui) {
                ecs.create_entity();
            }

            // PREFABS
            if let Some(prefab_submenu) = ui.begin_menu(im_str!("Instantiate Prefabs"), true) {
                for (prefab_id, prefab) in resources.prefabs.iter() {
                    let name = match &prefab.name {
                        Some((name, _)) => im_str!("{}##MenuItem", &name.name),
                        None => im_str!("ID: {}##MenuItem", prefab.id),
                    };

                    if imgui::MenuItem::new(&name).build(ui) {
                        prefab_system::create_new_prefab_entity(
                            ecs,
                            *prefab_id,
                            &resources.prefabs,
                        );
                    }
                }

                prefab_submenu.end(ui);
            }

            menu_bar.end(ui);
        }

        ui_handler.scene_graph_entities.clear();

        // SCENE GRAPH
        scene_graph::walk_graph_inspect(
            &ecs.component_database.transforms,
            &ecs.component_database.graph_nodes,
            &mut ecs.component_database.names,
            &ecs.component_database.prefab_markers,
            &ecs.component_database.serialization_data,
            &mut |entity, names, serialization_data, name_inspector_params| {
                ui_handler.scene_graph_entities.push(*entity);

                display_entity_id(
                    entity,
                    name_inspector_params,
                    names,
                    serialization_data,
                    ui_handler,
                    &mut entity_to_clone,
                    &mut entity_to_delete,
                )
            },
        );

        ui_handler.ui.separator();

        // ENTITY_GRAPH
        for entity in ecs.entities.iter_mut() {
            if ui_handler.scene_graph_entities.contains(entity) == false {
                let nip = NameInspectorParameters {
                    is_prefab: ecs.component_database.prefab_markers.get(entity).is_some(),
                    being_inspected: ui_handler.stored_ids.contains(entity),
                    ..Default::default()
                };

                display_entity_id(
                    entity,
                    nip,
                    &mut ecs.component_database.names,
                    &ecs.component_database.serialization_data,
                    ui_handler,
                    &mut entity_to_clone,
                    &mut entity_to_delete,
                );
            }
        }

        if let Some(original) = entity_to_clone {
            info!("Attempting to clone...");
            ecs.clone_entity(&original);
        }

        if let Some(entity_to_delete) = entity_to_delete {
            info!("Attempting to delete...");
            ecs.remove_entity(&entity_to_delete);
        }

        entity_inspector_window.end(&ui_handler.ui);
    }

    if open == false {
        ui_handler.flags.remove(ImGuiFlags::ENTITY_VIEWER);
    }
}

fn display_entity_id(
    entity: &Entity,
    mut name_inspector_params: NameInspectorParameters,
    names: &mut ComponentList<Name>,
    serialization_data: &ComponentList<SerializationData>,
    ui_handler: &mut UiHandler<'_>,
    clone_me: &mut Option<Entity>,
    delete_me: &mut Option<Entity>,
) -> bool {
    // Name Inspector Params
    name_inspector_params.being_inspected = ui_handler.stored_ids.contains(entity);

    let name = if let Some(name) = names.get_mut(entity) {
        name.inner_mut().name.clone()
    } else {
        format!("ID {}", entity)
    };

    // Get that fucker
    let mut entry = match ui_handler.entity_list_information.get_mut(entity) {
        Some(stuff) => stuff,
        None => {
            // for none stuff
            ui_handler
                .entity_list_information
                .insert(*entity, EntityListInformation::default());
            ui_handler.entity_list_information.get_mut(entity).unwrap()
        }
    };
    entry.new_name = None;

    let result = Name::inspect(
        &name,
        entry,
        name_inspector_params,
        &ui_handler.ui,
        &entity.index().to_string(),
    );

    if result.reserialize {
        // Serialize this Entity and add the name to it...
        if let Err(e) =
            SerializationData::edit_serialized_entity(&serialization_data, entity, |se| {
                se.name = names.get(entity).unwrap().fast_serialize()
            })
        {
            error!("COULDN'T SERIALIZE NAME: {}", e);
        }
    }

    if result.clone {
        *clone_me = Some(*entity);
    }

    if result.delete {
        *delete_me = Some(*entity);
    }

    // Store or Remove it...
    if result.inspect {
        if ui_handler.stored_ids.contains(entity) {
            ui_handler.stored_ids.remove(entity);
        } else {
            ui_handler.stored_ids.insert(entity.clone());
        }
    }

    // Should we change the name?
    if let Some(new_name) = result.serialize_name {
        info!("Guah!");
        let name_component = names.get_mut_or_default(entity);
        name_component.inner_mut().name = new_name;
    }

    result.show_children
}
