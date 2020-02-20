use super::{
    serialization_util, Component, ComponentDatabase, Ecs, Entity, Prefab, PrefabMap, PrefabMarker,
    ResourcesDatabase, SerializedComponent, SerializedEntity, SingletonDatabase,
};
use anyhow::{Context, Result};
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

    if let Some(post) = success {
        ecs.component_database
            .post_deserialization(post, |component_list, sl| {
                if let Some(inner) = component_list.get_mut(&entity) {
                    inner.post_deserialization(entity, sl);
                }
            });

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

/// This gets the parent prefab of a given inheritor.
/// To make this simpler, imagine Player's parent Prefab is
/// Actor. If Player's entity was passed into this method,
/// a Serialized Actor would come out.
///
/// Returns a **flag** indicating if a prefab was found,
/// which will have been loaded into the SerializedEntity provided.
pub fn get_serialized_parent_prefab_from_inheritor(
    maybe_prefab_marker: Option<&Component<PrefabMarker>>,
    resources: &ResourcesDatabase,
    serialized_entity: &mut SerializedEntity,
) -> bool {
    if let Some(prefab_component) = maybe_prefab_marker {
        let prefab = match resources.prefabs().get(&prefab_component.inner().main_id()) {
            Some(i) => i,
            None => return false,
        };

        let mut serialized_prefab = match prefab.members.get(&prefab_component.inner().sub_id()) {
            Some(sp) => sp.clone(),
            None => return false,
        };

        serialized_prefab.prefab_marker = Some(SerializedComponent {
            active: true,
            inner: prefab_component.inner().clone(),
        });

        *serialized_entity = serialized_prefab;
        true
    } else {
        false
    }
}

/// This uses the *experimental* idea of some dynamic typings in YAML. This is relatively
/// prone to crash in current form, but I'm a lazy slut!
pub fn load_override_into_prefab(
    prefab_serialized_entity: SerializedEntity,
    se_override: SerializedEntity,
) -> Result<SerializedEntity> {
    let mut prefab_serialized_yaml = serde_yaml::to_value(prefab_serialized_entity).unwrap();
    let se_override_yaml = serde_yaml::to_value(se_override).unwrap();

    let prefab_serialized_value_as_map = prefab_serialized_yaml.as_mapping_mut().unwrap();

    for (key, value) in se_override_yaml.as_mapping().unwrap().iter() {
        if *value != serde_yaml::Value::Null {
            prefab_serialized_value_as_map.insert(key.clone(), value.clone());
        }
    }

    serde_yaml::from_value(prefab_serialized_yaml)
        .with_context(|| format!("We could not transform a composed YAML SE back to SE",))
}
