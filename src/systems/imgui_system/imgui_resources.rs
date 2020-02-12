use super::*;
use sprite_resources::*;
use uuid::Uuid;

pub fn create_resources_windows(resources: &mut ResourcesDatabase, ui_handler: &mut UiHandler<'_>) {
    imgui_utility::create_window(
        ui_handler,
        ImGuiFlags::SPRITE_RESOURCE,
        |ui_handler: &mut UiHandler<'_>| sprite_viewer(resources, ui_handler),
    );

    imgui_utility::create_window(
        ui_handler,
        ImGuiFlags::TILEMAP_RESOURCE,
        |ui_handler: &mut UiHandler<'_>| tileset_viewer(resources, ui_handler),
    );

    imgui_utility::create_window(
        ui_handler,
        ImGuiFlags::GAME_CONFIG,
        |ui_handler: &mut UiHandler<'_>| game_config_editor(&mut resources.config, ui_handler),
    );

    imgui_utility::create_window(
        ui_handler,
        ImGuiFlags::PREFAB_INSPECTOR,
        |ui_handler: &mut UiHandler<'_>| prefab_entity_viewer(resources, ui_handler),
    );
}

pub fn sprite_viewer(resources: &mut ResourcesDatabase, ui_handler: &mut UiHandler<'_>) -> bool {
    let mut close = true;
    let ui: &mut Ui<'_> = &mut ui_handler.ui;
    let sprite_resource_window = imgui::Window::new(imgui::im_str!("Sprite Resource"))
        .size(
            Vec2::new(290.0, 400.0).into(),
            imgui::Condition::FirstUseEver,
        )
        .opened(&mut close);

    if let Some(window) = sprite_resource_window.begin(ui) {
        let current_spd: Vec<&mut SpriteData> = {
            let mut temp: Vec<_> = resources.sprites.values_mut().collect();
            temp.sort_by(|elem, next_elem| elem.sprite_name.cmp(&next_elem.sprite_name));
            temp
        };

        for (i, sprite_data) in current_spd.into_iter().enumerate() {
            let uid = &format!("{}{}", sprite_data.sprite_name, i);
            ui.text(im_str!("{}", sprite_data.sprite_name.better_display()));

            ui.spacing();
            sprite_data.origin.inspect(ui, uid, sprite_data.size);
            ui.spacing();

            // FACING DIRECTIONS
            cardinals::inspect_facing(
                ui,
                uid,
                &mut sprite_data.facing_horizontal,
                &mut sprite_data.facing_vertical,
            );

            ui.spacing();
            ui.label_text(&im_str!("Size##{}", uid), &im_str!("{}", sprite_data.size));

            ui.label_text(
                &im_str!("Texture Page##{}", uid),
                &imgui_utility::pretty_option_debug(&sprite_data.texture_page),
            );

            // Texture page fun times!
            if sprite_data.texture_page.is_some() {
                let l = sprite_data.frames.len();

                if l > 1 {
                    let mut val = sprite_data.frames[0].duration;
                    if ui
                        .drag_float(&im_str!("All##{}", uid), &mut val)
                        .min(0.0)
                        .max(0.5)
                        .speed(0.005)
                        .build()
                    {
                        for this_frame in &mut sprite_data.frames {
                            this_frame.duration = val;
                        }
                    }
                }

                for (i, this_frame) in sprite_data.frames.iter_mut().enumerate() {
                    ui.drag_float(&im_str!("Frame {}##{}", i, uid), &mut this_frame.duration)
                        .min(0.0)
                        .max(0.5)
                        .speed(0.005)
                        .build();
                }
            }

            // Serialize, Deserialize
            if ui.button(&im_str!("Serialize##{}", uid), [0.0, 0.0]) {
                if let Err(e) = serialization_util::sprites::serialize_sprite(sprite_data) {
                    error!(
                        "Couldn't serialize the tile set! Warning! Your data may be lost! {}",
                        e
                    )
                }
            }

            ui.same_line_with_spacing(0.0, 25.0);
            if ui.button(&im_str!("Revert##{}", uid), [0.0, 0.0]) {
                if let Ok(sprite_ingame_data) = serialization_util::sprites::load_sprite(
                    sprite_data.sprite_name,
                    sprite_data.texture_page.unwrap(),
                ) {
                    *sprite_data = sprite_ingame_data;
                }
            }

            ui.separator();
        }
        window.end(ui);
    }

    close
}

