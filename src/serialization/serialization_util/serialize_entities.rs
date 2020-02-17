use super::*;
use std::collections::HashMap;
use uuid::Uuid;

pub fn path() -> (String, bool) {
    let scene: &Scene = &scene_system::CURRENT_SCENE.lock().unwrap();
    (scene.entity_path(), scene.is_prefab())
}

pub fn load_all_entities() -> Result<HashMap<Uuid, SerializedEntity>, Error> {
    let (scene_entity_path, is_prefab) = path();
    if is_prefab {
        let prefab: Prefab = load_serialized_file(&scene_entity_path)?;
        Ok(prefab.members)
    } else {
        load_serialized_file(&scene_entity_path)
    }
}

pub fn save_entity_list(entities: &HashMap<Uuid, SerializedEntity>) -> Fallible<()> {
    let (path, is_prefab) = path();
    if is_prefab {
        let mut prefab: Prefab = load_serialized_file(&path)?;
        prefab.members = entities.clone();

        save_serialized_file(&prefab, &path)
    } else {
        save_serialized_file(entities, &path)
    }
}

pub fn process_serialized_command(
    entity: &Entity,
    command: ImGuiSerializationDataCommand,
    component_database: &mut ComponentDatabase,
    singleton_database: &mut SingletonDatabase,
    entities: &mut Vec<Entity>,
    entity_allocator: &mut EntityAllocator,
    resources: &ResourcesDatabase,
) {
    match &command.serialization_type {
        ImGuiSerializationDataType::Revert => {
            // Remove the Entity
            component_database.deregister_entity(entity);

            match load_entity_by_id(&command.id) {
                Ok(Some(serialized_entity)) => {
                    // Reload the Entity
                    let post = component_database.load_serialized_entity(
                        entity,
                        serialized_entity,
                        entity_allocator,
                        entities,
                        &mut singleton_database.associated_entities,
                        resources.prefabs(),
                    );

                    if let Some(post) = post {
                        component_database.post_deserialization(post, |component_list, sl| {
                            if let Some(inner) = component_list.get_mut(entity) {
                                inner.post_deserialization(*entity, sl);
                            }
                        });
                    }
                }

                Ok(None) => {
                    error!(
                        "We couldn't find {}. Is it in the YAML?",
                        Name::get_name_quick(&component_database.names, entity)
                    );
                }

                Err(e) => error!(
                    "IO Error On Revert for {}: {}",
                    Name::get_name_quick(&component_database.names, entity),
                    e
                ),
            }
        }

        ImGuiSerializationDataType::Overwrite => {
            // SERIALIZE OVER:
            serialize_entity_full(
                entity,
                command.id,
                component_database,
                singleton_database,
                resources,
            );
        }
    }
}

pub fn serialize_all_entities(
    entities: &[Entity],
    component_database: &ComponentDatabase,
    singleton_database: &SingletonDatabase,
    resources: &ResourcesDatabase,
) -> Result<(), Error> {
    let mut serialized_entities = load_all_entities()?;

    // FIND THE OLD SERIALIZED ENTITY
    for entity in entities {
        if let Some(serialization_thing) = component_database.serialization_markers.get(entity) {
            if let Some(se) = SerializedEntity::new(
                entity,
                serialization_thing.inner().id,
                component_database,
                singleton_database,
                resources,
            ) {
                serialized_entities.insert(se.id, se);
            }
        }
    }

    save_entity_list(&serialized_entities)
}

/// This serializes an entity. It is "full" because of its parameters taken -- it serializes over the
/// entire entity, essentially creating a new Serialized Entity and then comitting that to the scene.
pub fn serialize_entity_full(
    entity_id: &Entity,
    serialized_id: uuid::Uuid,
    component_database: &ComponentDatabase,
    singleton_database: &SingletonDatabase,
    resources: &ResourcesDatabase,
) -> bool {
    if let Some(se) = SerializedEntity::new(
        entity_id,
        serialized_id,
        component_database,
        singleton_database,
        resources,
    ) {
        match commit_entity_to_scene(se) {
            Ok(()) => true,
            Err(e) => {
                error!("COULDN'T SERIALIZE! {}", e);
                false
            }
        }
    } else {
        false
    }
}

// @techdebt Use it or lose it!
pub fn unserialize_entity(serialized_id: &uuid::Uuid) -> Result<bool, Error> {
    let mut entities = load_all_entities()?;

    // FIND THE OLD PREFAB
    let succeeded = entities.remove(serialized_id).is_some();
    save_entity_list(&entities)?;

    Ok(succeeded)
}

pub fn commit_entity_to_scene(serialized_entity: SerializedEntity) -> Result<(), Error> {
    let mut entities = load_all_entities()?;
    entities.insert(serialized_entity.id, serialized_entity);

    save_entity_list(&entities)
}

pub fn load_committed_entity(
    serialized_data: &SerializationMarker,
) -> Result<Option<SerializedEntity>, Error> {
    load_entity_by_id(&serialized_data.id)
}

pub fn load_entity_by_id(id: &uuid::Uuid) -> Result<Option<SerializedEntity>, Error> {
    let mut entities: HashMap<Uuid, SerializedEntity> = load_all_entities()?;
    Ok(entities.remove(id))
}
