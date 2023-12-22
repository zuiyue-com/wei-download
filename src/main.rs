use serde_json::{Value, json};

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

            send(body);
        },
        "list" => {
            let body = json!({
                "jsonrpc":"2.0",
                "method":"aria2.tellActive",
                "id": id,
                "params":[
                    token(),
                    [
                        "gid", "status", "bittorrent"
                    ]
                ]
            });

            send(body);
        },
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
        },
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
        // "list" => {
        //     let body = json!({
        //         "jsonrpc":"2.0",
        //         "method":"aria2.tellActive",
        //         "id":"abc",
        //         "params":[
        //             token(),
        //             [
        //                 "gid","totalLength","completedLength",
        //                 "uploadSpeed","downloadSpeed","connections",
        //                 "numSeeders","seeder","status",
        //                 "errorCode","verifiedLength","verifyIntegrityPending"
        //             ]
        //         ]
        //     });
        //     send(body);
        // },
        "quit" => {
            
        }
        "help" => {
            help();
        }
        _ => {
            
        }
    }

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

fn token() -> String {
    "token:abc123".to_string()
}

fn url() -> String {
    "http://localhost:6800/jsonrpc".to_string()
}

fn help() {
    print!("{}", json!({
        "code": 200,
        "message": "success",
        "data": "add,get,resume,del,list"
    }).to_string());
}

fn success(data: Value) {
    print!("{}", json!({
        "code": 200,
        "message": "success",
        "data": data
    }).to_string());
}

fn error(message: String) {
    print!("{}", json!({
        "code": 400,
        "message": message
    }).to_string());
}