pub fn tileset_viewer(resources: &mut ResourcesDatabase, ui_handler: &mut UiHandler<'_>) -> bool {
    let mut close = true;
    let ui: &mut Ui<'_> = &mut ui_handler.ui;
    let tileset_viewer_window = imgui::Window::new(imgui::im_str!("Tileset Resources"))
        .size(
            Vec2::new(290.0, 400.0).into(),
            imgui::Condition::FirstUseEver,
        )
        .opened(&mut close);

    if let Some(window) = tileset_viewer_window.begin(ui) {
        let tile_sets: Vec<&mut tile_resources::TileSet> = {
            let mut temp: Vec<_> = resources.tilesets.values_mut().collect();
            temp.sort_by(|elem, next_elem| elem.name.cmp(&next_elem.name));
            temp
        };

        for (i, this_tileset) in tile_sets.into_iter().enumerate() {
            // Ignore the default tset
            if this_tileset.name == tile_resources::TileSetName::Default {
                continue;
            }

            let unique_id = format!("tileset{}{}", this_tileset.name, i);

            // Name
            ui.text(im_str!("{}", this_tileset.name));

            // Size
            imgui_utility::input_usize(
                ui,
                &imgui::im_str!("Number of Tiles##{}", unique_id),
                &mut this_tileset.size,
            );

            // Visual Data
            this_tileset.visual_data.inspector(ui, &unique_id);

            let mut current_editing_tile =
                if let EditingMode::Editing(Some(v), _) = this_tileset.editing_mode {
                    v
                } else {
                    0
                };

            let tileset_size = (this_tileset.size as i32).max(0);

            // Change the physics data
            if imgui_utility::typed_option_selection(
                "Is Collidable",
                "Yes",
                "No",
                ui,
                &unique_id,
                &mut this_tileset.physics_data.bounding_boxes,
                |bounding_boxes| {
                    if bounding_boxes.is_empty() {
                        for _ in 0..tileset_size as usize {
                            bounding_boxes.push(physics_components::RelativeBoundingBox::default());
                        }
                    }

                    if ui
                        .input_int(im_str!("Tile to Edit"), &mut current_editing_tile)
                        .build()
                    {
                        current_editing_tile = current_editing_tile.min(tileset_size - 1);
                        current_editing_tile = current_editing_tile.max(0);
                    }

                    let this_bb: &mut physics_components::RelativeBoundingBox =
                        &mut bounding_boxes[current_editing_tile as usize];

                    if this_bb
                        .rect
                        .rect_inspector(ui, &format!("{}{}", unique_id, current_editing_tile))
                    {
                        // Clamp the Min
                        math::clamped(&mut this_bb.rect.min.x, 0.0, 1.0);
                        math::clamped(&mut this_bb.rect.min.y, 0.0, 1.0);

                        // Clamp the Max
                        math::clamped(&mut this_bb.rect.max.x, 0.0, 1.0);
                        math::clamped(&mut this_bb.rect.max.y, 0.0, 1.0);
                        true
                    } else {
                        false
                    }
                },
                |_| false,
            ) {
                this_tileset.physics_data.dirty = true;
            }

            if current_editing_tile != 0 {
                this_tileset.editing_mode =
                    EditingMode::Editing(Some(current_editing_tile), vec![]);
            }

            // Serialize, Deserialize
            let serialize_label = im_str!("Serialize##{}", unique_id);
            if ui.button(&serialize_label, [0.0, 0.0]) {
                if let Err(e) =
                    serialization_util::tilesets::serialize_tileset(this_tileset.clone())
                {
                    error!(
                        "Couldn't serialize the tile set! Warning! Your data may be lost! {}",
                        e
                    )
                }
            }

            ui.same_line_with_spacing(0.0, 25.0);
            if ui.button(&im_str!("Revert##{}", unique_id), [0.0, 0.0]) {
                this_tileset.revert = true;

                if let Ok(Some(reverted_tset)) =
                    serialization_util::tilesets::load_tileset(this_tileset.name)
                {
                    *this_tileset = reverted_tset;
                }
            }
        }

        window.end(ui);
    }

    close
}

pub fn game_config_editor(
    config: &mut game_config::Config,
    ui_handler: &mut UiHandler<'_>,
) -> bool {
    let mut close = true;
    let ui: &mut Ui<'_> = &mut ui_handler.ui;
    let game_config_window = imgui::Window::new(imgui::im_str!("Game Config Editor"))
        .size(
            Vec2::new(290.0, 400.0).into(),
            imgui::Condition::FirstUseEver,
        )
        .opened(&mut close);

    if let Some(window) = game_config_window.begin(ui) {
        let uid = "game_config";

        config
            .window_size
            .inspector(ui, &im_str!("Window Size##{}", uid));

        ui.input_float(
            &im_str!("ImGui Font Size##{}", uid),
            &mut config.imgui_pixel_size,
        )
        .build();

        // Serialize
        if ui.button(&im_str!("Serialize##{}", uid), [-1.0, 0.0]) {
            if let Err(e) = serialization_util::game_config::serialize_config(config) {
                error!(
                    "Couldn't serialize the Game Config! Warning! Your data may be lost! {}",
                    e
                )
            }
        }
        window.end(ui)
    }

    close
}

