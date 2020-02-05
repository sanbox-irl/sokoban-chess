use super::{
    cardinals::CardinalPrime, Component, ComponentList, Entity, GridObject, GridType, Marker, Name,
    Player, Transform, Vec2, Velocity,
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
    players: &mut ComponentList<Player>,
    transforms: &mut ComponentList<Transform>,
    velocities: &mut ComponentList<Velocity>,
    grid_objects: &mut ComponentList<GridObject>,
    grid: &mut Grid,
) {
    // ImGui Movement
    for grid_object_c in grid_objects.iter_mut() {
        let id = grid_object_c.entity_id();
        let grid_object: &mut GridObject = grid_object_c.inner_mut();

        if grid_object.move_to_point {
            if let Some(transform) = transforms.get_mut(&id) {
                let current_position = world_to_grid_position(transform.inner().world_position());
                let desired_position: (usize, usize) = (
                    grid_object.move_to_point_pos.x as usize,
                    grid_object.move_to_point_pos.y as usize,
                );

                if desired_position.0 >= GRID_DIMENSIONS.0
                    || desired_position.1 >= GRID_DIMENSIONS.1
                {
                    error!("Couldn't move! Attempting to move to far!")
                } else {
                    move_entity(transform, grid, desired_position, current_position);
                }
            }

            // Reset the Grid Object Move to Point
            grid_object.move_to_point = false;
            grid_object.move_to_point_pos = Default::default();
        }

        if grid_object.register {
            if let Some(transform) = transforms.get(&id) {
                register_entity(grid, id, transform.inner().world_position(), None);
            }

            grid_object.register = false;
        }
    }

    // Player Movement
    for player in players.iter_mut() {
        let entity_id = player.entity_id();

        let movement: Option<CardinalPrime> = {
            if let Some(vc) = velocities.get_mut(&entity_id) {
                vc.inner_mut().intended_direction.take()
            } else {
                None
            }
        };
        let current_position = transforms
            .get_mut(&entity_id)
            .map(|tc| world_to_grid_position(tc.inner_mut().world_position()));

        if let Some(movement) = movement {
            if let Some(current_position) = current_position {
                if let Some(valid_next_position) = move_position(current_position, movement) {
                    attempt_to_move(
                        &entity_id,
                        GridType::Player,
                        current_position,
                        valid_next_position,
                        movement,
                        grid_objects,
                        transforms,
                        grid,
                    );
                }
            }
        }
    }
}

pub fn initialize_transforms(
    transforms: &mut ComponentList<Transform>,
    names: &ComponentList<Name>,
    grid: &mut Grid,
    markers: &std::collections::HashMap<Marker, Entity>,
) {
    for transform_c in transforms.iter() {
        if markers
            .values()
            .find(|e| *e == &transform_c.entity_id())
            .is_none()
        {
            super::grid_system::register_entity(
                grid,
                transform_c.entity_id(),
                transform_c.inner().world_position(),
                Some(names),
            );
        }
    }
}

fn attempt_to_move(
    entity_id: &Entity,
    my_object_type: GridType,
    current_position: (usize, usize),
    next_position: (usize, usize),
    movement: CardinalPrime,
    grid_objects: &ComponentList<GridObject>,
    transforms: &mut ComponentList<Transform>,
    grid: &mut Grid,
) -> bool {
    let mut move_to_spot = true;

    // Check the Entity
    if let Some(entity_in_grid) = grid[next_position] {
        let grid_type = grid_objects
            .get(&entity_in_grid)
            .unwrap()
            .inner()
            .grid_type();

        match grid_type {
            GridType::Flag => {
                if my_object_type == GridType::Player {
                    // switch scene!
                }
            }
            GridType::Pushable => {
                if let Some(next_next_position) = move_position(next_position, movement) {
                    move_to_spot = attempt_to_move(
                        &entity_in_grid,
                        grid_type,
                        next_position,
                        next_next_position,
                        movement,
                        grid_objects,
                        transforms,
                        grid,
                    );
                }
            }
            GridType::Blockable | GridType::Player => {
                move_to_spot = false;
            }
            GridType::NonInteractable => {
                // good to go!
            }
        }
    }

    if move_to_spot {
        move_entity(
            transforms.get_mut(entity_id).unwrap(),
            grid,
            next_position,
            current_position,
        );
    }

    move_to_spot
}

fn register_entity(
    grid: &mut Grid,
    entity: Entity,
    position: Vec2,
    names: Option<&ComponentList<Name>>,
) {
    if position.x >= GRID_DIMENSIONS_MIN_F32.0
        && position.x <= GRID_DIMENSIONS_MAX_F32.0
        && position.y >= GRID_DIMENSIONS_MIN_F32.0
        && position.y <= GRID_DIMENSIONS_MAX_F32.1
    {
        grid[world_to_grid_position(position)] = Some(entity);
    } else {
        error!(
            "Couldn't register {} to grid!",
            Name::get_name(names, &entity)
        );
    }
}

fn move_entity(
    transform: &mut Component<Transform>,
    grid: &mut Grid,
    valid_next_position: (usize, usize),
    current_position: (usize, usize),
) {
    grid[valid_next_position] = Some(transform.entity_id());
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
