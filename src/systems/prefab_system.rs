use super::{
    serialization_util, ComponentDatabase, Ecs, Entity, Prefab, PrefabMap, PrefabMarker,
    ResourcesDatabase, SerializedEntity, SingletonDatabase,
};
use failure::Fallible;
use uuid::Uuid;

pub fn commit_blank_prefab(resources: &mut ResourcesDatabase) -> Fallible<uuid::Uuid> {
    let blank_prefab = Prefab::new_blank();

    serialization_util::prefabs::serialize_prefab(&blank_prefab)?;
    let id = blank_prefab.root_id();
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
    let success = ecs.component_database.load_serialized_prefab(
        &entity,
        &prefab_id,
        &mut ecs.entity_allocator,
        &mut ecs.entities,
        prefab_map,
        &mut ecs.singleton_database.associated_entities,
    );

    if success {
        // Set our Prefab Marker
        ecs.component_database
            .prefab_markers
            .set_component(&entity, PrefabMarker::new_main(prefab_id));
    } else {
        if ecs.remove_entity(&entity) == false {
            error!("We couldn't remove the Entity either, so we have a dangler!");
        }
    }

    entity
}

pub fn load_entity_into_prefab(
    entity: &Entity,
    prefab_id: Uuid,
    component_database: &mut ComponentDatabase,
    singleton_database: &SingletonDatabase,
    resources: &mut ResourcesDatabase,
) {
    // Create a serialized entity
    if let Some(serialized_entity) = SerializedEntity::new(
        entity,
        prefab_id,
        component_database,
        singleton_database,
        resources,
    ) {
        let prefab = Prefab::new(serialized_entity);

        if let Err(e) = serialization_util::prefabs::serialize_prefab(&prefab) {
            error!("Error Creating Prefab: {}", e);
        }

        match serialization_util::prefabs::cycle_prefab(prefab) {
            Ok(prefab) => {
                resources.add_prefab(prefab);
            }
            Err(e) => error!("We couldn't cycle the Prefab! It wasn't saved! {}", e),
        }

        // Add our Prefab Marker back to the Original entity we made into a prefab...
        component_database
            .prefab_markers
            .set_component(entity, PrefabMarker::new_main(prefab_id));

        // And if it's serialized, let's cycle our Serialization too!
        // We do this to remove the "Overrides" that would otherwise appear
        if let Some(sc) = component_database.serialization_markers.get(entity) {
            serialization_util::entities::serialize_entity_full(
                entity,
                sc.inner().id,
                component_database,
                singleton_database,
                resources,
            );
        }
    }
}
