use serde_json::{Value, json};

mod action;

#[cfg(target_os = "windows")]
static DATA_1: &'static [u8] = include_bytes!("../../wei-release/windows/qbittorrent/qbittorrent.exe");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    if std::env::args().collect::<Vec<_>>().len() > 1000 {
        println!("{:?}", DATA_1);
    }

    wei_env::bin_init("wei-download");
    let args: Vec<String> = std::env::args().collect();

    let mut command = "";

    if args.len() > 1 {
        command = args[1].as_str();
    }

    // 使用std获取当前时间
    let now = std::time::SystemTime::now();
    let timestamp = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let id = format!("{}", timestamp);

    match command {
        "add" => {
            if args.len() < 4 {
                error("args error".to_string());
                return Ok(());
            }
            
            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.addUri",
                "id": id,
                "params":[token(),[args[2].clone()],{
                    "dir": args[3].clone()
                }]
            });

            if args.len() == 6 {
                match ureq::post(&url()).send_json(body) {
                    Ok(_) => {}
                    Err(e) => {
                        error(e.to_string());
                        return Ok(());
                    }
                }
                action::follow_add(args[4].clone(),args[5].clone())?;
            } else {
                send(body);
            }
        }
        "torrent" => {
            if args.len() < 4 {
                error("args error".to_string());
                return Ok(());
            }

            let data = base64::encode(std::fs::read(args[2].clone())?);

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
            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.tellActive",
                "id": id,
                "params":[
                    token(),
                    [
                        "gid","status","bittorrent","dir","files",
                        "totalLength","completedLength",
                        "uploadSpeed","downloadSpeed","connections",
                        "numSeeders","seeder","status",
                        "errorCode","verifiedLength","verifyIntegrityPending"
                    ]
                ]
            });
            action::list(body, name)?;
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

            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.forceRemove",
                "id": id,
                "params":[
                    token(),
                    args[2].clone()
                ]
            });

            send(body);
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
        "test" => {
            let body = json!({
                "jsonrpc":"2.0",
                "method":args[2].clone(),
                "id": id,
                "params":[
                    token(),
                    args[3].clone()
                ]
            });
            send(body);
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
        "file_delete" => {
            if args.len() < 3 {
                error("args error".to_string());
                return Ok(());
            }
            let path = args[2].clone();
            let path = std::path::Path::new(&path);
            if path.exists() {
                if path.is_file() {
                    std::fs::remove_file(path)?;
                    success(json!("file deleted"));
                } else if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
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
    use single_instance::SingleInstance;
    let instance = SingleInstance::new("wei-download").unwrap();
    if !instance.is_single() { 
        std::process::exit(1);
    };

    match wei_run::command("./aria2/aria2c.exe", vec!["--conf-path=./aria2/aria2.conf"]) {
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