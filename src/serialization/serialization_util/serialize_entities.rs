use super::*;
const ENTITY_SUBPATH: &str = "entities_data.yaml";

pub fn path() -> String {
    format!(
        "{}/{}/{}",
        SCENE_DIRECTORY,
        CURRENT_SCENE.lock().unwrap(),
        ENTITY_SUBPATH
    )
}

pub fn load_all_entities() -> Result<Vec<SerializedEntity>, Error> {
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
    match command {
        ImGuiSerializationDataCommand::Revert(serialized_entity) => {
            // REMOVE FROM GAME
            component_database.deregister_entity(entity, true);

            // RELOAD
            component_database.load_serialized_entity(
                entity,
                serialized_entity,
                &mut singleton_database.associated_entities,
                prefab_map,
            );
        }
        ImGuiSerializationDataCommand::Overwrite => {
            // SERIALIZE OVER:
            serialize_entity_full(entity, component_database, singleton_database);
        }
    }
}

pub fn serialize_entity_full(
    entity_id: &Entity,
    component_database: &ComponentDatabase,
    singleton_database: &SingletonDatabase,
) -> bool {
    let se = SerializedEntity::new(entity_id, component_database, singleton_database);

    match serialize_entity(se) {
        Ok(()) => true,
        Err(e) => {
            error!("COULDN'T SERIALIZE! {}", e);
            false
        }
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

pub fn serialize_entity(serialized_entity: SerializedEntity) -> Result<(), Error> {
    let path = path();

    let mut entities: Vec<SerializedEntity> = load_serialized_file(&path)?;

    // FIND THE OLD SERIALIZED ENTITY
    let old_pos = entities.iter().position(|x| x.id == serialized_entity.id);
    if let Some(old_pos) = old_pos {
        entities[old_pos] = serialized_entity;
    } else {
        entities.push(serialized_entity);
    }

    save_serialized_file(&entities, &path)
}

pub fn load_entity(serialized_data: &SerializationData) -> Result<Option<SerializedEntity>, Error> {
    // ENTITIES
    let entities: Vec<SerializedEntity> = load_serialized_file(&path())?;

    // FIND THE OLD SERIALIZED ENTITY
    let old_pos = entities.iter().position(|x| x.id == serialized_data.id);

    if let Some(old_pos) = old_pos {
        Ok(Some(entities[old_pos].clone()))
    } else {
        Ok(None)
    }
}
