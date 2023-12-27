use serde::{ Serialize, Deserialize };

#[derive(Copy, Clone)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Player {
    pub id: u64,
}
impl Player {
    pub fn new(id: u64) -> Self {
        Self {
            id: id,
        }
    }
}
