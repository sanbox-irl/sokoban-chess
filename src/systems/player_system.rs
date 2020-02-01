use super::{
    cardinals::FacingHorizontal, physics_system, sprite_resources::SpriteName, ComponentDatabase, Entity,
    Input, Player, Transform, Vec2,
};
use winit::event::VirtualKeyCode as VK;

pub fn player_update<'a>(
    player_singleton: &Player,
    player: Option<&Entity>,
    component_database: &mut ComponentDatabase,
    input: &Input,
    delta_time: f32,
) {
    // Stuff we need to proceed
    let player_entity = match player {
        Some(player) => player,
        None => {
            log_once::error_once!(
                "We cannot find the Player Entity. We cannot run Player Update without it."
            );
            return;
        }
    };

    let player_sprite = match component_database.sprites.get_mut(&player_entity) {
        Some(sp) => sp.inner_mut(),
        None => {
            log_once::error_once!("Player has no Sprite Component. We cannot run Player Update without it.");
            return;
        }
    };

    let _ = match component_database.transforms.get_mut(&player_entity) {
        Some(sp) => sp.inner_mut(),
        None => {
            log_once::error_once!(
                "Player has no Transform Component. We cannot run Player Update without it."
            );
            return;
        }
    };

    let player_veloc = match component_database.velocities.get_mut(&player_entity) {
        Some(vel) => vel.inner_mut(),
        None => {
            log_once::error_once!(
                "Player has no Velocity Component. We cannot run Player Update without it."
            );
            return;
        }
    };

    let _ = match component_database.bounding_boxes.get_mut(&player_entity) {
        Some(vel) => vel.inner_mut(),
        None => {
            log_once::error_once!("Player has no Bounding Boxes. We cannot run Player Update without it.");
            return;
        }
    };

    let mut accel = Vec2::ZERO;
    if input.kb_input.is_held(VK::Up) || input.kb_input.is_held(VK::W) {
        accel += Vec2::UP;
    }

    if input.kb_input.is_held(VK::Left) || input.kb_input.is_held(VK::A) {
        player_sprite.running_data.facing_horizontal = FacingHorizontal::Left;

        accel += -Vec2::RIGHT;
    }

    if input.kb_input.is_held(VK::Down) || input.kb_input.is_held(VK::S) {
        accel += -Vec2::UP;
    }

    if input.kb_input.is_held(VK::Right) || input.kb_input.is_held(VK::D) {
        player_sprite.running_data.facing_horizontal = FacingHorizontal::Right;
        accel += Vec2::RIGHT;
    }

    let moving_this_frame = accel != Vec2::ZERO;

    if moving_this_frame {
        player_sprite.ensure_sprite(SpriteName::PixelMainCharacterWalking);

        let move_amount = accel.normalized() * player_singleton.move_speed * delta_time;
        player_veloc.velocity += move_amount;
    } else {
        player_sprite.ensure_sprite(SpriteName::PixelMainCharacterStanding);
    }

    if player_veloc.velocity != Vec2::ZERO {
        // Friction
        player_veloc.velocity -= player_veloc.velocity
            * delta_time
            * if moving_this_frame {
                player_singleton.friction_moving
            } else {
                player_singleton.friction_standstill
            };

        if player_veloc.velocity.x.abs() < 0.01 {
            player_veloc.velocity.x = 0.0;
        }
        if player_veloc.velocity.y.abs() < 0.01 {
            player_veloc.velocity.y = 0.0;
        }

        let new_world_position = {
            let (my_bounding_box, bounding_boxes) = physics_system::create_positional_rect_lists(
                *player_entity,
                &component_database.transforms,
                &component_database.bounding_boxes,
                &component_database.tilemaps,
            );

            physics_system::move_towards(
                player_veloc.velocity * delta_time,
                my_bounding_box,
                &bounding_boxes,
            )
        };

        // Set the position
        if let Some(transform) = component_database.transforms.get_mut(player_entity) {
            let transform: &mut Transform = transform.inner_mut();

            let positional_differential = new_world_position - transform.world_position();
            transform.edit_local_position(|pos| pos + positional_differential);
        }
    }
}
