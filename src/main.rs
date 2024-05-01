#[macro_use]
extern crate wei_log;

use serde_json::{Value, json};

mod action;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    wei_windows::init();

    wei_env::bin_init("wei-download");
    let args: Vec<String> = std::env::args().collect();

    let mut command = "";

    if args.len() > 1 {
        command = args[1].as_str();
    }

    // 使用std获取当前时间
    let id = id();

    match command {
        "add" => {
            if args.len() < 4 {
                error("args error".to_string());
                return Ok(());
            }

            let body;

            // 如果是种子链接，先下载种子文件，再添加种子文件
            if args[2].ends_with(".torrent") {
                info!("add torrent: {:?}", args.clone());
                // 使用ureq下载种子文件
                let res = match ureq::get(&args[2]).call() {
                    Ok(res) => {res}
                    Err(e) => {
                        error(e.to_string());
                        return Ok(());
                    }
                };

                let mut buffer = Vec::new();
                match res.into_reader().read_to_end(&mut buffer) {
                    Ok(c) => c,
                    Err(e) => {
                        error(e.to_string());
                        return Ok(());
                    }
                };

                let data = base64::encode(&buffer);

                body = json!({
                    "jsonrpc":"2.0",
                    "method":"aria2.addTorrent",
                    "id": id,
                    "params":[
                        token(),
                        data,
                        [],
                        {"dir": args[3].clone()}
                    ]
                });
            } else {
                body = json!({
                    "jsonrpc":"2.0",
                    "method":"aria2.addUri",
                    "id": id,
                    "params":[token(),[args[2].clone()],{
                        "dir": args[3].clone()
                    }]
                });
            }
            
            info!("add {:?}", args.clone());

            if args.len() == 5 { // 上报进度给服务器
                let data = match ureq::post(&url()).send_json(body) {
                    Ok(c) => {c}
                    Err(e) => {
                        error(e.to_string());
                        return Ok(());
                    }
                };

                let body: serde_json::Value = match data.into_json() {
                    Ok(c) => {c}
                    Err(e) => {
                        error(e.to_string());
                        return Ok(());
                    }
                };

                let body = match body["result"].as_str() {
                    Some(c) => {c}
                    None => {
                        error("result error".to_string());
                        return Ok(());
                    }
                };

                action::follow_add(args[4].clone(), body.into())?;
            } else {
                send(body);
            }
        }
        "torrent" => {
            if args.len() < 4 {
                error("args error".to_string());
                return Ok(());
            }

            let data = match std::fs::read(&args[2]) {
                Ok(data) => {data}
                Err(e) => {
                    error(e.to_string());
                    return Ok(());
                }
            };

            let data = base64::encode(data);

            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.addTorrent",
                "id": id,
                "params":[
                    token(),
                    data,
                    [],
                    {"dir": args[3].clone()}
                ]
            });
            send(body);
        }
        "list" => {
            let mut name = "".to_string();
            if args.len() > 2 {
                name = args[2].clone();
            }
            action::list(name)?;
        }
        "list_id" => {
            let mut id = "".to_string();
            if args.len() > 2 {
                id = args[2].clone();
            }
            action::list_id(id)?;
        }
        "list_all" => {
            action::list_all()?;
        }
        "stop_all" => {
            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.pauseAll",
                "id": id,
                "params":[
                    token()
                ]
            });
            send(body);
        }
        "stop" => {
            if args.len() < 3 {
                error("args error".to_string());
                return Ok(());
            }

            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.forcePause",
                "id": id,
                "params":[
                    token(),
                    args[2].clone()
                ]
            });

            send(body);
        }
        "delete" => {
            if args.len() < 3 {
                error("args error".to_string());
                return Ok(());
            }

            let data = wei_run::run("wei-download", vec!["list_all"])?;

            let data: Value = serde_json::from_str(&data).unwrap();
            let data = data["data"].as_object().unwrap();

            for (key, value) in data {
                if key == &args[2] {
                    let mut body = json!({
                        "jsonrpc":"2.0",
                        "method":"aria2.forceRemove",
                        "id": id,
                        "params":[
                            token(),
                            key
                        ]
                    });
                    send(body.clone());

                    body["method"] = "aria2.removeDownloadResult".into();
                    send(body);

                    let files = value["files"].as_array().unwrap();
                    for file in files {
                        let path = file["path"].as_str().unwrap();
                        match std::fs::remove_file(path) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                    }

                    // 如果保存的种子只包含文件，不包含文件夹，则会删除错误的文件
                    // let dir = value["dir"].as_str().unwrap();
                    // match std::fs::remove_dir_all(dir) {
                    //     Ok(_) => {}
                    //     Err(_) => {}
                    // };

                    return Ok(());
                }
            }
            error("not found".to_string());
        }
        "delete_name" => {
            if args.len() < 3 {
                error("args error".to_string());
                return Ok(());
            }

            let data = wei_run::run("wei-download", vec!["list_all"])?;

            let data: Value = serde_json::from_str(&data).unwrap();
            let data = data["data"].as_object().unwrap();

            for (key, value) in data {
                let name = match value["name"].as_str() {
                    Some(name) => name,
                    None => "".into()
                };
                if name == args[2] {
                    let body = json!({
                        "jsonrpc":"2.0",
                        "method":"aria2.removeDownloadResult",
                        "id": id,
                        "params":[
                            token(),
                            key
                        ]
                    });
                    match ureq::post(&url()).send_json(body) {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                }
            }
            success(json!("success"));
        }
        "delete_all" => {
            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.purgeDownloadResult",
                "id": id,
                "params":[
                    token()
                ]
            });
            send(body);
        }
        "set_location" => {
            if args.len() < 4 {
                error("args error".to_string());
                return Ok(());
            }

            let gid = &args[2];
            let dir_new = &args[3];
            
            let data = wei_run::run("wei-download", vec!["list_all"])?;
            let data: Value = serde_json::from_str(&data).unwrap();
            let data = data["data"].as_object().unwrap();
            let item;

            if data.get(gid).is_none() {
                error("not found".to_string());
                return Ok(());
            } else {
                item = data[gid].clone();
            }

            
            let completed_length = item["completed_length"].as_str().unwrap();
            let total_length = item["total_length"].as_str().unwrap();

            if completed_length != total_length {
                error("not completed".to_string());
                return Ok(());
            }

            let dir = item["dir"].as_str().unwrap();
            let files = item["files"].as_array().unwrap();
            for file in files {
                let path = file["path"].as_str().unwrap();
                let path_new = path.replace(dir, dir_new);

                let path_dir = std::path::Path::new(&path_new);
                // 获取path_new的上级目录
                let path_dir = path_dir.parent().unwrap();
                if !path_dir.exists() {
                    match std::fs::create_dir_all(path_dir) {
                        Ok(_) => {}
                        Err(err) => {
                            error(format!("create dir error: {}", err.to_string()));
                            return Ok(());
                        }
                    };
                }

                match std::fs::rename(path, path_new) {
                    Ok(_) => {}
                    Err(err) => {
                        error(format!("rename error: {}", err.to_string()));
                        return Ok(());
                    }
                };
            }

            wei_run::run("wei-download", vec!["delete", gid])?;

            // let info_hash = item["info_hash"].as_str().unwrap();
            // if info_hash != "" {
            //     let url = format!("magnet:?xt=urn:btih:{}", info_hash);
            //     wei_run::run("wei-download", vec!["add", &url, dir_new])?;
            // }

            success(json!("success"));
        }
        "resume" => {
            if args.len() < 3 {
                error("args error".to_string());
                return Ok(());
            }

            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.unpause",
                "id": id,
                "params":[
                    token(),
                    args[2].clone()
                ]
            });

            send(body);
        }
        "resume_all" => {
            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.unpauseAll",
                "id": id,
                "params":[
                    token()
                ]
            });
            send(body);
        }
        "quit" => {
            let body = json!({
                "jsonrpc":"2.0",
                "method": "aria2.shutdown",
                "id": id,
                "params":[
                    token()
                ]
            });
            send(body);
        }
        "check" => {
            let gid = &args[2];
            let data = wei_run::run("wei-download", vec!["list_all"])?;
            // 列出所有下载的文件，比对文件的大小，如果文件大小不一致，则重新下载
            let data: Value = serde_json::from_str(&data).unwrap();
            let data = data["data"].as_object().unwrap();
            let item;

            if data.get(gid).is_none() {
                error("not found".to_string());
                return Ok(());
            } else {
                item = data[gid].clone();
            }

            let files = item["files"].as_array().unwrap();
            for file in files {
                let path = file["path"].as_str().unwrap();
                let length = file["length"].as_str().unwrap();

                let file = std::fs::File::open(path)?;
                let metadata = file.metadata()?;
                let file_size = metadata.len();
                let file_size = format!("{}", file_size);
                if file_size != length {
                    success(json!({
                        "check": false,
                    }));
                    return Ok(());
                }
            }

            success(json!({
                "check": true,
            }));
        }
        "file_list" => {
            if args.len() < 3 {
                error("args error".to_string());
                return Ok(());
            }
            let path = args[2].clone();
            if std::path::Path::new(&path).exists() {
                let list = serde_json::from_str(&wei_hardware::get_file_info(path))?;
                success(list);
            } else {
                error("file not exists".to_string());
            }            
        }
        "file_list_with_timestamp" => {
            use std::time::{SystemTime, UNIX_EPOCH};

            let now = SystemTime::now();
            let since_the_epoch = now.duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let timestamp = since_the_epoch.as_secs();

            if args.len() < 3 {
                error("args error".to_string());
                return Ok(());
            }
            let path = args[2].clone();
            if std::path::Path::new(&path).exists() {
                let list: serde_json::Value = serde_json::from_str(&wei_hardware::get_file_info(path))?;
                
                let data = serde_json::json!({
                    "list": list,
                    "time": timestamp
                });
                success(data);
            } else {
                error("file not exists".to_string());
            }       
        }
        "file_delete" => {
            if args.len() < 3 {
                error("args error".to_string());
                return Ok(());
            }
            let path = args[2].clone();
            let path = std::path::Path::new(&path);
            if path.exists() {
                if path.is_file() {
                    match std::fs::remove_file(path) {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                    success(json!("file deleted"));
                } else if path.is_dir() {
                    match std::fs::remove_dir_all(path) {
                        Ok(_) => {}
                        Err(_) => {}
                    };
                    success(json!("dir deleted"));
                }
            } else {
                error("file not exists".to_string());
            }

        }
        "help" => {
            help();
        }
        _ => {
            start()?;
        }
    }

    Ok(())
}

