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
        serialization_id: Uuid,
        component_database: &ComponentDatabase,
        singleton_database: &SingletonDatabase,
    ) -> Option<Self> {
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
            serialized_entity.foreach_component_dedup(|component, active| {
                component.is_serialized(&prefab, *active)
            });
        }

        Some(serialized_entity)
    }

    pub fn foreach_component_dedup(
        &mut self,
        mut f: impl FnMut(&dyn ComponentBounds, &bool) -> bool,
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
            prefab_marker: _,
            id: _,
            marker: _,
        } = self;

        macro_rules! dedup_serialization {
            ( $( $x:ident ),* ) => {
                $(
                    if let Some((c, a)) = $x {
                        if f(c, a) {
                            *$x = None;
                        }
                    }
                )*
            };
        }

        dedup_serialization!(
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
            conversant_npc
        );
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
