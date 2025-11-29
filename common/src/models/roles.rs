use crate::errors::BotError;
use crate::settings::{BotState, REDIS_USER_CALCS_PREFIX, REDIS_USER_PREFIX};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use teloxide::{prelude::*, types::Message};
use uuid::Uuid;



#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum Role {
    Admin = 1,
    Business = 2,
    Guest = 3,
}

impl Role {
    fn priority(&self) -> u8 {
        match self {
            Role::Admin => 1,
            Role::Business => 2,
            Role::Guest => 3,
        }
    }

    pub fn is_granted(&self, required: Role) -> bool {
        self.priority() <= required.priority()
    }
}
