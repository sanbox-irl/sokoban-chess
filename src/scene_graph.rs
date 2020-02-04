use super::{
    Component, ComponentList, Entity, GraphNode, Name, NameInspectorParameters, PrefabMarker,
    SerializationData, Transform, Vec2,
};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref ROOT_NODES: Mutex<GraphNode> = Mutex::new(GraphNode {
        children: Some(vec![])
    });
}

pub fn add_to_scene_graph(
    transform_c: &mut Component<Transform>,
    serializations: &ComponentList<SerializationData>,
) {
    ROOT_NODES
        .lock()
        .unwrap()
        .add_child_directly(None, transform_c, serializations);
}

pub fn build_flat(
    transforms: &mut ComponentList<Transform>,
    serializations: &ComponentList<SerializationData>,
) {
    let mut root_nodes = ROOT_NODES.lock().unwrap();
    root_nodes.children.as_mut().unwrap().clear();

    for transform_c in transforms.iter_mut() {
        if transform_c.inner_mut().parent_exists() == false {
            root_nodes.add_child_directly(None, transform_c, serializations);
            transform_c.inner_mut().update_world_position(Vec2::ZERO);
        }
    }
}

pub fn walk_graph(transforms: &mut ComponentList<Transform>, nodes: &ComponentList<GraphNode>) {
    let root_nodes = ROOT_NODES.lock().unwrap();

    if let Some(root_nodes) = &root_nodes.children {
        for secondary_node in root_nodes {
            if let Some(target) = &secondary_node.target {
                walk_node(target, transforms, nodes, Vec2::ZERO);
            }
        }
    }
}

fn walk_node(
    entity: &Entity,
    transforms: &mut ComponentList<Transform>,
    nodes: &ComponentList<GraphNode>,
    last_world_position: Vec2,
) {
    let new_world_position = if let Some(transform) = transforms.get_mut(entity) {
        transform
            .inner_mut()
            .update_world_position(last_world_position)
    } else {
        last_world_position
    };

    if let Some(this_node) = nodes.get(entity) {
        if let Some(children) = &this_node.inner().children {
            for child in children {
                if let Some(target) = &child.target {
                    walk_node(target, transforms, nodes, new_world_position);
                }
            }
        }
    }
}

type GraphInspectorLambda<'a> = &'a mut dyn FnMut(
    &Entity,
    &mut ComponentList<Name>,
    &ComponentList<SerializationData>,
    NameInspectorParameters,
) -> bool;

pub fn walk_graph_inspect(
    transforms: &ComponentList<Transform>,
    nodes: &ComponentList<GraphNode>,
    names: &mut ComponentList<Name>,
    prefabs: &ComponentList<PrefabMarker>,
    serialization_data: &ComponentList<SerializationData>,
    f: GraphInspectorLambda<'_>,
) {
    let root_nodes = ROOT_NODES.lock().unwrap();

    if let Some(root_nodes) = &root_nodes.children {
        for secondary_node in root_nodes {
            if let Some(target) = &secondary_node.target {
                walk_node_inspect(
                    target,
                    transforms,
                    nodes,
                    names,
                    prefabs,
                    serialization_data,
                    0,
                    f,
                );
            }
        }
    }
}

fn walk_node_inspect(
    entity: &Entity,
    transforms: &ComponentList<Transform>,
    nodes: &ComponentList<GraphNode>,
    names: &mut ComponentList<Name>,
    prefabs: &ComponentList<PrefabMarker>,
    serialization_data: &ComponentList<SerializationData>,
    depth: usize,
    f: GraphInspectorLambda<'_>,
) {
    let mut show_children = true;
    let has_children: bool = nodes
        .get(entity)
        .map(|n| {
            if let Some(children) = &n.inner().children {
                children.len() > 0
            } else {
                false
            }
        })
        .unwrap_or_default();

    if transforms.get(entity).is_some() {
        show_children = f(
            entity,
            names,
            serialization_data,
            NameInspectorParameters {
                depth,
                has_children,
                is_prefab: prefabs.get(entity).is_some(),
                being_inspected: false,
            },
        );
    }

    if show_children {
        if let Some(this_node) = nodes.get(entity) {
            if let Some(children) = &this_node.inner().children {
                for child in children {
                    if let Some(target) = &child.target {
                        walk_node_inspect(
                            target,
                            transforms,
                            nodes,
                            names,
                            prefabs,
                            serialization_data,
                            depth + 1,
                            f,
                        );
                    }
                }
            }
        }
    }
}
