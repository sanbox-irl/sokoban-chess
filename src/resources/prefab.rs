use super::SerializedEntity;
use uuid::Uuid;

/// Where the Key in the HashMap is the same as the MainID in the Prefab.
pub type PrefabMap = std::collections::HashMap<Uuid, Prefab>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Prefab {
    main_id: Uuid,
    pub members: Vec<SerializedEntity>,
}

impl Clone for Prefab {
    fn clone(&self) -> Self {
        // Make a new ID
        let new_main_id = Uuid::new_v4();

        // Find our Old Main and give it the new ID:
        let mut members = self.members.clone();

        // Iterate over all the others and give them new IDs:
        for member in members.iter_mut() {
            member.id = if member.id == self.main_id {
                new_main_id
            } else {
                Uuid::new_v4()
            }
        }

        Prefab { main_id: new_main_id, members }
    }
}

impl Prefab {
    pub fn new(main_id: Uuid, members: Vec<SerializedEntity>) -> Prefab {
        Prefab { main_id, members }
    }

    pub fn new_blank() -> Prefab {
        let uuid = Uuid::new_v4();
        Prefab {
            main_id: uuid,
            members: vec![SerializedEntity {
                id: uuid,
                ..Default::default()
            }],
        }
    }

    pub fn main_entity(&self) -> &SerializedEntity {
        self.members.iter().find(|p| p.id == self.main_id).unwrap()
    }

    pub fn main_entity_mut(&mut self) -> &mut SerializedEntity {
        let id = self.main_id;
        self.members
            .iter_mut()
            .find(|p| p.id == id)
            .unwrap()
    }

    pub fn main_id(&self) -> Uuid {
        self.main_id
    }
}
