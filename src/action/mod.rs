use serde_json::{Value, json};

pub fn list(search_name: String) -> Result<(), Box<dyn std::error::Error>> {
    let body = match list_body() {
        Ok(body) => body,
        Err(e) => {
            crate::error(e.to_string());
            return Ok(());
        }
    };

    for (_,item) in body.as_object().unwrap() {
        if item["name"].as_str().unwrap_or("no_name") == &search_name {
            crate::success(item.clone());
            return Ok(());
        }
    }
    crate::success(json!({}));

    Ok(())
}

pub fn list_all() -> Result<(), Box<dyn std::error::Error>> {
    let body = match list_body() {
        Ok(body) => body,
        Err(e) => {
            crate::error(e.to_string());
            return Ok(());
        }
    };

    crate::success(body);
    Ok(())
}

pub fn list_body() -> Result<Value, Box<dyn std::error::Error>> {
    let data_struct = json!([
        "gid","status","bittorrent","dir","files",
        "totalLength","completedLength",
        "uploadSpeed","downloadSpeed","connections",
        "numSeeders","seeder","status","infoHash",
        "errorCode","verifiedLength","verifyIntegrityPending"
    ]);
    let mut data = json!({
        "jsonrpc":"2.0",
        "method":"aria2.tellActive",
        "id": crate::id(),
        "params":[
            crate::token(),
            data_struct
        ]
    });

    

    let data_active: Value = ureq::post(&crate::url()).send_json(data.clone())?.into_json()?;

    data["method"] = "aria2.tellWaiting".into();
    data["params"] = json!([crate::token(), 0, 1000, data_struct]);
    let data_waiting: Value = ureq::post(&crate::url()).send_json(data.clone())?.into_json()?;

    data["method"] = "aria2.tellStopped".into();
    let data_stopped: Value = ureq::post(&crate::url()).send_json(data)?.into_json()?;

    let mut data_new: Value = json!({});

    for item in data_active["result"].as_array().unwrap() {
        data_new[item["gid"].as_str().unwrap()] = list_data(item.clone())?;
    }

    for item in data_waiting["result"].as_array().unwrap() {
        data_new[item["gid"].as_str().unwrap()] = list_data(item.clone())?;
    }

    for item in data_stopped["result"].as_array().unwrap() {
        data_new[item["gid"].as_str().unwrap()] = list_data(item.clone())?;
    }

    Ok(data_new)
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
        seeder = match item["seeder"].as_str() {
            Some(seeder) => seeder,
            None => "false",
        };
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
    let sleep_time = 1;

    let mut gid;
    let mut name: String = "".to_string();
    let mut status;
    let mut completed_length;
    let mut total_length;

    // 一直都是空名，则退出，空名字60秒以上则退出
    // 如果是完成/停止/等待状态则退出，返回结果
    // completed_length == total_length 则退出，返回结果
    loop {
        i += 1;
        // 一直都是空名，则退出，空名字60秒以上则退出
        if i > 5 && name.clone() == "".to_string() {
            crate::error("没有找到下载的文件".to_string());
            return Ok(());
        }

        info!("search_name: {}", search_name);
        let data = wei_run::command("wei-download", vec!["list", search_name.as_str()])?;
        let data: Value = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_secs(sleep_time));
                continue;
            }
        };

        if data["code"] != 200 || data["data"].to_string() == "{}" {
            std::thread::sleep(std::time::Duration::from_secs(sleep_time));
            continue;
        }

        match data["data"].as_object() {
            Some(data) => {
                i = 0;

                gid = data.get("gid").map_or("", |v| v.as_str().unwrap_or_default());
                name = data.get("name").map_or("", |v| v.as_str().unwrap_or_default()).to_string();
                status = data.get("status").map_or("", |v| v.as_str().unwrap_or_default());
                completed_length = data.get("completed_length").map_or("", |v| v.as_str().unwrap_or_default());
                total_length = data.get("total_length").map_or("", |v| v.as_str().unwrap_or_default());

                data_return = json!({
                    "gid": gid,
                    "name": name.clone(),
                    "status": status,
                    "completedLength": completed_length,
                    "totalLength": total_length,
                });

                if status == "complete" || 
                   status == "error" || 
                   status == "removed" || 
                   status == "paused" ||
                   status == "waiting" {
                    crate::success(data_return);
                    return Ok(());
                }

                if completed_length == total_length &&
                   completed_length != "" && 
                   total_length != "" {
                    crate::success(data_return);
                    return Ok(());
                }

                match ureq::post(&url).send_json(data_return.clone()) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            None => {
                std::thread::sleep(std::time::Duration::from_secs(sleep_time));
                continue;
            }
        }
  
        std::thread::sleep(std::time::Duration::from_secs(sleep_time));
    }
}
