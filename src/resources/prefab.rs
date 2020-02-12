use super::SerializedEntity;
use uuid::Uuid;

/// Where the Key in the HashMap is the same as the MainID in the Prefab.
pub type PrefabMap = std::collections::HashMap<Uuid, Prefab>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Prefab {
    root_id: Uuid,
    valid: bool,
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
            member.id = if member.id == self.root_id {
                new_main_id
            } else {
                Uuid::new_v4()
            }
        }

        Prefab {
            root_id: new_main_id,
            members,
            valid: true,
        }
    }
}

impl Prefab {
    pub fn new(root_id: Uuid, members: Vec<SerializedEntity>) -> Prefab {
        Prefab {
            root_id,
            members,
            valid: true,
        }
    }

    pub fn new_blank() -> Prefab {
        let uuid = Uuid::new_v4();
        Prefab {
            root_id: uuid,
            members: vec![SerializedEntity {
                id: uuid,
                ..Default::default()
            }],
            valid: true,
        }
    }

    pub fn root_entity(&self) -> &SerializedEntity {
        self.members.iter().find(|p| p.id == self.root_id).unwrap()
    }

    pub fn root_entity_mut(&mut self) -> &mut SerializedEntity {
        let id = self.root_id;
        self.members.iter_mut().find(|p| p.id == id).unwrap()
    }

    pub fn root_id(&self) -> Uuid {
        self.root_id
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }
}
