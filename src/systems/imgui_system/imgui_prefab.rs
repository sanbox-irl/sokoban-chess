use super::*;

#[allow(dead_code)]
pub fn prefab_editor(
    ui_handler: &mut UiHandler<'_>,
    resources: &mut ResourcesDatabase,
    component_database: &mut ComponentDatabase,
) {
    let ui: &mut Ui<'_> = &mut ui_handler.ui;

    let mut remove_this_id = None;
    // @techdebt BORROW CHECKER NONSENSE
    let fake_prefab_map = resources.prefabs.clone();

    for prefab_id in ui_handler.stored_prefabs.iter() {
        let prefab = resources.prefabs.get_mut(prefab_id).unwrap();

        let mut open = true;
        let mut changed = false;
        let name = match &mut prefab.name {
            Some((name, _)) => im_str!("{} (Prefab)", &name.name),
            None => im_str!("{} (Prefab)", prefab_id),
        };

        let prefab_editor = imgui::Window::new(&name)
            .size(
                Vec2::new(290.0, 400.0).into(),
                imgui::Condition::FirstUseEver,
            )
            .opened(&mut open);

        if let Some(window) = prefab_editor.begin(ui) {
            let mut component_info = ComponentInfo::new(true, false);
            let stubbed_names_list = ComponentList::new();
            // let stubbed_serialization_data = ComponentList::new();
            // let mut stubbed_transforms = ComponentList::new();
            let uid = prefab.id.to_string();

            macro_rules! prefab_inspector_quick {
                ( $( [$x:ident, $y:ident] ),* ) => {
                    $(
                        match prefab_inspector(
                            &mut prefab.$x,
                            &uid,
                            &stubbed_names_list,
                            &mut component_info,
                            &fake_prefab_map,
                            ui,
                            true,
                            |me, ip| me.entity_inspector(ip),
                        ) {
                            PrefabComponentInspectorResult::NoChange => {}
                            PrefabComponentInspectorResult::Deleted => {
                                prefab.$x = None;

                                for prefab_marker in component_database.prefab_markers.iter() {
                                    if prefab_marker.inner().id == prefab.id {
                                        component_database.$y.unset(&prefab_marker.entity_id);
                                    }
                                }

                                changed = true;
                            }

                            PrefabComponentInspectorResult::Update => {
                                for prefab_marker in component_database.prefab_markers.iter() {
                                    if prefab_marker.inner().id == prefab.id {
                                        if let Some(comp) = component_database.$y.get_mut(&prefab_marker.entity_id) {
                                            *comp.inner_mut() = prefab.$x.clone().unwrap().0;
                                        };
                                    }
                                }

                                changed = true;

                            }
                        }
                    )*
                };
            }

            // @update_components
            prefab_inspector_quick!(
                [player, players],
                [transform, transforms],
                [velocity, velocities],
                [sprite, sprites],
                [sound_source, sound_sources],
                [draw_rectangle, draw_rectangles],
                [bounding_box, bounding_boxes],
                [text_source, text_sources]
            );

            // Custom Tilemap inspector! Except not, cause it's confusing
            // fuck that. So we just uh don't do this. Because what is a prefab
            // tilemap anyway?
            // prefab_serialized_inspector(
            //     &mut prefab.tilemap,
            //     "Tilemap",
            //     &uid,
            //     &stubbed_names_list,
            //     &mut component_info,
            //     &fake_prefab_map,
            //     ui,
            //     true,
            // );

            prefab_inspector_quick!([follow, follows], [conversant_npc, conversant_npcs]);

            // graph nodes
            // @techdebt We haven't implemented prefabs for the graphnode system yet!
            // match prefab_inspector(
            //     &mut prefab.graph_node,
            //     &uid,
            //     &stubbed_names_list,
            //     &mut component_info,
            //     &fake_prefab_map,
            //     ui,
            //     true,
            //     |me, ip| {
            //         me.specific_entity_inspector(
            //             Entity::
            //             ip,
            //             &stubbed_serialization_data,
            //             &mut stubbed_transforms,
            //         )
            //     },
            // ) {
            //     PrefabComponentInspectorResult::NoChange => {}
            //     PrefabComponentInspectorResult::Deleted => {
            //         prefab.graph_node = None;

            //         for prefab_marker in component_database.prefab_markers.iter() {
            //             if prefab_marker.inner().id == prefab.id {
            //                 component_database.graph_nodes.unset(&prefab_marker.entity_id);
            //             }
            //         }

            //         changed = true;
            //     }

            //     PrefabComponentInspectorResult::Update => {
            //         for prefab_marker in component_database.prefab_markers.iter() {
            //             if prefab_marker.inner().id == prefab.id {
            //                 if let Some(comp) =
            //                     component_database.graph_nodes.get_mut(&prefab_marker.entity_id)
            //                 {
            //                     *comp.inner_mut() = prefab.graph_node.clone().unwrap().0;
            //                 };
            //             }
            //         }

            //         changed = true;
            //     }
            // }

            window.end(ui);
        }

        if open == false {
            remove_this_id = Some(prefab_id.clone());
        }

        if changed {
            // Update the serialization...
            if let Err(e) = serialization_util::prefabs::serialize_prefab(prefab) {
                error!("Couldn't serialize prefab {}. Error: {}", name, e);
            }
        }
    }
    if let Some(remove_this_id) = remove_this_id {
        if let Some(position) = ui_handler
            .stored_prefabs
            .iter()
            .position(|x| *x == remove_this_id)
        {
            ui_handler.stored_prefabs.remove(position);
        }
    }

    // Create a new Prefab button goes here
}

