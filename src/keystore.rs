use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}, time::{SystemTime, UNIX_EPOCH}};

use sha3::{Sha3_512, Digest};

pub(crate) const AUTH_HEADER: &str = "x-apikey";

#[derive(Clone)]
pub struct Keystore {
    api_keys: HashMap<[u8; 64], KeyInfo>
}

impl Keystore {
    pub fn new() -> Self {
        Self {
            api_keys: HashMap::new()
        }
    }

    pub fn from_file(file_path: &str) -> Self {
        let mut store = Self::new();
        let file = match File::open(&file_path) {
            Ok(f) => {f},
            Err(_) => {
                File::create(&file_path).unwrap();
                File::open(&file_path).unwrap()
            }
        };
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            store.add_api_key(line);
        }
        store
    }

    fn add_api_key(&mut self, data: String) {
        let parts: Vec<_> = data.split('|').collect();
        if parts.len() >= 4 && parts[0].len() == 128 {
            let value = {
                let mut arr: [u8; 64] = [0;64];
                for i in 0..64 {
                    let hex = &parts[0][2*i..2*i+2];
                    arr[i] = u8::from_str_radix(&hex, 16).unwrap();
                }
                arr
            };
            let expiry = parts[1].parse::<usize>().unwrap();
            let name = parts[2].to_string();

            let mut info = KeyInfo::new(expiry, name);
            info.read_permissions(parts[3]);
            println!("Loaded key: {:?}", info);
            self.api_keys.insert(value, info);
        }
    }

    pub fn authorize(&self, key: &String, perm: Permission) -> AuthResult {
        let mut hasher = Sha3_512::new();
        hasher.update(key.clone());
        let hash: [u8; 64] = hasher.finalize().into();
        match self.api_keys.get(&hash) {
            Some(key_info) => {
                if key_info.is_expired() {
                    println!("{} failed to authorize due to expired key", key_info.get_name());
                    false
                }
                else if key_info.permissions.contains(&perm) {
                    println!("{} authorized succesfully", key_info.get_name());
                    true
                }
                else {
                    println!("{} failed to authorize due to insufficient permissions", key_info.get_name());
                    false
                }
            }
            None => {
                false
            }
        }

    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Permission {
    Upload
}

#[derive(Debug, Clone)]
struct KeyInfo {
    expiry: usize,
    name: String,
    permissions: Vec<Permission>
}

impl KeyInfo {
    pub fn new(expiry: usize, name: String) -> Self {
        Self {
            expiry,
            name,
            permissions: Vec::new()
        }
    }

    pub fn is_expired(&self) -> bool {
        if self.expiry == 0 {
            return false;
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        now > self.expiry
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn read_permissions(&mut self, text: &str) {
        for p in text.split(',') {
            if let Some(perm) = match p {
                "Upload" => {Some(Permission::Upload)}
                _ => {None}
            } {
                if !self.permissions.contains(&perm) {
                    self.permissions.push(perm);
                }
            }
        }
    }
}

pub type AuthResult = bool;