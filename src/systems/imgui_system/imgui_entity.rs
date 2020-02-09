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
    let mut entity_to_console_dump = None;

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

            if imgui::MenuItem::new(im_str!("Serialize Scene")).build(ui) {
                match serialization_util::entities::serialize_all_entities(
                    &ecs.entities,
                    &ecs.component_database,
                    &ecs.singleton_database,
                ) {
                    Ok(()) => info!("Serialized Scene"),
                    Err(e) => {
                        error!("Error on Serialization: {}", e);
                    }
                }
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
            &mut ecs.component_database.serialization_data,
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
                    &mut entity_to_console_dump,
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
                    depth: 0,
                    has_children: false,
                    is_serialized: ecs
                        .component_database
                        .serialization_data
                        .get(entity)
                        .is_some(),
                };

                display_entity_id(
                    entity,
                    nip,
                    &mut ecs.component_database.names,
                    &mut ecs.component_database.serialization_data,
                    ui_handler,
                    &mut entity_to_clone,
                    &mut entity_to_delete,
                    &mut entity_to_console_dump,
                );
            }
        }

        if let Some(original) = entity_to_clone {
            let new_entity = ecs.clone_entity(&original);

            let names: *const ComponentList<Name> = &mut ecs.component_database.names;
            if let Some(name) = ecs.component_database.names.get_mut(&new_entity) {
                name.inner_mut().update_name(new_entity, unsafe { &*names });
            }
        }

        if let Some(entity_to_delete) = entity_to_delete {
            ecs.remove_entity(&entity_to_delete);
        }

        if let Some(console_dump_me) = entity_to_console_dump {
            println!("---Console Dump for {}---", console_dump_me);
            ecs.component_database.foreach_component_list_mut(NonInspectableEntities::all(), |comp_list| {
                comp_list.dump_to_log(&console_dump_me);
            });
            println!("-------------------------");
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
    serialization_data: &mut ComponentList<SerializationMarker>,
    ui_handler: &mut UiHandler<'_>,
    clone_me: &mut Option<Entity>,
    delete_me: &mut Option<Entity>,
    console_dump_me: &mut Option<Entity>,
) -> bool {
    // Name Inspector Params
    name_inspector_params.being_inspected = ui_handler.stored_ids.contains(entity);

    // Find our ImGui entry list info
    let entity_list_info = match ui_handler.entity_list_information.get_mut(entity) {
        Some(stuff) => stuff,
        None => {
            // for none stuff
            ui_handler
                .entity_list_information
                .insert(*entity, EntityListInformation::default());
            ui_handler.entity_list_information.get_mut(entity).unwrap()
        }
    };

    let NameInspectorResult {
        clone,
        delete,
        dump_into_console_log,
        inspect,
        serialize_name,
        show_children,
        unserialize,
    } = Name::inspect(
        names
            .get_mut(entity)
            .map_or(&format!("ID {}", entity), |name| &name.inner_mut().name),
        entity_list_info,
        name_inspector_params,
        &ui_handler.ui,
        &entity.index().to_string(),
    );

    if unserialize {
        let can_unserialize = if let Some(serialization_data) = serialization_data.get(entity) {
            match serialization_util::entities::unserialize_entity(&serialization_data.inner().id) {
                Ok(success) => success,
                Err(e) => {
                    error!("Couldn't unserialize! {}", e);
                    false
                }
            }
        } else {
            false
        };

        if can_unserialize {
            serialization_data.unset(entity);
        } else {
            error!("Couldn't unserialize!");
        }
    }

    if clone {
        *clone_me = Some(*entity);
    }

    if delete {
        *delete_me = Some(*entity);
        if ui_handler.stored_ids.contains(entity) {
            ui_handler.stored_ids.remove(entity);
        }
    }

    if dump_into_console_log {
        *console_dump_me = Some(*entity);
    }

    // Store or Remove it...
    if inspect {
        if ui_handler.stored_ids.contains(entity) {
            ui_handler.stored_ids.remove(entity);
        } else {
            ui_handler.stored_ids.insert(entity.clone());
        }
    }

    // Should we change the name?
    if let Some(new_name) = serialize_name {
        let name_component = names.get_mut_or_default(entity);
        name_component.inner_mut().name = new_name;
    }

    show_children
}
