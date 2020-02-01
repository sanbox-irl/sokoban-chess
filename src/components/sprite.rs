use super::{
    cardinals::{FacingHorizontal, FacingVertical},
    component_utils::SpriteRunningData,
    imgui_system, number_util,
    sprite_resources::*,
    Axis, Color, ComponentBounds, DrawOrder, InspectorParameters, ResourcesDatabase, StandardQuad,
    StandardQuadFactory, StandardTexture, TextureDescription, Vec2,
};

#[derive(Debug, Clone, PartialEq, typename::TypeName)]
pub struct Sprite {
    pub sprite_data: Option<SpriteData>,
    pub new_sprite: Option<SpriteName>,
    pub running_data: SpriteRunningData,
}

impl Sprite {
    // GETTERS AND SETTERS!
    pub fn current_frame(&self) -> Option<&FrameData> {
        if let Some(sprite_data) = &self.sprite_data {
            Some(&sprite_data.frames[self.running_data.current_frame])
        } else {
            None
        }
    }

    pub fn ensure_sprite(&mut self, name: SpriteName) {
        if self.sprite_name() != Some(name) {
            self.set_new_sprite(name);
        }
    }

    pub fn set_new_sprite(&mut self, name: SpriteName) {
        self.new_sprite = Some(name);
    }

    fn reset_animation(&mut self) {
        self.running_data.current_frame = 0;
        self.running_data.frame_time = 0.0;
        self.running_data.is_animating = true;
    }

    pub fn origin(&self) -> Option<Origin> {
        if let Some(sprite_data) = &self.sprite_data {
            Some(sprite_data.origin)
        } else {
            None
        }
    }

    pub fn sprite_name(&self) -> Option<SpriteName> {
        if let Some(sprite_data) = &self.sprite_data {
            Some(sprite_data.sprite_name)
        } else {
            None
        }
    }

    pub fn set_sprite(&mut self, name: SpriteName, resources: &ResourcesDatabase) {
        if let Some(sprite_data) = resources.sprites.get(&name) {
            self.sprite_data = Some(sprite_data.clone());
        } else {
            error!(
                "We couldn't find the sprite data for {} in resources. Are we loading it correctly?",
                name
            );
        }
        self.running_data.current_frame = 0;
        self.running_data.frame_time = 0.0;
        self.running_data.is_animating = true;
    }

    pub fn reset_sprite(&mut self, resources: &ResourcesDatabase) {
        let old_sd = self.sprite_data.take();

        if let Some(our_sd) = old_sd {
            self.set_sprite(our_sd.sprite_name, resources);
        }
    }
}

impl ComponentBounds for Sprite {
    fn entity_inspector(&mut self, inspector_parameters: InspectorParameters<'_, '_>) {
        let InspectorParameters { uid, ui, .. } = inspector_parameters;

        if let Some(new_sprite) = imgui_system::typed_enum_selection_option(ui, &self.sprite_name(), uid) {
            self.reset_animation();
            self.new_sprite = new_sprite;
        };

        self.running_data.inspect(ui, uid);

        if let Some(sprite_data) = &self.sprite_data {
            // CURRENT FRAME
            let mut frame = self.running_data.current_frame as i32;
            if ui
                .input_int(&imgui::im_str!("Current Frame##{}", uid), &mut frame)
                .build()
            {
                let frame = number_util::wrap_usize(frame, 0, sprite_data.frames.len() as i32);
                self.running_data.current_frame = frame as usize;
            }
        }
    }
}

impl StandardQuadFactory for Sprite {
    fn to_standard_quad(&self, pos: Vec2) -> StandardQuad {
        let (texture_info, image_size, pos) = if let Some(sprite_data) = &self.sprite_data {
            let tex_info = StandardTexture {
                norm_image_coordinate: self.current_frame().unwrap().normalized_coord,
                norm_image_size: sprite_data.normalized_dimensions,
                texture_page: sprite_data.texture_page.unwrap(),
            };

            let mut position = pos
                - sprite_data
                    .origin
                    .gfx_adjustment(sprite_data.size)
                    .cwise_product(self.running_data.scale);

            let image_size = {
                let mut size: Vec2 = Vec2::from(sprite_data.size).cwise_product(self.running_data.scale);
                if self.running_data.facing_horizontal != sprite_data.facing_horizontal {
                    position.x += size.x;
                    size.reflected(Axis::X);
                }

                if self.running_data.facing_vertical != sprite_data.facing_vertical {
                    position.y += size.y;
                    size.reflected(Axis::Y);
                }

                size
            };

            (TextureDescription::Standard(tex_info), image_size, position)
        } else {
            (TextureDescription::White, Vec2::ZERO, Vec2::ZERO)
        };

        StandardQuad {
            pos,
            draw_order: self.running_data.draw_order,
            color: self.running_data.tint,
            image_size,
            texture_info,
        }
    }
}

impl Default for Sprite {
    fn default() -> Sprite {
        Sprite {
            // THIS WHOLE THING IS JUST FOR THIS FIELD
            // EVERYTHING ELSE IS NORMAL DEFAULT
            sprite_data: None,
            new_sprite: None,
            running_data: SpriteRunningData {
                scale: Vec2::ONE,
                facing_horizontal: FacingHorizontal::default(),
                facing_vertical: FacingVertical::default(),
                draw_order: DrawOrder::default(),
                tint: Color::default(),

                current_frame: usize::default(),
                frame_time: f32::default(),
                is_animating: bool::default(),
            },
        }
    }
}
