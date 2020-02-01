use super::{
    serialization_util, Component, ComponentDatabase, Ecs, Entity, PrefabMarker, ResourcesDatabase,
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

pub fn instantiate_prefab(prefab: &SerializedEntity, ecs: &mut Ecs) -> Entity {
    // Make an entity
    let entity = ecs.create_entity();
    update_prefab_marker(prefab, &mut ecs.component_database, &entity);

    entity
}

pub fn instantiate_prefab_as_child(
    prefab: &SerializedEntity,
    ecs: &mut Ecs,
    parent_id: Entity,
) -> Entity {
    let entity = instantiate_prefab(prefab, ecs);

    let our_graph_node = ecs
        .component_database
        .graph_nodes
        .get_mut_or_default(&parent_id);
    if let Some(their_transform) = ecs.component_database.transforms.get_mut(&entity) {
        our_graph_node.inner_mut().add_child_directly(
            Some(parent_id),
            their_transform,
            &ecs.component_database.serialization_data,
        );
    }

    entity
}

pub fn update_prefab_marker(
    prefab: &SerializedEntity,
    component_database: &mut ComponentDatabase,
    entity: &Entity,
) {
    // RELOAD ENTITIES
    component_database.load_prefab(&entity, prefab.clone());

    // SET OUR SERIALIZATION MARKER
    component_database.prefab_markers.set(
        &entity,
        Component::new(
            &entity,
            PrefabMarker {
                id: prefab.id.clone(),
            },
        ),
    );
}
