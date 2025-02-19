use std::{collections::HashMap, fs::{self, File, OpenOptions}, io::{ Read, Write}, os::unix::fs::chown, path::Path};

use serde_json::json;

use crate::{config::{Config, ProgramFile}, http_server::{HttpMethod, HttpRequest, HttpResponse}};

pub fn serve_file(req: HttpRequest, config: &Config) -> HttpResponse {
    match {
        let p = req.path.to_lowercase();
        if p == "/index.html" || p == "/" {
            Some(ProgramFile::IndexHTML)
        }
        else if p == "/index.css" {
            Some(ProgramFile::IndexCSS)
        }
        else if p == "/index.js" {
            Some(ProgramFile::IndexJS)
        }
        else {
            None
        }
    } {
        Some(f) => {
            let path = config.get_path(f);
            let mut headers: HashMap<String, String> = HashMap::new();
            headers.insert("content-type".to_string(), (
                    if path.ends_with(".html") {"text/html"}
                    else if path.ends_with(".css") {"text/css"}
                    else if path.ends_with(".js") {"application/javascript"}
                    else {"text/plain"}
                ).to_string() + "; charset=utf-8"
            );
            match File::open(path) {
                Ok(mut file) => {
                    let mut content = String::new();
                    match file.read_to_string(&mut content) {
                        Ok(_) => {HttpResponse {
                            status: 200,
                            headers,
                            body: Some(content),
                        }}
                        Err(_) => {HttpResponse::minimal(500)}
                    }
                    
                }
                Err(_) => {
                    HttpResponse::minimal(404)
                }
            }

        }
        None => {HttpResponse::minimal(404)}
    }
}

pub fn upload_file(req: HttpRequest, config: &Config) -> HttpResponse {
    if let Some(collection) = req.headers.get("x-collection") {
        let collection = collection.replace("/", "");
        if let Some(folder) = config.get_collection_folder(&collection) {
            if let Some(filename) = req.headers.get("x-filename") {
                let filename = filename.replace("/", "").trim().to_string();
                let path = {
                    let prefix = config.get_path(ProgramFile::Content) + &folder + "/";
                    if let Some(subfolder) = req.headers.get("x-foldername") {
                        let subfolder = subfolder.replace(".", "").trim().trim_matches('/').to_string();
                        let path = prefix.clone() + &subfolder + "/";
                        let path = Path::new(&path);
                        if !path.exists() {
                            if fs::create_dir_all(path).is_err() {
                                return HttpResponse::minimal(500);
                            }
                            if r_chown(&folder, &subfolder, &config).is_err() {
                                return HttpResponse::minimal(500);
                            }
                        }
                        else if !path.is_dir() {
                            return HttpResponse::minimal(406);
                        }
                        prefix + &subfolder + "/" + &filename
                    }
                    else {
                        prefix + &filename
                    }
                };
                match OpenOptions::new().create(true).truncate(req.method == HttpMethod::PUT).append(req.method == HttpMethod::POST).write(true).open(&path) {
                    Ok(mut file) => {
                        if let Some(body) = req.body {
                            match file.write_all(&body) {
                                Ok(_) => {
                                    drop(file);
                                    let _ = chown(path, Some(config.get_uid()), Some(config.get_gid()));
                                    return HttpResponse::minimal(204);
                                }
                                Err(_) => {return HttpResponse::minimal(500);}
                            }
                        }
                    }
                    Err(_) => {
                        println!("ERROR: Failed to open file {}", path);
                        return HttpResponse::minimal(500);
                    }
                }
            }
        }
    }
    
    HttpResponse::minimal(400)
}

pub fn collections(config: &Config) -> HttpResponse {
    let pairs = config.get_collections();
    HttpResponse::normal(json!(pairs).to_string())
}

pub fn space(req: HttpRequest, config: &Config) -> HttpResponse {
    if let Some(c) = req.headers.get("x-collection") {
        if let Some(path) = config.get_collection_folder(c) {
            if let Ok(bytes) = fs2::free_space(config.get_path(ProgramFile::Content) + path.as_str()) {
                return HttpResponse::normal(bytes.to_string());
            }
        }
    }
    HttpResponse::minimal(400)
}

fn r_chown(folder: &String, subfolder: &String, config: &Config) -> Result<(), std::io::Error> {
    let steps: Vec<&str> = subfolder.split('/').collect();
    let mut current = folder.clone().trim_end_matches('/').to_string();
    for step in steps {
        current += "/";
        current += step;
        let res = chown(&current, Some(config.get_uid()), Some(config.get_gid()));
        if res.is_err() {
            return res;
        }
    }
    Ok(())
}