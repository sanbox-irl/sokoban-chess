use super::{component_serialization::TilemapSerialized, *};
use imgui::{Condition, ImString, MenuItem, Window};

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

        let entity_window = Window::new(&name)
            .size([600.0, 800.0], Condition::FirstUseEver)
            .position([1200.0, 100.0], Condition::FirstUseEver)
            .menu_bar(true)
            .opened(&mut is_open);

        if let Some(entity_inspector_window) = entity_window.begin(ui) {
            // Update the serialization if it's there...
            component_database
                .serialization_data
                .get_mut(entity)
                .map(|sd| sd.inner_mut().imgui_serialization());

            // This unsafety is not actually unsafe at all -- Rust doesn't yet realize
            // that this method, though it takes `component_database`, doesn't involve
            // the field .names within component_database. If we use names, then this would
            // become a lot trickier.
            let names_raw_pointer: *const ComponentList<Name> = &component_database.names;
            component_database.foreach_component_list_inspectable(&mut |component_list| {
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
            let is_prefab = component_database.prefab_markers.get(entity).is_some();

            // Serialization
            let mut serialize_it = false;
            component_database
                .serialization_data
                .component_inspector_raw(
                    entities,
                    &component_database.names,
                    entity,
                    &resources.prefabs,
                    ui,
                    is_open,
                    |inner, ip| {
                        serialize_it = inner.entity_inspector_results(ip);
                    },
                );
            if serialize_it {
                serialization_util::entities::serialize_entity_full(
                    entity,
                    component_database,
                    singleton_database,
                );
            }

            // Menu bar funtimes!
            if let Some(menu_bar) = ui.begin_menu_bar() {
                let component_add_button_text = if is_prefab {
                    "Add Override"
                } else {
                    "Add Component"
                };

                if let Some(add_component_submenu) =
                    ui.begin_menu(&ImString::new(component_add_button_text), true)
                {
                    // @update_components exception
                    let had_transform = component_database.transforms.get(entity).is_some();

                    // Prefab Marker, Name is omitted
                    component_database.foreach_component_list(
                        NonInspectableEntities::SERIALIZATION,
                        |component_list| component_list.component_add_button(entity, ui),
                    );

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

                let serialization_menu_text = if is_prefab {
                    "Serialize Overrides"
                } else {
                    "Serialization"
                };

                if let Some(serialization_submenu) = ui.begin_menu(
                    &ImString::new(serialization_menu_text),
                    component_database.serialization_data.get(entity).is_some(),
                ) {
                    if let Some(comp) = component_database.serialization_data.get(entity) {
                        match entity_serialization_options(
                            comp.inner(),
                            ui,
                            entity,
                            is_prefab,
                            component_database,
                        ) {
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

                if let Some(prefab_submenu) = ui.begin_menu(im_str!("Create Prefab"), !is_prefab) {
                    let mut prefab_to_instantiate: Option<uuid::Uuid> = None;

                    if MenuItem::new(im_str!("New Prefab")).build(ui) {
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

                            if MenuItem::new(&name).build(ui) {
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

pub fn entity_serialization_options(
    serialized_marker: &SerializationData,
    ui: &mut Ui<'_>,
    entity_id: &Entity,
    is_prefab: bool,
    component_database: &ComponentDatabase,
) -> Result<Option<ImGuiSerializationDataCommand>, failure::Error> {
    macro_rules! serialization_option_quick {
        ( $( [$x:ident, $y: ident] ),* ) => {
            $(
                serialization_option(
                    serialized_marker,
                    ui,
                    entity_id,
                    &component_database.$x,
                    is_prefab,
                    |se, c| se.$y = c.fast_serialize(),
                    |se| se.$y = None,
                )?;

            )*
        };
    }

    // @update_components
    serialization_option_quick!(
        [names, name],
        [players, player],
        [transforms, transform],
        [grid_objects, grid_object],
        [scene_switchers, scene_switcher]
    );

    serialization_option(
        serialized_marker,
        ui,
        entity_id,
        &component_database.graph_nodes,
        is_prefab,
        |se, c| {
            se.graph_node = Some({
                let mut clone: super::GraphNode = c.inner().clone();
                if let Some(children) = clone.children.as_mut() {
                    for child in children.iter_mut() {
                        child.entity_id_to_serialized_refs(&component_database.serialization_data);
                    }
                }
                ((clone, c.is_active))
            });
        },
        |se| se.follow = None,
    )?;

    serialization_option_quick!(
        [velocities, velocity],
        [sprites, sprite],
        [sound_sources, sound_source],
        [bounding_boxes, bounding_box],
        [draw_rectangles, draw_rectangle]
    );

    serialization_option(
        serialized_marker,
        ui,
        entity_id,
        &component_database.tilemaps,
        is_prefab,
        |se, c| {
            se.tilemap = {
                let o: &tilemap::Tilemap = c.inner();

                TilemapSerialized::from_tilemap(o.clone(), &se.id)
                    .map_err(|e| error!("Error Serializing Tiles in Tilemap. Warning: our data might not be saved! {}", e))
                    .ok()
                    .and_then(|ts| Some((ts, c.is_active)))
            };
        },
        |se| se.tilemap = None,
    )?;
    serialization_option(
        serialized_marker,
        ui,
        entity_id,
        &component_database.text_sources,
        is_prefab,
        |se, c| se.text_source = Some((c.inner().clone().into(), c.is_active)),
        |se| se.text_source = None,
    )?;
    serialization_option(
        serialized_marker,
        ui,
        entity_id,
        &component_database.follows,
        is_prefab,
        |se, c| {
            se.follow = Some({
                let mut clone: super::Follow = c.inner().clone();
                clone
                    .target
                    .entity_id_to_serialized_refs(&component_database.serialization_data);
                ((clone, c.is_active))
            });
        },
        |se| se.follow = None,
    )?;

    serialization_option(
        serialized_marker,
        ui,
        entity_id,
        &component_database.conversant_npcs,
        is_prefab,
        |se, c| {
            se.conversant_npc = Some({
                let mut clone: super::ConversantNPC = c.inner().clone();
                clone
                    .conversation_partner
                    .entity_id_to_serialized_refs(&component_database.serialization_data);
                ((clone, c.is_active))
            });
        },
        |se| se.conversant_npc = None,
    )?;

    ui.spacing();

    let mut sc = None;

    // REVERT SAVE
    if ui.button(im_str!("Revert"), [0.0, 0.0]) {
        match serialization_util::entities::load_entity(serialized_marker) {
            Ok(se_option) => match se_option {
                Some(serialized_entity) => {
                    sc = Some(ImGuiSerializationDataCommand::Revert(serialized_entity))
                }
                None => error!(
                    "Couldn't find a serialized entity to revert to! Are you sure this is serialized?"
                ),
            },
            Err(e) => error!("Error reading serialization file {}", e),
        }
    }

    // OVERWRITE
    ui.same_line(0.0);
    if ui.button(im_str!("Overwrite"), [0.0, 0.0]) {
        sc = Some(ImGuiSerializationDataCommand::Overwrite);
    }

    Ok(sc)
}

fn serialization_option<T: ComponentBounds + typename::TypeName + Clone, F1, F2>(
    serialized_marker: &SerializationData,
    ui: &mut Ui<'_>,
    entity_id: &Entity,
    component_list: &ComponentList<T>,
    is_prefab: bool,
    serialization_lambda: F1,
    deserialization_lambda: F2,
) -> Result<(), failure::Error>
where
    F1: Fn(&mut SerializedEntity, &Component<T>),
    F2: Fn(&mut SerializedEntity),
{
    lazy_static::lazy_static! {
        static ref DEFAULT_SERIALIZE_TEXT: ImString = ImString::new("Serialize");
        static ref DEFAULT_DESERIALIZE_TEXT: ImString = ImString::new("Deserialize");
        static ref PREFAB_SERIALIZE_TEXT: ImString = ImString::new("Serialize Override");
        static ref PREFAB_DESERIALIZE_TEXT: ImString = ImString::new("Deserialize Override");
    }

    let type_name = ImString::new(imgui_system::typed_text_ui::<T>());
    let component_exists = component_list.get(entity_id).is_some();

    if let Some(serde_menu) = ui.begin_menu(&type_name, component_exists) {
        if let Some(component) = component_list.get(entity_id) {
            // SERIALIZE
            if MenuItem::new(if is_prefab {
                &PREFAB_SERIALIZE_TEXT
            } else {
                &DEFAULT_SERIALIZE_TEXT
            })
            .build(ui)
            {
                let serialized_entity =
                    serialization_util::entities::load_entity(serialized_marker)?;

                if let Some(mut serialized_entity) = serialized_entity {
                    serialization_lambda(&mut serialized_entity, component);
                    serialization_util::entities::serialize_entity(serialized_entity)?;
                } else {
                    error!(
                        "Couldn't find a Serialized Entity for {}. Check the YAML?",
                        entity_id
                    );
                }
            }

            // DESERIALIZE
            if MenuItem::new(if is_prefab {
                &PREFAB_DESERIALIZE_TEXT
            } else {
                &DEFAULT_DESERIALIZE_TEXT
            })
            .build(ui)
            {
                let serialized_entity =
                    serialization_util::entities::load_entity(serialized_marker)?;

                if let Some(mut serialized_entity) = serialized_entity {
                    deserialization_lambda(&mut serialized_entity);
                    serialization_util::entities::serialize_entity(serialized_entity)?;
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

    Ok(())
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
