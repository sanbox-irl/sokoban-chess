use super::{
    cardinals::CardinalPrime, Component, ComponentList, Entity, Marker, Player, Transform, Vec2,
    Velocity,
};
use array2d::Array2D;

pub const GRID_DIMENSIONS: (usize, usize) = (5, 10);
pub const GRID_DIMENSIONS_MIN_F32: (f32, f32) = (8.0, 8.0);
pub const GRID_DIMENSIONS_MAX_F32: (f32, f32) = (
    GRID_DIMENSIONS.0 as f32 * 8.0,
    GRID_DIMENSIONS.1 as f32 * 8.0,
);
pub type Grid = Array2D<Option<Entity>>;

pub fn update_grid_positions(
    transforms: &mut ComponentList<Transform>,
    velocities: &mut ComponentList<Velocity>,
    players: &ComponentList<Player>,
    grid: &mut Grid,
) {
    // Core movement:
    for this_velocity in velocities.iter_mut() {
        let entity_id = this_velocity.entity_id;
        if let Some(movement) = this_velocity.inner_mut().intended_direction.take() {
            if let Some(transform) = transforms.get_mut(&entity_id) {
                let current_position =
                    world_to_grid_position(transform.inner_mut().world_position());

                if let Some(valid_next_position) = move_position(current_position, movement) {
                    if let Some(entity_in_grid) = grid[valid_next_position] {
                        // Check if we can move on the entity...I guess?
                    } else {
                        move_entity(transform, grid, valid_next_position, current_position);
                    }
                } else {
                    error!("Can't walk there! That's outside the grid bounds!");
                }
            };
        }
    }
}

pub fn initialize_transforms(
    transforms: &mut ComponentList<Transform>,
    grid: &mut Grid,
    markers: &std::collections::HashMap<Marker, Entity>,
) {
    for transform_c in transforms.iter() {
        if markers
            .values()
            .find(|e| *e == &transform_c.entity_id)
            .is_none()
        {
            super::grid_system::register_entity(
                grid,
                transform_c.entity_id,
                transform_c.inner().world_position(),
            );
        }
    }
}

fn register_entity(grid: &mut Grid, entity: Entity, position: Vec2) {
    if position.x >= GRID_DIMENSIONS_MIN_F32.0
        && position.x < GRID_DIMENSIONS_MAX_F32.0
        && position.y >= GRID_DIMENSIONS_MIN_F32.0
        && position.y < GRID_DIMENSIONS_MAX_F32.1
    {
        grid[world_to_grid_position(position)] = Some(entity);
    } else {
        error!("Couldn't register entity: {} to grid!", entity);
    }
}

fn move_entity(
    transform: &mut Component<Transform>,
    grid: &mut Grid,
    valid_next_position: (usize, usize),
    current_position: (usize, usize),
) {
    grid[valid_next_position] = Some(transform.entity_id);
    grid[current_position] = None;
    transform
        .inner_mut()
        .set_local_position(grid_to_world_position(valid_next_position));
}

fn world_to_grid_position(pos: Vec2) -> (usize, usize) {
    let tuple: (f32, f32) = ((pos - Vec2::new(8.0, 8.0)).cwise_div(Vec2::new(8.0, 8.0))).into();
    (tuple.0 as usize, tuple.1 as usize)
}

fn grid_to_world_position(pos: (usize, usize)) -> Vec2 {
    Vec2::new(pos.0 as f32, pos.1 as f32).cwise_product(Vec2::with_single(8.0))
        + Vec2::with_single(8.0)
}

fn move_position(pos: (usize, usize), direction: CardinalPrime) -> Option<(usize, usize)> {
    match direction {
        CardinalPrime::Right => {
            if pos.0 + 1 < GRID_DIMENSIONS.0 {
                Some((pos.0 + 1, pos.1))
            } else {
                None
            }
        }
        CardinalPrime::Up => {
            if pos.1 + 1 < GRID_DIMENSIONS.1 {
                Some((pos.0, pos.1 + 1))
            } else {
                None
            }
        }
        CardinalPrime::Left => {
            if pos.0 != 0 {
                Some((pos.0 - 1, pos.1))
            } else {
                None
            }
        }
        CardinalPrime::Down => {
            if pos.1 != 0 {
                Some((pos.0, pos.1 - 1))
            } else {
                None
            }
        }
    }
}
