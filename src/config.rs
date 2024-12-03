use std::{collections::HashMap, env, fs::File, io::{BufRead, BufReader, Read}};

const CONTENT_FOLDER: &str = "/var/lib/jellyfin-upload/";

#[derive(Debug, Clone)]
pub struct Config {
    net_address: String,
    directory: String,
    content: String,
    uid: u32,
    gid: u32,
    collections: HashMap<String, Collection>
}

impl Config {
    pub fn from_args() -> Self {
        let argv: Vec<String> = env::args().collect();
        let args: usize = argv.len();

        let net_address = {
            if args > 1 {
                argv[1].clone()
            }
            else {
                "0.0.0.0:80".to_string()
            }
        };
        let directory = {
            if args > 2 {
                let d = argv[2].clone();
                if d.as_bytes()[d.len()-1] != b'/' {
                    d + "/"
                }
                else {
                    d
                }
            }
            else {
                "/opt/jellyfin-upload/".to_string()
            }
        };
        let content = {
            if args > 3 {
                let d = argv[3].clone();
                if d.as_bytes()[d.len()-1] != b'/' {
                    d + "/"
                }
                else {
                    d
                }
            }
            else {
                "/media/".to_string()
            }
        };

        let (uid, gid) = {
            let mut result: (u32, u32) = (0,0);
            let mut file = File::open("/etc/passwd").unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            let lines: Vec<String> = content.lines().map(|x|x.to_string()).collect();
            for line in lines {
                let parts: Vec<&str> = line.split(':').collect();
                if parts[0] == "jellyfin" {
                    result = (
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap()
                    );
                    break;
                }
            }
            result
        };

        let collections = HashMap::new();

        let mut c = Self {
            net_address,
            directory,
            content,
            uid,
            gid,
            collections
        };

        c.update_collections();
        c
    }

    pub fn get_path(&self, file: ProgramFile) -> String {
        match file {
            ProgramFile::IndexCSS => {CONTENT_FOLDER.to_owned() + "index.css"}
            ProgramFile::IndexJS => {CONTENT_FOLDER.to_owned() + "index.js"}
            ProgramFile::IndexHTML => {CONTENT_FOLDER.to_owned() + "index.html"}
            ProgramFile::Keystore => {self.directory.clone() + "keystore.csv"}
            ProgramFile::Collections => {self.directory.clone() + "collections.csv"}
            ProgramFile::Content => {self.content.clone()}
        }
    }

    pub fn get_net_address(&self) -> String {
        self.net_address.clone()
    }

    pub fn get_gid(&self) -> u32 {
        self.gid
    }

    pub fn get_uid(&self) -> u32 {
        self.uid
    }

    pub fn get_collection_folder(&self, val: &String) -> Option<String> {
        if let Some(c) = self.collections.get(val) {
            return Some(c.path.clone());
        }
        None
    }

    pub fn update_collections(&mut self) {
        let mut file = match File::open(self.get_path(ProgramFile::Collections)) {
            Ok(f) => {f},
            Err(_) => {
                File::create(self.get_path(ProgramFile::Collections)).unwrap();
                File::open(self.get_path(ProgramFile::Collections)).unwrap()
            }
        };
        let mut cs: HashMap<String, Collection> = HashMap::new();
        let reader: BufReader<&mut File> = BufReader::new(&mut file);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.starts_with('#') {
                continue;
            }
            let parts: Vec<String> = line.split('|').map(|x|x.to_string()).collect();
            let c = Collection {
                display: parts[0].clone(),
                path: parts[2].clone()
            };
            cs.insert(parts[1].clone(), c);
        }
        self.collections = cs;
    }

    pub fn get_collections(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();
        for c in &self.collections {
            result.push((c.1.display.clone(), c.0.clone()));
        }
        result
    }
}

pub enum ProgramFile {
    IndexHTML,
    IndexJS,
    IndexCSS,
    Keystore,
    Content,
    Collections
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub(crate) display: String,
    pub(crate) path: String,
}