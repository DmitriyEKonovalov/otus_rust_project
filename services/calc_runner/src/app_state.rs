use crate::storage::SharedStorage;

#[derive(Clone, Debug)]
pub struct AppState { 
    pub storage: SharedStorage 
}