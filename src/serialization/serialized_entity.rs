use super::{
    component_serialization::*, physics_components::*, serialization_util, Component,
    ComponentBounds, ComponentDatabase, ConversantNPC, DrawRectangle, Entity, Follow, GraphNode,
    GridObject, Marker, Name, NonInspectableEntities, Player, PrefabMarker, SceneSwitcher,
    SingletonDatabase, SoundSource, Sprite, TextSource, Transform, Velocity,
};
use uuid::Uuid;

pub type SerializedComponentWrapper<T> = Option<(T, bool)>;

// This should mirror ComponentDatabse
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct SerializedEntity {
    pub name: SerializedComponentWrapper<Name>,
    pub player: SerializedComponentWrapper<Player>,
    pub transform: SerializedComponentWrapper<Transform>,
    pub grid_object: SerializedComponentWrapper<GridObject>,
    pub scene_switcher: SerializedComponentWrapper<SceneSwitcher>,
    pub graph_node: SerializedComponentWrapper<GraphNode>,
    pub velocity: SerializedComponentWrapper<Velocity>,
    pub sprite: SerializedComponentWrapper<Sprite>,
    pub sound_source: SerializedComponentWrapper<SoundSource>,
    pub draw_rectangle: SerializedComponentWrapper<DrawRectangle>,
    pub bounding_box: SerializedComponentWrapper<BoundingBox>,
    pub text_source: SerializedComponentWrapper<TextSource>,
    pub tilemap: SerializedComponentWrapper<TilemapSerialized>,
    pub follow: SerializedComponentWrapper<Follow>,
    pub conversant_npc: SerializedComponentWrapper<ConversantNPC>,
    pub prefab_marker: SerializedComponentWrapper<PrefabMarker>,

    pub id: Uuid,
    pub marker: Option<Marker>,
}

impl SerializedEntity {
    pub fn new(
        entity_id: &Entity,
        component_database: &ComponentDatabase,
        singleton_database: &SingletonDatabase,
    ) -> Option<Self> {
        let serialization_id = component_database
            .serialization_data
            .get(entity_id)
            .map(|c| c.inner().clone())
            .unwrap_or_default()
            .id;

        let mut serialized_entity = SerializedEntity::default();

        // If it's a prefab, add in all the PREFAB components
        let prefab = if let Some(prefab_component) =
            component_database.prefab_markers.get(entity_id)
        {
            let mut prefab = serialization_util::prefabs::load_prefab(&prefab_component.inner().id)
                .map_err(|e| error!("Error On Loading Prefab: {}", e))
                .ok()??;

            prefab.prefab_marker = Some((prefab_component.inner().clone(), true));
            serialized_entity = prefab.clone();
            Some(prefab)
        } else {
            None
        };

        // Save over the ID at this stage (we'll be copying over the prefab ID)
        serialized_entity.id = serialization_id;

        // Now add in all the normal components:
        component_database.foreach_component_list(NonInspectableEntities::NAME, |component_list| {
            component_list.create_serialized_entity(
                entity_id,
                &mut serialized_entity,
                &component_database.serialization_data,
            );
        });
        serialized_entity.marker = singleton_database.save_singleton_markers(entity_id);

        // Now, load the prefab again, and for every field where the prefab matches with
        // the the serialized entity, strip that field away from the serialized entity.
        // Therefore, when we load, we will *only* have the prefab to load from.
        if let Some(prefab) = prefab {
            serialized_entity.foreach_component(
                NonInspectableEntities::all(),
                |component, active| {
                    if component.is_serialized(&prefab, *active) {
                        component.uncommit_to_scene(&mut unimplemented!());
                    }
                },
            );
        }

        Some(serialized_entity)
    }

    // @update_components
    pub fn foreach_component(
        &self,
        non_inspectable_entities: NonInspectableEntities,
        mut f: impl FnMut(&dyn ComponentBounds, &bool),
    ) {
        let SerializedEntity {
            name,
            player,
            transform,
            grid_object,
            scene_switcher,
            graph_node,
            velocity,
            sprite,
            sound_source,
            draw_rectangle,
            bounding_box,
            text_source,
            tilemap,
            follow,
            conversant_npc,
            prefab_marker,
            id: _,
            marker: _,
        } = self;

        name.as_ref().map(|(name, a)| f(name, a));
        player.as_ref().map(|(c, a)| f(c, a));
        transform.as_ref().map(|(c, a)| f(c, a));
        grid_object.as_ref().map(|(c, a)| f(c, a));
        scene_switcher.as_ref().map(|(c, a)| f(c, a));
        graph_node.as_ref().map(|(c, a)| f(c, a));
        velocity.as_ref().map(|(c, a)| f(c, a));
        sprite.as_ref().map(|(c, a)| f(c, a));
        sound_source.as_ref().map(|(c, a)| f(c, a));
        draw_rectangle.as_ref().map(|(c, a)| f(c, a));
        bounding_box.as_ref().map(|(c, a)| f(c, a));
        text_source.as_ref().map(|(c, a)| f(c, a));
        tilemap.as_ref().map(|(c, a)| f(c, a));
        follow.as_ref().map(|(c, a)| f(c, a));
        conversant_npc.as_ref().map(|(c, a)| f(c, a));
        prefab_marker.as_ref().map(|(c, a)| f(c, a));
    }

    pub fn new_blank() -> Self {
        SerializedEntity {
            id: Uuid::new_v4(),
            ..Default::default()
        }
    }

    pub fn clone_component<T: ComponentBounds + Clone>(
        comp: Option<&Component<T>>,
    ) -> SerializedComponentWrapper<T> {
        if let Some(inner) = comp {
            Some((inner.inner().clone(), inner.is_active))
        } else {
            None
        }
    }
}
