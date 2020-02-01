use super::{physics_components::BoundingBox, ComponentList, DrawRectangle, Sprite, Ecs};

pub fn cross_cutting_system(ecs: &mut Ecs) {
    bounding_box_and_sprite(
        &mut ecs.component_database.bounding_boxes,
        &ecs.component_database.sprites,
    );

    draw_rectangle_and_bounding_box(
        &mut ecs.component_database.draw_rectangles,
        &ecs.component_database.bounding_boxes,
    );
}

fn bounding_box_and_sprite(bbs: &mut ComponentList<BoundingBox>, sprites: &ComponentList<Sprite>) {
    for this_bb in bbs.iter_mut() {
        let this_entity_id = this_bb.entity_id.clone();
        let this_bounding_box: &mut BoundingBox = this_bb.inner_mut();

        if this_bounding_box.bind_to_sprite {
            if let Some(this_sprite) = sprites.get(&this_entity_id) {
                if let Some(new_rect) = BoundingBox::rect_from_sprite(this_sprite) {
                    this_bounding_box.rect = new_rect;
                }
            } else {
                this_bounding_box.bind_to_sprite = false;
            }
        }
    }
}

fn draw_rectangle_and_bounding_box(
    draw_rects: &mut ComponentList<DrawRectangle>,
    bbs: &ComponentList<BoundingBox>,
) {
    for this_draw_rect in draw_rects.iter_mut() {
        let this_entity_id = this_draw_rect.entity_id.clone();
        let this_draw_rect_inner: &mut DrawRectangle = this_draw_rect.inner_mut();

        if this_draw_rect_inner.bind_to_bounding_box {
            if let Some(this_bb) = bbs.get(&this_entity_id) {
                this_draw_rect_inner.rect = this_bb.inner().rect;
            } else {
                this_draw_rect_inner.bind_to_bounding_box = false;
            }
        }
    }
}
