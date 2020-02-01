use super::{
    systems::*, Ecs, HardwareInterface, ImGui, ImGuiDrawCommands, ResourcesDatabase, TimeKeeper,
};
use failure::Error;

pub struct Clockwork {
    pub ecs: Ecs,
    pub action_map: ActionMap,
    pub hardware_interfaces: HardwareInterface,
    pub resources: ResourcesDatabase,
    pub time_keeper: TimeKeeper,
}

impl Clockwork {
    pub fn new() -> Result<Self, Error> {
        // Create Hardware Interfaces and Resources Handler
        let mut resources = ResourcesDatabase::new()?;
        let mut hardware_interfaces = HardwareInterface::new(&resources.config)?;
        resources.initialize(&mut hardware_interfaces.renderer)?;

        // Initialize the ECS
        let mut ecs = Ecs::new()?;
        ecs.game_start(&mut resources, &mut hardware_interfaces)?;

        // Load in the Scene Graph
        scene_graph::build_flat(
            &mut ecs.component_database.transforms,
            &ecs.component_database.serialization_data,
        );

        Ok(Clockwork {
            ecs,
            hardware_interfaces,
            resources,
            action_map: ActionMap::default(),
            time_keeper: TimeKeeper::new(),
        })
    }

    pub fn main_loop(&mut self) -> Result<(), Error> {
        // TICK STRUCTS
        let mut imgui = ImGui::new(&self.ecs.entity_allocator, &self.hardware_interfaces.window);
        renderer_system::initialize_imgui(&mut self.hardware_interfaces.renderer, &mut imgui)?;

        info!(
            "Final Window Size is {}",
            self.hardware_interfaces.window.get_window_size()
        );

        loop {
            self.time_keeper.start_frame();

            // GET INPUT PER FRAME
            input_system::poll_events(
                &mut self.hardware_interfaces.input,
                &mut self.hardware_interfaces.window.events_loop,
                &self.hardware_interfaces.window.window,
                |ev, window| imgui.take_input(window, ev),
            );

            if self.hardware_interfaces.input.end_requested {
                break;
            }

            let mut ui_handler = imgui.begin_frame(
                &self.hardware_interfaces.window,
                self.time_keeper.delta_time,
            )?;

            // CFG Resource Updates
            imgui_system::imgui_main(
                &mut self.ecs,
                &mut self.resources,
                &mut self.hardware_interfaces,
                &mut ui_handler,
                &self.time_keeper,
            );
            sprite_system::update_sprites(
                &mut self.ecs.component_database.sprites,
                &mut self.resources,
                self.time_keeper.delta_time,
            );
            tilemap_system::update_tilemaps_and_tilesets(
                &mut self.ecs.component_database.tilemaps,
                &mut self.ecs.component_database.transforms,
                &mut self.resources.tilesets,
                &mut self.resources.sprites,
                &self.hardware_interfaces.input,
                &self.ecs.singleton_database,
            );

            // Make the Action Map:
            self.action_map
                .update(&self.hardware_interfaces.input.kb_input);

            // Update
            while self.time_keeper.accumulator >= self.time_keeper.delta_time {
                self.ecs.update(
                    self.time_keeper.delta_time,
                    &self.resources,
                    &self.action_map,
                )?;
                self.time_keeper.accumulator -= self.time_keeper.delta_time;
            }

            // RENDER
            self.pre_render()?;
            self.render(ui_handler)?;
        }

        imgui.save_meta_data()?;

        Ok(())
    }

    pub fn pre_render(&mut self) -> Result<(), Error> {
        renderer_system::pre_draw(
            &mut self.ecs.component_database,
            &mut self.resources,
            &mut self.hardware_interfaces.renderer,
        )?;

        Ok(())
    }

    pub fn render(&mut self, ui_handler: UiHandler<'_>) -> Result<(), Error> {
        // Update transform by walking the scene graph...
        scene_graph::walk_graph(
            &mut self.ecs.component_database.transforms,
            &self.ecs.component_database.graph_nodes,
        );

        let mut draw_commands = DrawCommand::default();

        self.ecs.render(&mut draw_commands, &self.resources);
        draw_commands.imgui = Some(ImGuiDrawCommands {
            draw_data: ui_handler.ui.render(),
            imgui_dimensions: ui_handler
                .platform
                .scale_size_from_winit(
                    &self.hardware_interfaces.window.window,
                    self.hardware_interfaces
                        .window
                        .window
                        .inner_size()
                        .to_logical(self.hardware_interfaces.window.window.scale_factor()),
                )
                .into(),
        });

        renderer_system::render(
            &mut self.hardware_interfaces.renderer,
            &self.hardware_interfaces.window,
            &mut draw_commands,
        )?;

        Ok(())
    }
}
