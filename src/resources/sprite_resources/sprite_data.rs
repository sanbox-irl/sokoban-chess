use super::{
    cardinals::{FacingHorizontal, FacingVertical},
    Origin, SpriteInGameData, SpriteName, TextureInformation, Vec2, Vec2Int,
};
use clockwork_build_shared::sprite_packing::shared::SpriteResource;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SpriteData {
    pub sprite_name: SpriteName,
    pub texture_page: Option<usize>,
    pub origin: Origin,
    pub facing_horizontal: FacingHorizontal,
    pub facing_vertical: FacingVertical,

    pub normalized_dimensions: Vec2,
    pub size: Vec2Int,
    pub frames: Vec<FrameData>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FrameData {
    pub normalized_coord: Vec2,
    pub duration: Option<f32>,
}

impl SpriteData {
    pub fn from_sprite_resource(
        sprite_resource: SpriteResource,
        sprite_meta_data: SpriteInGameData,
        sprite_name: SpriteName,
        texture_information: TextureInformation,
    ) -> SpriteData {
        let mut frames = Vec::with_capacity(sprite_resource.frames.len());
        let size = Vec2Int::new(
            sprite_resource.frames[0].width as i32,
            sprite_resource.frames[0].height as i32,
        );

        // Iterate over our durations and our SpriteSheet data
        for (i, this_frame) in sprite_resource.frames.iter().enumerate() {
            let duration = if sprite_meta_data.frame_durations.len() < i {
                sprite_meta_data.frame_durations[i]
            } else {
                Some(0.1)
            };

            frames.push(FrameData {
                normalized_coord: Vec2::new(
                    this_frame.x as f32 / texture_information.dimensions.x,
                    this_frame.y as f32 / texture_information.dimensions.y,
                ),
                duration,
            });

            assert_eq!(this_frame.width as i32, size.x);
            assert_eq!(this_frame.height as i32, size.y);
        }

        SpriteData {
            sprite_name,
            texture_page: Some(texture_information.page),
            origin: sprite_meta_data.origin,
            normalized_dimensions: Vec2::from(size).cwise_div(texture_information.dimensions),
            size,
            frames,
            facing_horizontal: sprite_meta_data.facing_horizontal,
            facing_vertical: sprite_meta_data.facing_vertical,
        }
    }
}
