use super::{serialization_util, Ecs, Entity, Prefab, PrefabMap, PrefabMarker, ResourcesDatabase};
use failure::Fallible;

pub fn commit_blank_prefab(resources: &mut ResourcesDatabase) -> Fallible<uuid::Uuid> {
    let blank_prefab = Prefab::new_blank();

    serialization_util::prefabs::serialize_prefab(&blank_prefab)?;
    let id = blank_prefab.main_id();
    resources.add_prefab(blank_prefab);
    Ok(id)
}

pub fn instantiate_entity_from_prefab(
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
    ecs.component_database
        .prefab_markers
        .set_component(&entity, PrefabMarker::new_main(prefab_id));

    entity
}
