use super::{
    serialization_util, Component, Ecs, Entity, PrefabMap, PrefabMarker, ResourcesDatabase,
    SerializedEntity,
};

pub fn create_blank_prefab(
    resources: &mut ResourcesDatabase,
) -> Result<uuid::Uuid, failure::Error> {
    let blank_prefab = SerializedEntity::new_blank();

    serialization_util::prefabs::serialize_prefab(&blank_prefab)?;
    let id = blank_prefab.id;
    resources.prefabs.insert(id, blank_prefab);

    Ok(id)
}

pub fn create_new_prefab_entity(
    ecs: &mut Ecs,
    prefab_id: uuid::Uuid,
    prefab_map: &PrefabMap,
) -> Entity {
    // Make an entity
    let entity = ecs.create_entity();

    // Instantiate the Prefab
    ecs.component_database
        .load_serialized_prefab(&entity, &prefab_id, prefab_map);

    // Set our Prefab Marker
    ecs.component_database.prefab_markers.set(
        &entity,
        Component::new(&entity, PrefabMarker { id: prefab_id }),
    );

    entity
}
