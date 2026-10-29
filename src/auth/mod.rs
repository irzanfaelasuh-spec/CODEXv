use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub password_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AuthStore {
    pub accounts: Vec<Account>,
    pub active_user: Option<String>,
}

impl AuthStore {
    pub fn sign_up(&mut self, name: &str, password: &str) -> Result<(), String> {
        let name = name.trim();
        if name.len() < 3 {
            return Err("Name must contain at least 3 characters.".into());
        }
        if password.len() < 6 {
            return Err("Password must contain at least 6 characters.".into());
        }
        if self.accounts.iter().any(|a| a.name.eq_ignore_ascii_case(name)) {
            return Err("This name is already in use, please use another one.".into());
        }
        self.accounts.push(Account {
            name: name.to_owned(),
            password_hash: hash_password(name, password),
        });
        self.active_user = Some(name.to_owned());
        Ok(())
    }

    pub fn sign_in(&mut self, name: &str, password: &str) -> Result<(), String> {
        let Some(account) = self.accounts.iter().find(|a| a.name.eq_ignore_ascii_case(name)) else {
            return Err("We couldn't find that account.".into());
        };
        if account.password_hash != hash_password(&account.name, password) {
            return Err("Incorrect password. Please try again.".into());
        }
        self.active_user = Some(account.name.clone());
        Ok(())
    }

    pub fn logout(&mut self) {
        self.active_user = None;
    }
}

fn hash_password(name: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"VYRON-0.1.0::");
    hasher.update(name.as_bytes());
    hasher.update(b"::");
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}
