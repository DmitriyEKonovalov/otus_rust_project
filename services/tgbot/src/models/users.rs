use serde::{Deserialize, Serialize};
use serde_json;

use crate::models::roles::Role;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub user_name: String,
    pub user_groups: Role,
}
