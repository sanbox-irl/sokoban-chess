use super::*;

pub fn entity_list(ecs: &mut Ecs, resources: &mut ResourcesDatabase, ui_handler: &mut UiHandler<'_>) {
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
                for prefab in resources.prefabs.values_mut() {
                    let name = match &mut prefab.name {
                        Some((name, _)) => im_str!("{}##EntityInspector", &name.name),
                        None => im_str!("ID: {}##EntityInspector", prefab.id),
                    };

                    if imgui::MenuItem::new(&name).build(ui) {
                        prefabs::instantiate_prefab(prefab, ecs);
                    }
                }

                prefab_submenu.end(ui);
            }

            menu_bar.end(ui);
        }

        ui_handler.entity_vec.clear();

        // SCENE GRAPH
        scene_graph::walk_graph_inspect(
            &ecs.component_database.transforms,
            &ecs.component_database.graph_nodes,
            &mut ecs.component_database.names,
            &ecs.component_database.prefab_markers,
            &ecs.component_database.serialization_data,
            &mut |entity, names, serialization_data, name_inspector_params| {
                ui_handler.entity_vec.push(*entity);

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

        // ENTITY_GRAPH
        // for this_entity in ecs.entities.iter_mut() {
        //     display_entity_id(
        //         this_entity,
        //         &mut ecs.component_database,
        //         &ecs.ui_handler,
        //         &mut entity_to_clone,
        //         &mut entity_to_delete,
        //     );
        //     ui_handler.ui.separator();
        // }

        if let Some(original) = entity_to_clone {
            ecs.clone_entity(&original);
        }

        if let Some(entity_to_delete) = entity_to_delete {
            ecs.remove_entity(&entity_to_delete);
        }

        entity_inspector_window.end(&ui_handler.ui);
    }

    if open == false {
        ui_handler.flags.remove(ImGuiFlags::ENTITY_VIEWER);
    }
}

fn display_entity_id(
    this_entity: &Entity,
    mut name_inspector_params: NameInspectorParameters,
    names: &mut ComponentList<Name>,
    serialization_data: &ComponentList<SerializationData>,
    ui_handler: &mut UiHandler<'_>,
    clone_me: &mut Option<Entity>,
    delete_me: &mut Option<Entity>,
) -> bool {
    let mut reserialize = false;
    let mut show_children = true;

    // Name Inspector Params
    name_inspector_params.being_inspected = ui_handler.stored_ids.contains(this_entity);

    if let Some(name) = names.get_mut(this_entity) {
        let result = name.inner_mut().inspect(
            &ui_handler.ui,
            name_inspector_params,
            &this_entity.index().to_string(),
        );

        if result.reserialize {
            reserialize = true;
        }

        if result.clone {
            *clone_me = Some(*this_entity);
        }

        if result.delete {
            *delete_me = Some(*this_entity);
        }

        // Store or Remove it...
        if result.inspect {
            if ui_handler.stored_ids.contains(this_entity) {
                ui_handler.stored_ids.remove(this_entity);
            } else {
                ui_handler.stored_ids.insert(this_entity.clone());
            }
        }

        show_children = result.show_children;
    } else {
        ui_handler
            .ui
            .label_text(imgui::im_str!("Entity ID"), &im_str!("{}", this_entity));

        if imgui_utility::sized_button(&ui_handler.ui, &im_str!("Name Entity##{:?}", this_entity)) {
            let name = Name::new(&format!("Entity ID {}", this_entity.index()));
            names.set(this_entity, Component::new(this_entity, name));
            reserialize = true;
        }
    }

    if reserialize {
        // Serialize this Entity and add the name to it...
        if let Err(e) = SerializationData::edit_serialized_entity(&serialization_data, this_entity, |se| {
            se.name = names.get(this_entity).unwrap().fast_serialize()
        }) {
            error!("COULDN'T SERIALIZE NAME: {}", e);
        }
    }

    show_children
}