pub fn prefab_entity_viewer(
    resources: &mut ResourcesDatabase,
    ui_handler: &mut UiHandler<'_>,
) -> bool {
    let mut open = true;

    let mut prefab_to_clone: Option<Uuid> = None;
    let mut prefab_to_delete: Option<Uuid> = None;
    let mut prefab_to_console_log: Option<Uuid> = None;

    let prefab_list = imgui::Window::new(&im_str!("Prfab List"))
        .size([200.0, 400.0], imgui::Condition::FirstUseEver)
        .menu_bar(true)
        .opened(&mut open);

    if let Some(window) = prefab_list.begin(&ui_handler.ui) {
        for (id, prefab) in resources.prefabs_mut().unwrap().iter_mut() {
            let nip = NameInspectorParameters {
                has_children: false,
                depth: 0,
                is_prefab: true,
                being_inspected: ui_handler.stored_prefabs.contains(id),
                // All this does is stop the "unserialize" option from appearing.
                is_serialized: false,
            };

            // ENTITY ELEMENTS:
            let (_, serialize_name) = display_prefab_id(
                prefab.root_id(),
                &nip,
                prefab.root_entity().name.as_ref().map(|sc| &sc.inner),
                ui_handler,
                &mut prefab_to_clone,
                &mut prefab_to_delete,
                &mut prefab_to_console_log,
            );

            if let Some(new_name) = serialize_name {
                if let Some(serialized_name) = &mut prefab.root_entity_mut().name {
                    serialized_name.inner.name = new_name;
                } else {
                    prefab.root_entity_mut().name = Some(SerializedComponent {
                        active: true,
                        inner: Name::new(new_name),
                    })
                }
            }
        }

        if let Some(original) = prefab_to_clone {
            let clone: Prefab = resources.prefabs().get(&original).unwrap().clone();

            resources
                .prefabs_mut()
                .unwrap()
                .insert(clone.root_id(), clone);
        }

        if let Some(id) = prefab_to_delete {
            compile_error!("We need to also delete from the cold prefabs in some manner here...");
            compile_error!("And also from the actual file system too!");
            resources.prefabs_mut().unwrap().remove(&id);
        }

        if let Some(console_log) = prefab_to_console_log {
            println!("---Console Log for {}---", console_log);
            println!("{:#?}", resources.prefabs_mut().unwrap()[&console_log]);
            println!("-------------------------");
        }

        window.end(&ui_handler.ui);
    }

    open
}

fn display_prefab_id(
    prefab: Uuid,
    name_inspector_params: &NameInspectorParameters,
    name: Option<&Name>,
    ui_handler: &mut UiHandler<'_>,
    clone_me: &mut Option<Uuid>,
    delete_me: &mut Option<Uuid>,
    console_dump_me: &mut Option<Uuid>,
) -> (bool, Option<String>) {
    // Find our ImGui entry list info
    let entity_list_info = ui_handler
        .entity_list_information
        .entry(prefab.to_string())
        .or_default();

    let NameInspectorResult {
        serialize_name,
        unserialize: _,
        inspect,
        show_children,
        clone,
        delete,
        dump_into_console_log,
    } = imgui_utility::display_name_core(
        name.map_or(&format!("Prefab {}", prefab), |name| &name.name),
        entity_list_info,
        name_inspector_params,
        &ui_handler.ui,
        &prefab.to_string(),
    );

    if clone {
        *clone_me = Some(prefab);
    }

    if delete {
        *delete_me = Some(prefab);
        if let Some(pos) = ui_handler.stored_prefabs.iter().position(|u| *u == prefab) {
            ui_handler.stored_prefabs.remove(pos);
        }
    }

    if dump_into_console_log {
        *console_dump_me = Some(prefab);
    }

    // Store or Remove it...
    if inspect {
        if let Some(pos) = ui_handler.stored_prefabs.iter().position(|u| *u == prefab) {
            ui_handler.stored_prefabs.remove(pos);
        } else {
            ui_handler.stored_prefabs.push(prefab);
        }
    }

    // Should we change the name?
    (show_children, serialize_name)
}
