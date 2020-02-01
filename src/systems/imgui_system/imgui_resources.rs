use super::*;
use sprite_resources::*;

pub fn create_resources_windows(
    resources: &mut ResourcesDatabase,
    ui_handler: &mut UiHandler<'_>,
    component_database: &mut ComponentDatabase,
) {
    imgui_utility::create_window(
        ui_handler,
        ImGuiFlags::SPRITE_RESOURCE,
        |ui_handler: &mut UiHandler<'_>| {
            sprite_viewer(resources, ui_handler, &mut component_database.sprites)
        },
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

pub fn sprite_viewer(
    resources: &mut ResourcesDatabase,
    ui_handler: &mut UiHandler<'_>,
    sprites: &mut ComponentList<Sprite>,
) -> bool {
    let mut close = true;
    let ui: &mut Ui<'_> = &mut ui_handler.ui;
    let sprite_resource_window = imgui::Window::new(imgui::im_str!("Sprite Resource"))
        .size(Vec2::new(290.0, 400.0).into(), imgui::Condition::FirstUseEver)
        .opened(&mut close);

    if let Some(window) = sprite_resource_window.begin(ui) {
        let current_spd: Vec<&mut SpriteData> = {
            let mut temp: Vec<_> = resources.sprites.values_mut().collect();
            temp.sort_by(|elem, next_elem| elem.sprite_name.cmp(&next_elem.sprite_name));
            temp
        };

        for (i, sprite_data) in current_spd.into_iter().enumerate() {
            let mut dirty = false;

            let uid = &format!("{}{}", sprite_data.sprite_name, i);
            ui.text(im_str!("{}", sprite_data.sprite_name.better_display()));

            ui.spacing();
            if sprite_data.origin.inspect(ui, uid, sprite_data.size) {
                dirty = true;
            }
            ui.spacing();

            // FACING DIRECTIONS
            if cardinals::inspect_facing(
                ui,
                uid,
                &mut sprite_data.facing_horizontal,
                &mut sprite_data.facing_vertical,
            ) {
                dirty = true;
            }

            ui.spacing();
            ui.label_text(&im_str!("Size##{}", uid), &im_str!("{}", sprite_data.size));

            ui.label_text(
                &im_str!("Texture Page##{}", uid),
                &imgui_utility::pretty_option_debug(&sprite_data.texture_page),
            );

            // Texture page fun times!
            if let Some(_) = sprite_data.texture_page {
                let l = sprite_data.frames.len();

                if l > 1 && sprite_data.frames[0].duration.is_some() {
                    let mut val = sprite_data.frames[0].duration.unwrap();
                    if ui
                        .drag_float(&im_str!("All##{}", uid), &mut val)
                        .min(0.0)
                        .max(0.5)
                        .speed(0.005)
                        .build()
                    {
                        dirty = true;
                        for this_frame in &mut sprite_data.frames {
                            this_frame.duration = Some(val);
                        }
                    }
                }

                for (i, this_frame) in sprite_data.frames.iter_mut().enumerate() {
                    match &mut this_frame.duration {
                        Some(dur) => {
                            if ui
                                .drag_float(&im_str!("Frame {}##{}", i, uid), dur)
                                .min(0.0)
                                .max(0.5)
                                .speed(0.005)
                                .build()
                            {
                                dirty = true;
                            }
                        }

                        None => {
                            ui.label_text(&im_str!("No Duration"), &im_str!("~"));
                        }
                    }
                }
            }

            // Serialize, Deserialize
            let serialize_label = im_str!("Serialize##{}", uid);
            if imgui_utility::sized_button(ui, &serialize_label) {
                if let Err(e) = serialization_util::sprites::serialize_sprite(sprite_data) {
                    error!(
                        "Couldn't serialize the tile set! Warning! Your data may be lost! {}",
                        e
                    )
                }
            }

            ui.same_line_with_spacing(0.0, 25.0);
            if imgui_utility::sized_button(ui, &im_str!("Revert##{}", uid)) {
                if let Ok(sprite_ingame_data) = serialization_util::sprites::load_sprite(
                    sprite_data.sprite_name,
                    sprite_data.texture_page.unwrap(),
                ) {
                    dirty = true;
                    *sprite_data = sprite_ingame_data;
                }
            }

            // Update dirty sprites to sprite components
            if dirty {
                for sprite_comp in sprites.iter_mut() {
                    if let Some(comp_sprite_data) = &mut sprite_comp.inner_mut().sprite_data {
                        if comp_sprite_data.sprite_name == sprite_data.sprite_name {
                            *comp_sprite_data = sprite_data.clone();
                        }
                    }
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
        .size(Vec2::new(290.0, 400.0).into(), imgui::Condition::FirstUseEver)
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

            let mut current_editing_tile = if let EditingMode::Editing(Some(v), _) = this_tileset.editing_mode
            {
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
                    if bounding_boxes.len() == 0 {
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
                this_tileset.editing_mode = EditingMode::Editing(Some(current_editing_tile), vec![]);
            }

            // Serialize, Deserialize
            let serialize_label = im_str!("Serialize##{}", unique_id);
            if imgui_utility::sized_button(ui, &serialize_label) {
                if let Err(e) = serialization_util::tilesets::serialize_tileset(this_tileset.clone()) {
                    error!(
                        "Couldn't serialize the tile set! Warning! Your data may be lost! {}",
                        e
                    )
                }
            }

            ui.same_line_with_spacing(0.0, 25.0);
            if imgui_utility::sized_button(ui, &im_str!("Revert##{}", unique_id)) {
                this_tileset.revert = true;

                if let Ok(Some(reverted_tset)) = serialization_util::tilesets::load_tileset(this_tileset.name)
                {
                    *this_tileset = reverted_tset;
                }
            }
        }

        window.end(ui);
    }

    close
}

pub fn game_config_editor(config: &mut game_config::Config, ui_handler: &mut UiHandler<'_>) -> bool {
    let mut close = true;
    let ui: &mut Ui<'_> = &mut ui_handler.ui;
    let game_config_window = imgui::Window::new(imgui::im_str!("Game Config Editor"))
        .size(Vec2::new(290.0, 400.0).into(), imgui::Condition::FirstUseEver)
        .opened(&mut close);

    if let Some(window) = game_config_window.begin(ui) {
        let uid = "game_config";

        config.window_size.inspector(ui, &im_str!("Window Size##{}", uid));

        // Serialize
        if imgui_utility::sized_button(ui, &im_str!("Serialize##{}", uid)) {
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

pub fn prefab_entity_viewer(resources: &mut ResourcesDatabase, ui_handler: &mut UiHandler<'_>) -> bool {
    let ui: &mut Ui<'_> = &mut ui_handler.ui;

    let mut open = true;
    let mut prefab_to_clone: Option<uuid::Uuid> = None;
    let mut prefab_to_delete: Option<uuid::Uuid> = None;

    let prefab_editor = imgui::Window::new(imgui::im_str!("Prefab Inspector"))
        .size(Vec2::new(290.0, 400.0).into(), imgui::Condition::FirstUseEver)
        .opened(&mut open);

    if let Some(window) = prefab_editor.begin(ui) {
        for prefab in resources.prefabs.values_mut() {
            // ENTITY ELEMENTS:
            display_prefab_id(prefab, ui);

            // BUTTONS
            let mut size = {
                let (size, pressed) =
                    imgui_utility::sized_button_padding(ui, &im_str!("Inspect##{}", prefab.id), Vec2::ZERO);
                if pressed {
                    // Store or Remove it...
                    if ui_handler.stored_prefabs.contains(&prefab.id) {
                        if let Some(position) = ui_handler.stored_prefabs.iter().position(|x| *x == prefab.id)
                        {
                            ui_handler.stored_prefabs.swap_remove(position);
                        }
                    } else {
                        ui_handler.stored_prefabs.push(prefab.id);
                    }
                }
                size
            };
            ui.same_line_with_spacing(0.0, 10.0);

            // CLONE
            size += {
                let (size, pressed) =
                    imgui_utility::sized_button_padding(ui, &im_str!("Clone##{}", prefab.id), Vec2::ZERO);
                if pressed {
                    prefab_to_clone = Some(prefab.id.clone());
                }

                size
            };
            ui.same_line_with_spacing(0.0, 12.0);

            // DELETE
            size += {
                let (_, pressed) =
                    imgui_utility::sized_button_padding(ui, &im_str!("Delete##{}", prefab.id), Vec2::ZERO);
                if pressed {
                    prefab_to_delete = Some(prefab.id.clone());
                }
                size
            };

            ui.separator();
        }

        if let Some(original) = prefab_to_clone {
            let mut original: SerializedEntity = resources.prefabs.get(&original).unwrap().clone();
            original.id = uuid::Uuid::new_v4();

            resources.prefabs.insert(original.id.clone(), original);
        }

        if let Some(id) = prefab_to_delete {
            resources.prefabs.remove(&id);
        }
        window.end(ui);
    }

    open
}

// @techdebt this doesn't handle changing prefab names!
fn display_prefab_id(prefab: &mut SerializedEntity, ui: &Ui<'_>) {
    if let Some((name, _)) = &mut prefab.name {
        name.inspect(
            ui,
            NameInspectorParameters {
                depth: 0,
                has_children: false,
                is_prefab: true,
                being_inspected: false,
            },
            &prefab.id.to_string(),
        );
    } else {
        ui.label_text(imgui::im_str!("Prefab ID"), &im_str!("{}", prefab.id));

        if imgui_utility::sized_button(ui, &im_str!("Name Entity##{:?}", prefab.id)) {
            let name = Name::new(&format!("Entity ID {}", prefab.id));

            prefab.name = Some((name, true));
        }
    }
}
