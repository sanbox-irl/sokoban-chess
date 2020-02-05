use super::{
    components::{ComponentDatabase, Entity},
    components_singleton::SingletonDatabase,
    entities::EntityAllocator,
    hardware_interfaces::HardwareInterface,
    resources::ResourcesDatabase,
    systems::*,
    ActionMap, GameWorldDrawCommands,
};
use failure::Error;

pub struct Ecs {
    pub component_database: ComponentDatabase,
    pub singleton_database: SingletonDatabase,
    pub entities: Vec<Entity>,
    pub entity_allocator: EntityAllocator,
}

impl Ecs {
    pub fn new(prefabs: &PrefabMap) -> Result<Self, Error> {
        // Es and Cs
        let mut entity_allocator = EntityAllocator::new();
        let mut entities = Vec::new();

        // Deserialize Entities and Singletons
        let mut marker_map = std::collections::HashMap::new();
        let component_database = ComponentDatabase::new(
            &mut entity_allocator,
            &mut entities,
            &mut marker_map,
            prefabs,
        )?;

        let singleton_database = SingletonDatabase::new(marker_map)?;

        Ok(Ecs {
            entities,
            entity_allocator,
            component_database,
            singleton_database,
        })
    }

    /// The difference between GameStart and New is that everyting in initialized by now.
    pub fn game_start(
        &mut self,
        resources: &ResourcesDatabase,
        hardware_interfaces: &HardwareInterface,
        grid: &mut grid_system::Grid,
    ) -> Result<(), Error> {
        self.singleton_database
            .initialize_with_runtime_resources(resources, hardware_interfaces);

        tilemap_system::initialize_tilemaps(
            &mut self.component_database.tilemaps,
            &resources.tilesets,
        );

        player_system::initialize_players(
            &mut self.component_database.players,
            &mut self.component_database.sprites,
        );

        grid_system::initialize_transforms(
            &mut self.component_database.transforms,
            &self.component_database.names,
            grid,
            &self.singleton_database.associated_entities,
        );

        Ok(())
    }

    pub fn update(
        &mut self,
        grid: &mut grid_system::Grid,
        actions: &ActionMap,
    ) -> Result<(), Error> {
        // // Player Stuff
        player_system::player_update(
            &mut self.component_database.players,
            &mut self.component_database.sprites,
            &mut self.component_database.velocities,
            actions,
        );

        // Movement Stuff
        grid_system::update_grid_positions(
            &mut self.component_database.players,
            &mut self.component_database.transforms,
            &mut self.component_database.velocities,
            &mut self.component_database.grid_objects,
            &mut self.component_database.scene_switchers,
            grid,
        );

        Ok(())
    }

    pub fn update_resources(&mut self, resources: &ResourcesDatabase) {
        cross_cutting_system::cross_cutting_system(self, resources);
    }

    pub fn render<'a, 'b>(
        &'a mut self,
        draw_commands: &'b mut DrawCommand<'a>,
        resources: &'a ResourcesDatabase,
    ) {
        if let Some(camera_entity) = self
            .singleton_database
            .associated_entities
            .get(&self.singleton_database.camera.marker())
        {
            draw_commands.game_world = Some(GameWorldDrawCommands {
                text_sources: &self.component_database.text_sources,
                sprites: &self.component_database.sprites,
                rects: &self.component_database.draw_rectangles,
                tilemaps: &self.component_database.tilemaps,
                transforms: &self.component_database.transforms,
                camera_entity,
                camera: self.singleton_database.camera.inner(),
                rendering_utility: &mut self.singleton_database.rendering_utility,
                resources,
            })
        } else {
            log_once::error_once!(
                "No camera is present! The game world cannot draw without a camera entity!"
            );
        }
    }
}

impl Ecs {
    /// This is the standard method to create a new Entity in the Ecs. Try to
    /// always use this one. The returned Entity is the ID, or index, of the new
    /// entity.
    pub fn create_entity(&mut self) -> Entity {
        Ecs::create_entity_raw(
            &mut self.component_database,
            &mut self.entity_allocator,
            &mut self.entities,
        )
    }

    /// Use this only in weird situations. Otherwise, prefer to pass
    /// the entire Ecs around now that we have a leaner top level
    /// struct.
    pub fn create_entity_raw(
        component_database: &mut ComponentDatabase,
        entity_allocator: &mut EntityAllocator,
        entities: &mut Vec<Entity>,
    ) -> Entity {
        let entity = entity_allocator.allocate();
        component_database.register_entity(entity);
        entities.push(entity);
        entity
    }

    pub fn remove_entity(&mut self, entity_to_delete: &Entity) -> bool {
        let is_dealloc = self.entity_allocator.deallocate(entity_to_delete);
        if is_dealloc {
            self.component_database
                .deregister_entity(&entity_to_delete, false);
            self.entities
                .iter()
                .position(|i| i == entity_to_delete)
                .map(|i| self.entities.remove(i));
        }
        is_dealloc
    }

    pub fn clone_entity(&mut self, original: &Entity) -> Entity {
        let new_entity = self.create_entity();
        self.component_database.clone_components(
            original,
            &new_entity,
            &mut self.singleton_database,
        );

        new_entity
    }
}
