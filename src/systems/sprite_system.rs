use super::{ComponentList, ResourcesDatabase, Sprite};

pub fn update_sprites<'a>(
    sprites: &mut ComponentList<Sprite>,
    resources: &ResourcesDatabase,
    delta_time: f32,
) {
    for this_sprite_component in sprites.iter_mut() {
        let this_sprite = this_sprite_component.inner_mut();

        // Copy over a sprite if we need to
        if let Some(new_sprite) = this_sprite.new_sprite.take() {
            this_sprite.set_sprite(new_sprite, resources);
        }

        // Sprite Animation
        if this_sprite.running_data.is_animating {
            update_sprite_animation(this_sprite, delta_time);
        }
    }
}

fn update_sprite_animation(this_sprite: &mut Sprite, delta_time: f32) {
    if let Some(sprite_data) = &mut this_sprite.sprite_data {
        let this_frame_data = &sprite_data.frames[this_sprite.running_data.current_frame];

        // ACTUAL LOOP
        if let Some(dur) = this_frame_data.duration {
            this_sprite.running_data.frame_time += delta_time;

            while dur > 0.0 && this_sprite.running_data.frame_time > dur {
                this_sprite.running_data.current_frame += 1;
                if this_sprite.running_data.current_frame == sprite_data.frames.len() {
                    this_sprite.running_data.current_frame = 0;
                }
                this_sprite.running_data.frame_time -= dur;
            }
        }
    }
}
