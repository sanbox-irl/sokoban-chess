use super::{prefabs, ConversantNPC, Ecs, Entity, Name, ResourcesDatabase, Sprite, Vec2};
use log_once::error_once;

pub fn initialize_conv_npc_ui(ecs: &mut Ecs, resources: &mut ResourcesDatabase) {
    // We have to do this in two steps incase we end up reallocating the heap array:
    let mut vec = vec![];

    for conversant_npc_c in ecs.component_database.conversant_npcs.iter() {
        let conversant_npc: &ConversantNPC = conversant_npc_c.inner();

        // SPAWN OUR INITIAL PREFAB IF WE NEED IT:
        if let Some(prefab_ui) = &conversant_npc.initial_ui_prefab.target {
            vec.push((conversant_npc_c.entity_id.clone(), prefab_ui.clone()));
        } else {
            error_once!(
                "We didn't have a PrefabUi set up for our Initial UI Prefab for {}. It will need to be reloaded!",
                Name::get_name_quick(&ecs.component_database.names, &conversant_npc_c.entity_id)
            );
        }
    }

    for (conversant_npc_id, prefab_id) in vec {
        let new_ui = prefabs::instantiate_prefab_as_child(
            resources.prefabs.get(&prefab_id).unwrap(),
            ecs,
            conversant_npc_id,
        );

        // SET VALUES:
        ecs.component_database
            .conversant_npcs
            .get_mut(&conversant_npc_id)
            .unwrap()
            .inner_mut()
            .runtime_ui = Some(new_ui);
    }
}

pub fn update_conv_npc_ui_sprites(ecs: &mut Ecs, resources: &ResourcesDatabase, do_it: bool) {
    // We check if we want to create a prefab at the end here...
    struct ConversationChange<'a> {
        pub originator: Entity,
        pub originator_ui: Entity,
        pub new_prefab: &'a uuid::Uuid,
    }
    let mut create_prefab: Option<ConversationChange<'_>> = None;

    for conversant_npc_c in ecs.component_database.conversant_npcs.iter() {
        let our_id: Entity = conversant_npc_c.entity_id;
        let conversant_npc: &ConversantNPC = conversant_npc_c.inner();

        let conversation_partner = match &conversant_npc.conversation_partner.target {
            Some(t) => t,
            None => {
                error_once!("We don't have a conversation partner! We can't talk without a partner!");
                continue;
            }
        };

        let partner_position: Vec2 = match ecs.component_database.transforms.get(conversation_partner) {
            Some(t) => t.inner().world_position(),
            None => {
                error_once!(
                    "Partner doesn't have a transform. We cannot Converse with someone we can't find!"
                );
                continue;
            }
        };

        let our_position = match &ecs.component_database.transforms.get(&conversant_npc_c.entity_id) {
            Some(t) => t.inner().world_position(),
            None => {
                error_once!(
                    "Conversant NPC {} does not have a transform.
                    We cannot converse if we don't exist anywhere!",
                    Name::get_name_quick(&ecs.component_database.names, &conversant_npc_c.entity_id)
                );
                continue;
            }
        };

        if let Some(ui_target) = &conversant_npc.runtime_ui {
            // Sprite
            {
                let ui_target_sprite: &mut Sprite = match ecs.component_database.sprites.get_mut(&ui_target) {
                    Some(s) => s.inner_mut(),
                    None => {
                        error_once!(
                            "Target {} of Conversant {} did not have a Sprite!",
                            Name::get_name_quick(&ecs.component_database.names, &conversant_npc_c.entity_id),
                            Name::get_name_quick(&ecs.component_database.names, &ui_target)
                        );
                        continue;
                    }
                };

                // Actually update the Color of the Sprite
                let close_enough = (our_position - partner_position).magnitude() < conversant_npc.distance;

                let new_color = if close_enough {
                    // Update the Prefab itself!
                    if do_it {
                        if let Some(target) = &conversant_npc.text_ui_prefab.target {
                            // @cfg
                            if let Some(old) = &create_prefab {
                                warn!(
                                  r#"We have Two Conversant NPCS attempting to start a Conversation at once..
                                   ..{name}
                                   ..{new_name}
                                   ..Preferring {new_name}. This is arbitary, preferring later created entities."#,
                                  name = Name::get_name_quick(&ecs.component_database.names, &old.originator),
                                  new_name = Name::get_name_quick(&ecs.component_database.names, &our_id)
                                );
                            }
                            create_prefab = Some(ConversationChange {
                                originator: our_id,
                                originator_ui: *ui_target,
                                new_prefab: target,
                            });
                        } else {
                            error!(
                                "Attempted to change {} UI to Text UI Prefab, but 'text_ui_prefab.target' was None.",
                                Name::get_name_quick(&ecs.component_database.names, &our_id),
                            );
                            error!("Please Set It to a valid value in the Scene Inspector!");
                        }
                    }

                    conversant_npc.color_on_close
                } else {
                    conversant_npc.color_on_far
                };

                ui_target_sprite.running_data.tint = new_color;
            }
        }
    }

    if let Some(conversation_change) = create_prefab {
        let ConversationChange {
            originator,
            new_prefab,
            originator_ui,
        } = conversation_change;

        let new_entity = prefabs::instantiate_prefab(resources.prefabs.get(new_prefab).unwrap(), ecs);
        if ecs.remove_entity(&originator_ui) == false {
            warn!(
                "Couldn't delete {}!",
                Name::get_name_quick(&ecs.component_database.names, &originator_ui)
            );
        }

        // SET VALUES:
        ecs.component_database
            .follows
            .get_mut(&new_entity)
            .map(|f| f.inner_mut().target.target = Some(originator));

        ecs.component_database
            .conversant_npcs
            .get_mut(&originator)
            .unwrap()
            .inner_mut()
            .runtime_ui = Some(new_entity);
    }
}
