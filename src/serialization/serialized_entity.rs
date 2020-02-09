use super::{
    component_serialization::*, physics_components::*, Component, ComponentBounds,
    ComponentDatabase, ConversantNPC, DrawRectangle, Entity, Follow, GraphNode, GridObject, Marker,
    Name, NonInspectableEntities, Player, PrefabMarker, SceneSwitcher, SingletonDatabase,
    SoundSource, Sprite, TextSource, Transform, Velocity,
};
use uuid::Uuid;

pub type SerializedComponentWrapper<T> = Option<(T, bool)>;

// This should mirror ComponentDatabse
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct SerializedEntity {
    pub id: Uuid,
    pub marker: Option<Marker>,
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
}

impl SerializedEntity {
    pub fn new(
        entity_id: &Entity,
        component_database: &ComponentDatabase,
        singleton_database: &SingletonDatabase,
    ) -> Self {
        let serialization_id = component_database
            .serialization_data
            .get(entity_id)
            .map(|c| c.inner().clone())
            .unwrap_or_default()
            .id;

        let mut serialized_entity = SerializedEntity::default();

        if let Some(prefab_component) = component_database.prefab_markers.get(entity_id) {
            serialized_entity.id = serialization_id;
            serialized_entity.prefab_marker = Some((prefab_component.inner().clone(), true));
        } else {
            component_database.foreach_component_list(
                NonInspectableEntities::NAME,
                |component_list| {
                    component_list.create_serialized_entity(
                        entity_id,
                        &mut serialized_entity,
                        &component_database.serialization_data,
                    );
                },
            );

            serialized_entity.marker = singleton_database.save_singleton_markers(entity_id);
        }

        serialized_entity
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
