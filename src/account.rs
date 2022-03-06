use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub type Accounts = HashMap<u16, Account>;

#[derive(Default)]
pub struct Account {
    client_id: u16,
    pub available: f32,
    pub held: f32,
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            ..Default::default()
        }
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {:.4}, {:.4}, {:.4}, {}",
            self.client_id,
            self.available,
            self.held,
            self.available + self.held,
            self.locked,
        )
    }
}
