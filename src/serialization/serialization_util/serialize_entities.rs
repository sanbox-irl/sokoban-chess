use super::*;
pub const ENTITY_SUBPATH: &str = "entities_data.yaml";
use std::collections::HashMap;
use uuid::Uuid;

pub fn path() -> String {
    format!(
        "{}/{}/{}",
        SCENE_DIRECTORY,
        scene_system::CURRENT_SCENE.lock().unwrap(),
        ENTITY_SUBPATH
    )
}

pub fn load_all_entities() -> Result<HashMap<Uuid, SerializedEntity>, Error> {
    let scene_entity_path = path();
    load_serialized_file(&scene_entity_path)
}

pub fn process_serialized_command(
    entity: &Entity,
    command: ImGuiSerializationDataCommand,
    component_database: &mut ComponentDatabase,
    singleton_database: &mut SingletonDatabase,
    prefab_map: &PrefabMap,
) {
    match &command.serialization_type {
        ImGuiSerializationDataType::Revert => {
            // Remove the Entity
            component_database.deregister_entity(entity);

            match load_entity_by_id(&command.id) {
                Ok(Some(serialized_entity)) => {
                    // Reload the Entity
                    component_database.load_serialized_entity(
                        entity,
                        serialized_entity,
                        &mut singleton_database.associated_entities,
                        prefab_map,
                    );
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
            serialize_entity_full(entity, command.id, component_database, singleton_database);
        }
    }
}

pub fn serialize_all_entities(
    entities: &[Entity],
    component_database: &ComponentDatabase,
    singleton_database: &SingletonDatabase,
) -> Result<(), Error> {
    let path = path();

    let mut serialized_entities: HashMap<Uuid, SerializedEntity> = load_serialized_file(&path)?;

    // FIND THE OLD SERIALIZED ENTITY
    for entity in entities {
        if let Some(serialization_thing) = component_database.serialization_data.get(entity) {
            if let Some(se) = SerializedEntity::new(
                entity,
                serialization_thing.inner().id,
                component_database,
                singleton_database,
            ) {
                serialized_entities.insert(se.id, se);
            }
        }
    }

    save_serialized_file(&serialized_entities, &path)
}

/// This serializes an entity. It is "full" because of its parameters taken -- it serializes over the
/// entire entity, essentially creating a new Serialized Entity and then comitting that to the scene.
pub fn serialize_entity_full(
    entity_id: &Entity,
    serialized_id: uuid::Uuid,
    component_database: &ComponentDatabase,
    singleton_database: &SingletonDatabase,
) -> bool {
    if let Some(se) = SerializedEntity::new(
        entity_id,
        serialized_id,
        component_database,
        singleton_database,
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
    let path = path();

    let mut entities: Vec<SerializedEntity> = load_serialized_file(&path)?;

    // FIND THE OLD PREFAB
    let ret;
    let old_pos = entities.iter().position(|x| x.id == *serialized_id);
    if let Some(old_pos) = old_pos {
        entities.remove(old_pos);
        ret = true;
    } else {
        ret = false;
    }

    save_serialized_file(&entities, &path)?;
    Ok(ret)
}

pub fn commit_entity_to_scene(serialized_entity: SerializedEntity) -> Result<(), Error> {
    let path = path();

    let mut entities: HashMap<Uuid, SerializedEntity> = load_serialized_file(&path)?;
    entities.insert(serialized_entity.id, serialized_entity);

    save_serialized_file(&entities, &path)
}

pub fn load_committed_entity(
    serialized_data: &SerializationMarker,
) -> Result<Option<SerializedEntity>, Error> {
    load_entity_by_id(&serialized_data.id)
}

pub fn load_entity_by_id(id: &uuid::Uuid) -> Result<Option<SerializedEntity>, Error> {
    // ENTITIES
    let mut entities: HashMap<Uuid, SerializedEntity> = load_serialized_file(&path())?;

    // FIND THE OLD SERIALIZED ENTITY
    Ok(entities.remove(id))
}
