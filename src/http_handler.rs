use std::{fs::{File, OpenOptions}, io::{ Read, Write}, os::unix::fs::chown};

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
            match File::open(path) {
                Ok(mut file) => {
                    let mut content = String::new();
                    match file.read_to_string(&mut content) {
                        Ok(_) => {HttpResponse::normal(content)}
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
    if let Some(folder) = req.headers.get("x-folder") {
        let folder = folder.replace("/", "");
        if let Some(filename) = req.headers.get("x-filename") {
            let filename = filename.replace("/", "");
            let path = config.get_path(ProgramFile::Content) + &folder + "/" + &filename;
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
    
    HttpResponse::minimal(400)
}