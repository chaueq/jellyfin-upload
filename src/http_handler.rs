use std::{fs::{File, OpenOptions}, io::{ Read, Write}, os::unix::fs, thread::sleep, time::{Duration, Instant}};


use crate::{config::{Config, ProgramFile}, http_server::{HttpRequest, HttpResponse}};

pub(crate) const BUFFER_SIZE: usize = 10485760 * 2; //20 MB
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(600); //10 mins

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
                        Ok(_) => {HttpResponse::normal(content, req.stream)}
                        Err(_) => {HttpResponse::minimal(500, req.stream)}
                    }
                    
                }
                Err(_) => {
                    HttpResponse::minimal(404, req.stream)
                }
            }

        }
        None => {HttpResponse::minimal(404, req.stream)}
    }
}

pub fn upload_file(mut req: HttpRequest, config: &Config) -> HttpResponse {
    req.stream.set_nonblocking(true).unwrap();
    req.stream.set_nodelay(true).unwrap();
    if let Some(len) = req.headers.get("content-length") {
        if let Ok(len) = len.parse::<usize>() {
            if let Some(folder) = req.headers.get("x-folder") {
                let folder = folder.replace("/", "");
                if let Some(filename) = req.headers.get("x-filename") {
                    let filename = filename.replace("/", "");
                    let path = config.get_path(ProgramFile::Content) + &folder + "/" + &filename;
                    match OpenOptions::new().create_new(true).write(true).open(&path) {
                        Ok(mut file) => {
                            let transfer_start = Instant::now();
                            let mut buf = [0u8; BUFFER_SIZE];
                            let mut bytes_read: usize = 0;
                            loop {
                                match req.stream.read(&mut buf) {
                                    Ok(bytes) => {
                                        if bytes == 0 {
                                            break;
                                        }
                                        bytes_read += bytes;
                                        if file.write_all(&mut buf[..bytes]).is_err() {
                                            println!("ERROR: Failed while writing to {}", path);
                                            return HttpResponse::minimal(500, req.stream);
                                        }
                                        if bytes_read == len {
                                            break;
                                        }
                                    }
                                    Err(_) => {
                                        if bytes_read == len {
                                            break;
                                        }
                                        else if transfer_start.elapsed() > UPLOAD_TIMEOUT {
                                            break;
                                        }
                                        else {
                                            sleep(Duration::from_millis(1));
                                        }
                                    }
                                }
                            } 
                            println!("{}\t{}/{}", path, bytes_read, len);
                            let _ = fs::chown(path, Some(config.get_uid()), Some(config.get_gid()));
                            if bytes_read == len {
                                return HttpResponse::minimal(204, req.stream);
                            }
                        }
                        Err(_) => {
                            println!("ERROR: Failed to open file {}", path);
                            return HttpResponse::minimal(500, req.stream);
                        }
                    }
                }
            }
        }
    }
    
    HttpResponse::minimal(400, req.stream)
}