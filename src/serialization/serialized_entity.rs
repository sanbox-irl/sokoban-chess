use super::{
    component_serialization::*, physics_components::*, Component, ComponentBounds,
    ComponentDatabase, ConversantNPC, DrawRectangle, Entity, Follow, GraphNode, Marker, Name,
    Player, PrefabMarker, SingletonDatabase, SoundSource, Sprite, TextSource, Transform, Velocity,
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

        // Prefab Serialization! Here, we only serialize the Prefab reference, and then
        // on deserialization, we'll read the prefab file!
        if let Some(prefab_component) = component_database.prefab_markers.get(entity_id) {
            let mut prefab_serialization = SerializedEntity::default();
            prefab_serialization.id = serialization_id;
            prefab_serialization.prefab_marker = Some((prefab_component.inner().clone(), true));

            return prefab_serialization;
        }

        SerializedEntity {
                // @update_components
                name: Self::clone_component(component_database.names.get(entity_id)),
                player: Self::clone_component(component_database.players.get(entity_id)),
                transform: Self::clone_component(component_database.transforms.get(entity_id)),
                graph_node: Self::clone_component(component_database.graph_nodes.get(entity_id)).map(|(mut gn, is_active)| {
                    if let Some(children) = &mut gn.children {
                        for child in children {
                            child.serialize(&component_database.serialization_data);
                        }
                    }
                    (gn, is_active)
                }),
                velocity: Self::clone_component(component_database.velocities.get(entity_id)),
                sprite: Self::clone_component(component_database.sprites.get(entity_id))
                    .map(|spr| (spr.0.into(), spr.1)),
                draw_rectangle: Self::clone_component(component_database.draw_rectangles.get(entity_id)),
                sound_source: Self::clone_component(component_database.sound_sources.get(entity_id)),
                bounding_box: Self::clone_component(component_database.bounding_boxes.get(entity_id)),
                tilemap: Self::clone_component(component_database.tilemaps.get(entity_id)).and_then(
                    |(tilemap, is_active)| {
                        TilemapSerialized::from_tilemap(tilemap, &serialization_id)
                            .map_err(|e| {
                                error!(
                                "Error Serializing Tiles in Tilemap. Warning: our data might not be saved! {}",
                                e
                            )
                            })
                            .ok()
                            .map(|tmap_s| (tmap_s, is_active))
                    },
                ),
                text_source: Self::clone_component(component_database.text_sources.get(entity_id)),
                marker: singleton_database.save_singleton_markers(entity_id),
                follow: Self::clone_component(component_database.follows.get(entity_id)).map(
                    |(mut af, is_active)| {
                        af.target.serialize(&component_database.serialization_data);
                        (af, is_active)
                    },
                ),
                conversant_npc: Self::clone_component(component_database.conversant_npcs.get(entity_id)).map(
                    |(mut cn, is_active)| {
                        cn.conversation_partner
                            .serialize(&component_database.serialization_data);
                        (cn, is_active)
                    },
                ),
                id: serialization_id,
                prefab_marker: Self::clone_component(component_database.prefab_markers.get(entity_id)),
            }
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
