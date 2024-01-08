use serde_json::{Value, json};

pub fn list(search_name: String) -> Result<(), Box<dyn std::error::Error>> {
    let body = list_body();

    match ureq::post(&crate::url()).send_json(body) {
        Ok(res) => {
            let data = res.into_string()?;
            let data: Value = serde_json::from_str(&data)?;
            let mut data_new: Value;

            for item in data["result"].as_array().unwrap() {
                data_new = list_data(item.clone())?;

                if data_new["name"].as_str().unwrap_or("no_name") == &search_name {
                    crate::success(data_new);
                    return Ok(());
                }
            }

            crate::success(json!({}));
        }
        Err(e) => {
            crate::error(e.to_string());
        }
    }

    Ok(())
}

pub fn list_all() -> Result<(), Box<dyn std::error::Error>> {
    let body = list_body();

    match ureq::post(&crate::url()).send_json(body) {
        Ok(res) => {
            let data = res.into_string()?;
            let data:Value = serde_json::from_str(&data)?;
            let mut data_new:Value = json!({});

            for item in data["result"].as_array().unwrap() {
                data_new[item["gid"].as_str().unwrap()] = list_data(item.clone())?;
            }

            crate::success(data_new);
        }
        Err(e) => {
            crate::error(e.to_string());
        }
    }

    Ok(())
}

pub fn list_body() -> serde_json::Value {
    json!({
        "jsonrpc":"2.0",
        "method":"aria2.tellActive",
        "id": crate::id(),
        "params":[
            crate::token(),
            [
                "gid","status","bittorrent","dir","files",
                "totalLength","completedLength",
                "uploadSpeed","downloadSpeed","connections",
                "numSeeders","seeder","status","infoHash",
                "errorCode","verifiedLength","verifyIntegrityPending"
            ]
        ]
    })
}

pub fn list_data(item: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let gid = item["gid"].as_str().unwrap();

    let name;


    if !item["bittorrent"].is_null() {
        let bittorrent = item["bittorrent"].as_object().unwrap();
        let info = bittorrent["info"].as_object().unwrap();
        name = info["name"].as_str().unwrap();
        
    } else {
        let path = item["files"][0]["path"].as_str().unwrap();
        let path = std::path::Path::new(path);
        name = path.file_name().unwrap().to_str().unwrap();
    }
    let mut info_hash = "";

    if !item["infoHash"].is_null() {
        info_hash = item["infoHash"].as_str().unwrap();
    }

    let mut num_seeders = "0";
    let mut seeder = "false";

    if !item["numSeeders"].is_null() {
        num_seeders = item["numSeeders"].as_str().unwrap();
        seeder = item["seeder"].as_str().unwrap();
    }
    
    let path = item["dir"].as_str().unwrap();
    let path = format!("{}/{}", path, name);
    let path = path.replace("\\", "/");
    let dir = path.replace("//", "/");

    let data = json!({
        "gid": gid,
        "name": name,
        "status": item["status"].as_str().unwrap(),
        "connections": item["connections"].as_str().unwrap(),
        "num_seeders": num_seeders,
        "seeder": seeder,
        "completed_length": item["completedLength"].as_str().unwrap(),
        "total_length": item["totalLength"].as_str().unwrap(),
        "download_speed": item["downloadSpeed"].as_str().unwrap(),
        "upload_speed": item["uploadSpeed"].as_str().unwrap(),
        "info_hash": info_hash,
        "dir": dir,
        "files": item["files"].as_array().unwrap(),
    });

    Ok(data)
}

pub fn follow_add(url: String, search_name: String) -> Result<(), Box<dyn std::error::Error>> {
    // 先获取gid
    let mut i = 0;
    let mut data_return: Value;

    loop {
        i += 1;
        // 循环获取list, 如果超过一定时间没有找到下载的文件，则报错
        if i > 5 {
            crate::error("没有找到下载的文件".to_string());
            return Ok(());
        }

        let data = wei_run::command("wei-download", vec!["list", search_name.as_str()])?;
        let data: Value = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(_) => {
                continue;
            }
        };
  
        if let Some(obj) = data["data"].as_object() {
            for (gid, details) in obj {

                let name = details["name"].as_str().unwrap();
                let completed_length = details["completed_length"].as_str().unwrap();
                let total_length = details["total_length"].as_str().unwrap();

                i = 0;
                data_return = json!({
                    "code": 200,
                    "message": "success",
                    "data": {
                        "gid": gid,
                        "name": name,
                        "completedLength": completed_length,
                        "totalLength": total_length,
                    }
                });

                if completed_length == total_length {
                    crate::success(data_return);
                    return Ok(());
                }

                // 循环获取的过程中，发送上报数据给服务端
                match ureq::post(&url).send_json(data_return.clone()) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
