// use std::sync::Arc;
// use teloxide::{prelude::*, types::Message};


// use crate::{
// pub fn commands_help(user: &User) -> String {
//     let mut commands = vec![START_DESC.to_string(), HELP_DESC.to_string()];
//     if user.user_groups.is_granted(Role::Business) {
//         commands.push(CALC_DESC.to_string());
//     }
//     if user.user_groups.is_granted(Role::Admin) {
//         commands.push(USERS_CALC_DESC.to_string());
//     }

//     format!(
//         GREETING_TEMPLATE,
//         user.user_name,
//         user.user_groups,
//         commands.join("\n")
//     )
// }