fn start() -> Result<(), Box<dyn std::error::Error>> {
    let instance = wei_single::SingleInstance::new("wei-download").unwrap();
    if !instance.is_single() { 
        std::process::exit(1);
    };

    wei_run::kill("aria2c")?;

    // 判断文件是否存在 ./aria2/aria2.session, 如果不存在则创建
    let path = std::path::Path::new("./aria2/aria2.session");
    if !path.exists() {
        std::fs::File::create(&path)?;
    }

    #[cfg(target_os = "windows")]
    let aria2c = "./aria2/aria2c.exe";
    #[cfg(target_os = "linux")]
    let aria2c = "./aria2/aria2c";

    match wei_run::command(aria2c, vec!["--conf-path=./aria2/aria2.conf"]) {
        Ok(data) => {
            success(json!(data));
        }
        Err(e) => {
            error(e.to_string());
        }
    };
    Ok(())
}

fn send(body: Value) {
    match ureq::post(&url()).send_json(body) {
        Ok(res) => {
            let data = res.into_string().unwrap();
            let data:Value = serde_json::from_str(&data).unwrap();
            success(data);
        }
        Err(e) => {
            error(e.to_string());
        }
    }
}

pub fn token() -> String {
    "token:abc123".to_string()
}

pub fn id() -> String {
    let now = std::time::SystemTime::now();
    let timestamp = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    format!("{}", timestamp)
}

pub fn url() -> String {
    "http://localhost:6800/jsonrpc".to_string()
}

fn help() {
    print!("{}", json!({
        "code": 200,
        "message": "success",
        "data": "add,get,resume,del,list"
    }).to_string());
}

pub fn success(data: Value) {
    print!("{}", json!({
        "code": 200,
        "message": "success",
        "data": data
    }).to_string());
}

pub fn error(message: String) {
    print!("{}", json!({
        "code": 400,
        "message": message
    }).to_string());
}

