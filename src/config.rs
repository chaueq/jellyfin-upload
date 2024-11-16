use std::{env, fs::File, io::Read};

const CONTENT_FOLDER: &str = "/var/lib/jellyfin-upload/";

#[derive(Debug, Clone)]
pub struct Config {
    net_address: String,
    directory: String,
    content: String,
    uid: u32,
    gid: u32
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

        Self {
            net_address,
            directory,
            content,
            uid,
            gid
        }
    }

    pub fn get_path(&self, file: ProgramFile) -> String {
        match file {
            ProgramFile::IndexCSS => {CONTENT_FOLDER.to_owned() + "index.css"}
            ProgramFile::IndexJS => {CONTENT_FOLDER.to_owned() + "index.js"}
            ProgramFile::IndexHTML => {CONTENT_FOLDER.to_owned() + "index.html"}
            ProgramFile::Keystore => {self.directory.clone() + "keystore.csv"}
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
}

pub enum ProgramFile {
    IndexHTML,
    IndexJS,
    IndexCSS,
    Keystore,
    Content
}