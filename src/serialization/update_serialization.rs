// use super::{
//     component_serialization::*, physics_components::*, serialization_util, ConversantNPC, DrawRectangle,
//     Follow, Marker, Name, SerializationData, SerializedComponentWrapper, SerializedEntity, SoundSource,
//     TextSource, Transform, Velocity,
// };

pub const UPDATE_COMPONENT_DATABASE: bool = false;

// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
// #[serde(default)]
// pub struct Old {
//     pub serialization_data: SerializationData,
//     pub marker: Option<Marker>,
//     pub name: SerializedComponentWrapper<Name>,
//     pub transform: SerializedComponentWrapper<Transform>,
//     pub velocity: SerializedComponentWrapper<Velocity>,
//     pub sprite: SerializedComponentWrapper<SpriteSerialized>,
//     pub sound_source: SerializedComponentWrapper<SoundSource>,
//     pub draw_rectangle: SerializedComponentWrapper<DrawRectangle>,
//     pub bounding_box: SerializedComponentWrapper<BoundingBox>,
//     pub text_source: SerializedComponentWrapper<TextSource>,
//     pub tilemap: SerializedComponentWrapper<TilemapSerialized>,
//     pub follow: SerializedComponentWrapper<Follow>,
//     pub conversant_npc: SerializedComponentWrapper<ConversantNPC>,
// }

// impl From<Old> for SerializedEntity {
//     fn from(o: Old) -> SerializedEntity {
//         SerializedEntity {
//             id: o.serialization_data.id,
//             marker: o.marker,
//             name: o.name,
//             transform: o.transform,
//             velocity: o.velocity,
//             sprite: o.sprite,
//             sound_source: o.sound_source,
//             draw_rectangle: o.draw_rectangle,
//             bounding_box: o.bounding_box,
//             text_source: o.text_source,
//             tilemap: o.tilemap,
//             follow: o.follow,
//             conversant_npc: o.conversant_npc,
//         }
//     }
// }

pub fn update_component_database() -> Result<(), failure::Error> {
    // let scene_entity_path = serialization_util::entities::path();
    // let saved_entities: Vec<SerializedEntity> = serialization_util::load_serialized_file(&scene_entity_path)?;

    // let new_entities: Vec<SerializedEntity> = saved_entities.into_iter().map(|o| o.into()).collect();
    // serialization_util::save_serialized_file(&new_entities, &scene_entity_path)?;

    Ok(())
}
