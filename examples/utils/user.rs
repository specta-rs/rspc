use std::sync::Mutex;

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

static USER_MOCK_DB: OnceCell<Mutex<Vec<User>>> = OnceCell::new();

#[derive(Clone, Default, TS, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl User {
    pub async fn create(user: User) -> Self {
        USER_MOCK_DB
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .push(user.clone());
        user
    }

    pub async fn read(id: i32) -> Option<Self> {
        USER_MOCK_DB
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.id == id)
            .map(|v| v.clone())
    }

    pub async fn read_all() -> Vec<Self> {
        USER_MOCK_DB
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .clone()
    }

    pub async fn update(id: i32, user: User) -> Self {
        USER_MOCK_DB
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .iter_mut()
            .find(|u| u.id == id)
            .map(|u| *u = user.clone());
        user
    }

    pub async fn delete(id: i32) {
        USER_MOCK_DB
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .retain(|u| u.id != id);
    }
}