fn prefab_inspector<
    T: ComponentBounds + typename::TypeName + Clone + PartialEq,
    F: FnMut(&mut T, InspectorParameters<'_, '_>),
>(
    prefab_wrapper: &mut Option<(T, bool)>,
    uuid: &str,
    entity_names: &ComponentList<Name>,
    component_info: &mut ComponentInfo,
    prefabs: &std::collections::HashMap<uuid::Uuid, SerializedEntity>,
    ui: &mut Ui<'_>,
    is_open: bool,
    mut inspect_function: F,
) -> PrefabComponentInspectorResult {
    let mut change = false;
    let mut delete = false;
    if let Some(prefab_wrapper) = prefab_wrapper {
        // SEPARATE
        ui.separator();

        // COMPONENT INFO
        component_info.is_deleted = false;
        component_info.is_active = prefab_wrapper.1;
        let name = imgui_utility::typed_text_ui::<T>();
        super::imgui_component::component_name_and_status(&name, ui, component_info);
        prefab_wrapper.1 = component_info.is_active;

        // DELETE ENTITY
        ui.tree_node(&imgui::ImString::new(name))
            .default_open(true)
            .build(|| {
                if component_info.is_deleted == false {
                    // Expensive paying for stuff here...
                    let duplicate = prefab_wrapper.0.clone();
                    let inspector_parameters = InspectorParameters {
                        is_open,
                        uid: &format!("{}{}", uuid, &T::type_name()),
                        ui,
                        entities: &[],
                        entity_names,
                        prefabs,
                    };
                    inspect_function(&mut prefab_wrapper.0, inspector_parameters);

                    change = prefab_wrapper.0 != duplicate;
                } else {
                    delete = true
                }
            });
    };

    if delete {
        PrefabComponentInspectorResult::Deleted
    } else {
        if change {
            PrefabComponentInspectorResult::Update
        } else {
            PrefabComponentInspectorResult::NoChange
        }
    }
}

// fn prefab_serialized_inspector<T: ComponentSerializedBounds>(
//     prefab_wrapper: &mut Option<(T, bool)>,
//     typename: &str,
//     uid: &str,
//     entity_names: &ComponentList<Name>,
//     component_info: &mut ComponentInfo,
//     prefabs: &std::collections::HashMap<uuid::Uuid, SerializedEntity>,
//     ui: &mut Ui<'_>,
//     is_open: bool,
// ) {
//     if let Some(prefab_wrapper) = prefab_wrapper {
//         // SEPARATE
//         ui.separator();

//         // COMPONENT INFO
//         component_info.is_deleted = false;
//         component_info.is_active = prefab_wrapper.1;
//         super::imgui_component::component_name_and_status(typename, ui, component_info);
//         prefab_wrapper.1 = component_info.is_active;

//         ui.tree_node(&imgui::ImString::new(typename))
//             .default_open(true)
//             .build(|| {
//                 // DELETE ENTITY
//                 let inspector_parameters = InspectorParameters {
//                     is_open,
//                     uid: &format!("{}{}", uid, typename),
//                     ui,
//                     entities: &[],
//                     entity_names,
//                     prefabs,
//                 };
//                 prefab_wrapper.0.entity_inspector(inspector_parameters);
//             });
//     }
// }

enum PrefabComponentInspectorResult {
    NoChange,
    Deleted,
    Update,
}
