use super::{
    component_serialization::*, physics_components::*, Component, ComponentBounds,
    ComponentDatabase, ConversantNPC, DrawRectangle, Entity, Follow, GraphNode, GridObject, Marker,
    Name, NonInspectableEntities, Player, PrefabMarker, ResourcesDatabase, SceneSwitcher,
    SingletonDatabase, SoundSource, Sprite, TextSource, Transform, Velocity,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct SerializedComponent<T> {
    pub inner: T,
    pub active: bool,
}

pub type SerializedComponentWrapper<T> = Option<SerializedComponent<T>>;

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
        resources: &ResourcesDatabase,
    ) -> Option<Self> {
        let mut serialized_entity = SerializedEntity::default();

        // If it's a prefab, add in all the PREFAB components
        let prefab =
            if let Some(prefab_component) = component_database.prefab_markers.get(entity_id) {
                let prefab = resources
                    .prefabs()
                    .get(&prefab_component.inner().main_id())?;

                let mut serialized_prefab = prefab
                    .members
                    .get(&prefab_component.inner().sub_id())?
                    .clone();

                serialized_prefab.prefab_marker = Some(SerializedComponent {
                    active: true,
                    inner: prefab_component.inner().clone(),
                });

                serialized_entity = serialized_prefab.clone();
                Some(serialized_prefab)
            } else {
                None
            };

        // Save over the ID at this stage (we'll be copying over the prefab ID)
        serialized_entity.id = serialization_id;

        // Now add in all the normal components:
        component_database.foreach_component_list(
            NonInspectableEntities::NAME | NonInspectableEntities::GRAPH_NODE,
            |component_list| {
                component_list.create_serialized_entity(
                    entity_id,
                    &mut serialized_entity,
                    &component_database.serialization_data,
                );
            },
        );
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

    pub fn foreach_component(
        &mut self,
        entity_bitmask: NonInspectableEntities,
        mut f: impl FnMut(&dyn ComponentBounds, &bool),
        f_util: Option<impl FnMut(&mut Uuid, &mut Option<Marker>)>,
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
            id,
            marker,
        } = self;

        macro_rules! do_action {
            ( $( $x:ident ),* ) => {
                $(
                    if let Some(serialized_component) = $x {
                        f(&serialized_component.inner, &serialized_component.active);
                    }
                )*
            };
        }

        if entity_bitmask.contains(NonInspectableEntities::NAME) {
            do_action!(name);
        }

        do_action!(
            player,
            transform,
            grid_object,
            scene_switcher,
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

        if entity_bitmask.contains(NonInspectableEntities::GRAPH_NODE) {
            do_action!(graph_node);
        }

        if entity_bitmask.contains(NonInspectableEntities::PREFAB) {
            do_action!(prefab_marker);
        }

        if let Some(mut f_util) = f_util {
            f_util(id, marker);
        }
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
                    if let Some(serialized_component) = $x {
                        if f(&serialized_component.inner, &serialized_component.active) {
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
        if let Some(comp) = comp {
            Some(SerializedComponent {
                active: comp.is_active,
                inner: comp.clone_inner(),
            })
        } else {
            None
        }
    }
}
