use super::{
    components::{ComponentDatabase, Entity},
    components_singleton::SingletonDatabase,
    entities::EntityAllocator,
    hardware_interfaces::HardwareInterface,
    resources::ResourcesDatabase,
    systems::*,
    Vec2,
};
use failure::Error;
use lazy_static::lazy_static;
use std::sync::Mutex;

const INITIAL_SCENE: SceneName = SceneName::Main;
lazy_static! {
    pub static ref CURRENT_SCENE: Mutex<SceneName> = Mutex::new(INITIAL_SCENE);
}

pub struct Ecs {
    pub component_database: ComponentDatabase,
    pub singleton_database: SingletonDatabase,
    pub entities: Vec<Entity>,
    pub entity_allocator: EntityAllocator,
}

impl Ecs {
    pub fn new() -> Result<Self, Error> {
        // Es and Cs
        let mut entity_allocator = EntityAllocator::new();
        let mut entities = Vec::new();

        // Deserialize Entities and Singletons
        let mut marker_map = std::collections::HashMap::new();
        let component_database =
            ComponentDatabase::new(&mut entity_allocator, &mut entities, &mut marker_map)?;

        let singleton_database = SingletonDatabase::new(marker_map)?;

        // serialization panic guard
        if entities.is_empty() {
            bail!("We have an empty ECS! Something probably went wrong in deserialization!");
        }

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
        resources: &mut ResourcesDatabase,
        hardware_interfaces: &mut HardwareInterface,
    ) -> Result<(), Error> {
        self.singleton_database
            .initialize_with_runtime_resources(resources, hardware_interfaces);

        tilemap_system::initialize_tilemaps(
            &mut self.component_database.tilemaps,
            &resources.tilesets,
        );
        conversant_npc_system::initialize_conv_npc_ui(self, resources);
        Ok(())
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        resources: &ResourcesDatabase,
        hardware_interfaces: &HardwareInterface,
    ) -> Result<(), Error> {
        // // Player Stuff
        // player_system::player_update(
        //     &self.singleton_database.player.inner(),
        //     self.singleton_database
        //         .associated_entities
        //         .get(&self.singleton_database.player.marker()),
        //     &mut self.component_database,
        //     &hardware_interfaces.input,
        //     delta_time,
        // );

        follow_system::update_follows(
            &mut self.component_database.follows,
            &mut self.component_database.transforms,
            &self.component_database.names,
            delta_time,
        );

        let camera_associated_entity = self
            .singleton_database
            .associated_entities
            .get(&self.singleton_database.camera.marker())
            .unwrap();

        singleton_systems::update_camera(
            self.singleton_database.camera.inner_mut(),
            camera_associated_entity,
            &mut self.component_database.transforms,
            &mut self.component_database.follows,
            &hardware_interfaces.input,
        );

        conversant_npc_system::update_conv_npc_ui_sprites(
            self,
            &resources,
            hardware_interfaces
                .input
                .kb_input
                .is_pressed(winit::event::VirtualKeyCode::Return),
        );

        cross_cutting_system::cross_cutting_system(self);

        Ok(())
    }

    pub fn render<'a, 'b>(&'a mut self, draw_commands: &'b mut DrawCommand<'a>, size: Vec2) {
        let camera_entity = self
            .singleton_database
            .associated_entities
            .get(&self.singleton_database.camera.marker())
            .expect("We couldn't find the camera. We require a valid camera!");

        draw_commands.take_game_world(
            &self.component_database.text_sources,
            &self.component_database.sprites,
            &self.component_database.draw_rectangles,
            &self.component_database.tilemaps,
            &self.component_database.transforms,
            camera_entity,
            self.singleton_database.camera.inner(),
            &mut self.singleton_database.rendering_utility,
            size,
        );
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
