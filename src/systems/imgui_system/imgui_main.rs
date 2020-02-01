use super::*;

pub fn imgui_main(
    ecs: &mut Ecs,
    resources: &mut ResourcesDatabase,
    hardware_interfaces: &mut HardwareInterface,
    ui_handler: &mut UiHandler<'_>,
    time_keeper: &TimeKeeper,
) {
    main_menu_bar(
        hardware_interfaces
            .input
            .kb_input
            .is_pressed(winit::event::VirtualKeyCode::F1),
        ui_handler,
    );

    // Scene Entity Inspector
    if ui_handler.flags.contains(ImGuiFlags::ENTITY_VIEWER) {
        imgui_entity::entity_list(ecs, resources, ui_handler);
    }

    // Window for Each Entity
    imgui_component::entity_inspector(
        &mut ecs.component_database,
        &mut ecs.singleton_database,
        resources,
        &ecs.entities,
        ui_handler,
    );

    // Singleton
    imgui_utility::create_window(ui_handler, ImGuiFlags::SINGLETONS, |ui_handler| {
        imgui_singleton::singleton_inspector(
            &mut ecs.singleton_database,
            &ecs.component_database.names,
            &ecs.entities,
            &resources.prefabs,
            ui_handler,
        )
    });

    // Time Keeper
    imgui_utility::create_window(ui_handler, ImGuiFlags::TIME_KEEPER, |ui_handler| {
        time_keeper.create_imgui_window(ui_handler)
    });

    // Resources Windows
    imgui_resources::create_resources_windows(resources, ui_handler, &mut ecs.component_database);

    // Window for each Prefabs
    imgui_prefab::prefab_editor(ui_handler, resources, &mut ecs.component_database);

    // Always last here...
    if ui_handler.ui.io().want_capture_mouse {
        hardware_interfaces.input.mouse_input.clear();
        hardware_interfaces.input.mouse_input.clear_held();
    }
    if ui_handler.ui.io().want_capture_keyboard {
        hardware_interfaces.input.kb_input.clear();
        hardware_interfaces.input.kb_input.held_keys.clear();
    }
}

fn main_menu_bar(toggle_main_menu_bar: bool, ui_handler: &mut UiHandler<'_>) {
    if toggle_main_menu_bar {
        ui_handler.flags.toggle(ImGuiFlags::MAIN_MENU_BAR);
    }

    if ui_handler.flags.contains(ImGuiFlags::MAIN_MENU_BAR) {
        // MENU
        let ui = &ui_handler.ui;
        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            if let Some(menu) = ui.begin_menu(im_str!("Scene"), true) {
                // BLANK ENTITY
                menu_option(
                    im_str!("Component Inspector"),
                    ImGuiFlags::ENTITY_VIEWER,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Singleton Inspector"),
                    ImGuiFlags::SINGLETONS,
                    ui,
                    &mut ui_handler.flags,
                );

                menu.end(ui);
            }

            // UTILITIES
            if let Some(utility_bar) = ui.begin_menu(im_str!("Utilities"), true) {
                menu_option(
                    im_str!("Time Keeper"),
                    ImGuiFlags::TIME_KEEPER,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Game Config Inspector"),
                    ImGuiFlags::GAME_CONFIG,
                    ui,
                    &mut ui_handler.flags,
                );
                utility_bar.end(ui);
            }

            // PANELS
            if let Some(other_windows) = ui.begin_menu(im_str!("Assets"), true) {
                menu_option(
                    im_str!("Sprite Inspector"),
                    ImGuiFlags::SPRITE_RESOURCE,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Tile Set Inspector"),
                    ImGuiFlags::TILEMAP_RESOURCE,
                    ui,
                    &mut ui_handler.flags,
                );

                menu_option(
                    im_str!("Prefab Inspector"),
                    ImGuiFlags::PREFAB_INSPECTOR,
                    ui,
                    &mut ui_handler.flags,
                );

                other_windows.end(ui);
            }

            menu_bar.end(ui);
        }
    }
}

fn menu_option(imstr: &imgui::ImStr, flag: ImGuiFlags, ui: &Ui<'_>, flags_to_change: &mut ImGuiFlags) {
    if imgui::MenuItem::new(imstr)
        .selected(flags_to_change.contains(flag))
        .build(ui)
    {
        flags_to_change.toggle(flag);
    }
}
