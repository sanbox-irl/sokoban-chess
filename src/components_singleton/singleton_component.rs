use super::{Component, ComponentBounds, ComponentList, Marker, SingletonDatabase};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingletonComponent<T: ComponentBounds> {
    marker: Marker,
    inner: T,
}

impl<T: ComponentBounds> SingletonComponent<T> {
    pub fn new(marker: Marker, inner: T) -> Self {
        Self { marker, inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn find_component_on_list<'a, SC: ComponentBounds, C: ComponentBounds>(
        singleton_component: &SingletonComponent<SC>,
        singleton_database: &SingletonDatabase,
        comp_list: &'a ComponentList<C>,
    ) -> Option<&'a Component<C>> {
        comp_list.get(
            &singleton_database
                .get_associated_entity(singleton_component)
                .unwrap(),
        )
    }

    pub fn marker(&self) -> Marker {
        self.marker
    }
}
