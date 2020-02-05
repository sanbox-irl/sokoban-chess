use super::{
    component_utils::RawComponent, imgui_system, Component, ComponentBounds, ComponentList, Entity,
    InspectorParameters, SerializableEntityReference, SerializationData, Transform,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct GraphNode {
    pub children: Option<Vec<SerializableEntityReference>>,
}

impl GraphNode {
    #[allow(dead_code)]
    pub fn specific_entity_inspector(
        &mut self,
        entity_id: Entity,
        ip: InspectorParameters<'_, '_>,
        serializations: &ComponentList<SerializationData>,
        transforms: &mut ComponentList<Transform>,
    ) {
        if let Some(our_children) = &self.children {
            for this_child in our_children {
                if let Some(this_child_target) = this_child.target {
                    ip.ui
                        .bullet_text(&imgui::im_str!("{}##{}", this_child_target, ip.uid));
                } else {
                    ip.ui.bullet_text(imgui::im_str!("Blank Child!"));
                }
            }
        } else {
            ip.ui.text("None");
        }

        if let Some(new_child) =
            imgui_system::select_entity("Add Child", ip.uid, ip.ui, ip.entities, ip.entity_names)
        {
            self.add_child(Some(entity_id), new_child, transforms, serializations);
        }
    }

    /// Use this to add a child to a GraphNode. It handles calling all subordinate
    /// functions in the Transform and all other classes. It also makes a RawComponent
    /// for you, so RC never needs to be seen outside this sub-folder.
    ///
    /// Use this for most child operations.
    /// Use `add_child_directly` if you want to use the transform directly:
    /// `add_child_directly`should be used for the RootNode or when iterating over transforms;
    /// otherwise, prefer `add_child`
    pub fn add_child(
        &mut self,
        my_entity_id: Option<Entity>,
        new_child: Entity,
        transforms: &mut ComponentList<Transform>,
        serializations: &ComponentList<SerializationData>,
    ) {
        // If they have a transform, work on it.
        // It is possible to have a child without a transform, if that child is
        // essentially just a folder.
        if let Some(trans) = transforms.get_mut(&new_child) {
            let id = trans.entity_id();
            trans.inner_mut().set_new_parent(
                id,
                RawComponent {
                    entity: my_entity_id,
                    graph_node: &mut *self,
                },
            );
        }

        // Make the reference to the new_child...
        let new_child_reference =
            SerializableEntityReference::into_reference(Some(new_child), serializations);

        match &mut self.children {
            Some(children) => children.push(new_child_reference),
            None => self.children = Some(vec![new_child_reference]),
        }
    }

    /// Use this function to add a child to a GraphNode. Prefer to use `add_child` over
    /// `add_child_directly` when possible. This function is made available for iterations
    /// especially for the Root Node.
    pub fn add_child_directly(
        &mut self,
        my_entity_id: Option<Entity>,
        transform: &mut Component<Transform>,
        serializations: &ComponentList<SerializationData>,
    ) {
        let id = transform.entity_id();
        transform.inner_mut().set_new_parent(
            id,
            RawComponent {
                entity: my_entity_id,
                graph_node: &mut *self,
            },
        );

        assert_ne!(my_entity_id, Some(id));

        // Make the reference to the new_child...
        let new_child_reference =
            SerializableEntityReference::into_reference(Some(id), serializations);

        match &mut self.children {
            Some(children) => children.push(new_child_reference),
            None => self.children = Some(vec![new_child_reference]),
        }
    }
}

impl ComponentBounds for GraphNode {
    fn entity_inspector(&mut self, _ip: InspectorParameters<'_, '_>) {
        unimplemented!();
    }
}
