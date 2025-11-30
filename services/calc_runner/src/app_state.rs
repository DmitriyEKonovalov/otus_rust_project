use crate::storage::Storage;

#[derive(Clone, Debug)]
pub struct AppState {
    pub storage: Storage,
}
