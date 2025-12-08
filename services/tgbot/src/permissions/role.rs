use teloxide::types::User;

use crate::models::Role;


pub fn get_user_role(user: &User, admin_ids: &[i64]) -> Role {
    let user_id = user.id.0 as i64;
    if admin_ids.contains(&user_id) {
        Role::Admin
    } else {
        Role::Guest
    }
}
