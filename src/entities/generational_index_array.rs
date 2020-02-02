use super::{GenerationalIndex, GenerationalIndexValue};

#[derive(Serialize, Deserialize, Default, Debug)]
struct ArrayEntry<T: GenerationalIndexValue> {
    value: T,
    generation: u64,
}

// An array from GenerationalIndex to some Value T.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GenerationalIndexArray<T: GenerationalIndexValue>(Vec<Option<ArrayEntry<T>>>);

impl<T: GenerationalIndexValue> GenerationalIndexArray<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns a mutable Iterator over the internal members of the Vec.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut().flat_map(|opt_ent| {
            let value = &mut opt_ent.as_mut()?.value;
            if value.is_active() {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Returns an immutable Iterator over the internal members of the Vec.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flat_map(|opt_ent| {
            let value = &opt_ent.as_ref()?.value;
            if value.is_active() {
                Some(value)
            } else {
                None
            }
        })
    }

    /// Set the value for some generational index.  May overwrite past generation
    /// values.
    pub fn set(&mut self, index: &GenerationalIndex, value: T) {
        self.0[index.index] = Some(ArrayEntry {
            value,
            generation: index.generation,
        });
    }

    /// Adds a new component for a new entity to the end of the List. Don't use this
    /// very commonly -- this is for adding new entities, essentially.
    pub fn expand_list(&mut self) {
        self.0.push(None);
    }

    /// Unsets the value for some generational index. Returns true if succesfully
    /// unset.
    pub fn unset(&mut self, index: &GenerationalIndex) -> bool {
        let ret = &self.0[index.index];
        if let Some(ret) = ret {
            if ret.generation == index.generation {
                self.0[index.index] = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // Gets an immutable reference to the contained value, if it exists.
    pub fn get(&self, index: &GenerationalIndex) -> Option<&T> {
        let ret = &self.0[index.index];
        if let Some(ret) = ret {
            if ret.generation == index.generation {
                Some(&ret.value)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets a mutable reference to the contained value, if it exists.
    pub fn get_mut(&mut self, index: &GenerationalIndex) -> Option<&mut T> {
        let ret = &mut self.0[index.index];
        if let Some(ret) = ret {
            if ret.generation == index.generation {
                Some(&mut ret.value)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<T: Default + typename::TypeName + GenerationalIndexValue> GenerationalIndexArray<T> {
    /// Gets a mutable reference to the contained if it exists.
    /// Otherwise, it creates the contained using default and returns
    /// a mutable reference to that.
    pub fn get_mut_or_default(&mut self, index: &GenerationalIndex) -> &mut T {
        if self.get_mut(index).is_none() {
            error!(
                "No {} for {} with get_mut_or_default. Generating component...",
                T::type_name(),
                index,
            );
            self.set(index, T::default());
        }

        self.get_mut(index).unwrap()
    }

    /// Gets an immutable reference to the contained if it exists.
    /// Otherwise, it creates the contained using default and returns
    /// an immutable reference to that. This is **slower** than just
    /// `get`, so use that if you can help it.
    pub fn get_or_default(&mut self, index: &GenerationalIndex) -> &T {
        if self.get(index).is_none() {
            error!(
                "No {} for {} with get_mut. Generating component...",
                T::type_name(),
                index
            );

            self.set(index, T::default());
        }

        self.get(index).unwrap()
    }
}
