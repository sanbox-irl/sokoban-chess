use super::{
    cardinals::{CardinalPrime, FacingHorizontal},
    number_util, ActionMap, ComponentList, Player, Transform,
};

pub fn player_update(
    players: &mut ComponentList<Player>,
    transforms: &mut ComponentList<Transform>,
    action_map: &ActionMap,
) {
    let mut active_player: Option<isize> = None;

    for (i, player) in players.iter_mut().enumerate() {
        let id = player.entity_id;
        let player: &mut Player = player.inner_mut();
        if let Some(transform) = transforms.get_mut(&id) {
            let player_transform = transform.inner_mut();

            if active_player.is_none() && player.active {
                active_player = Some(i as isize);
                if let Some(move_dir) = action_map.move_direction() {
                    match move_dir {
                        CardinalPrime::Right => {
                            player_transform.edit_local_position(|p| p + Transform::TILE_RIGHT)
                        }
                        CardinalPrime::Up => {
                            player_transform.edit_local_position(|p| p + Transform::TILE_UP)
                        }
                        CardinalPrime::Left => {
                            player_transform.edit_local_position(|p| p - Transform::TILE_RIGHT)
                        }
                        CardinalPrime::Down => {
                            player_transform.edit_local_position(|p| p - Transform::TILE_UP)
                        }
                    }
                }
            }
        }
    }

    // Check for Active Player
    if let Some(mut active_player) = active_player {
        if let Some(increment_active_player) = action_map.switch_active_player() {
            active_player += match increment_active_player {
                FacingHorizontal::Right => 1,
                FacingHorizontal::Left => -1,
            };

            let correct_index =
                number_util::wrap_isize(active_player, 0, players.iter().count() as isize) as usize;
            if let Some(p) = players.iter_mut().nth(correct_index) {
                p.inner_mut().active = true;
            }
        }
    } else {
        if let Some(zeroeth) = players.iter_mut().nth(0) {
            zeroeth.inner_mut().active = true;
        } else {
            log_once::error_once!("We have no active players. That seems bad.");
        }
    }
}
