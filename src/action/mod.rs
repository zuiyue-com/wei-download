use serde_json::{Value, json};

pub fn list(body: Value, search_name: String) -> Result<(), Box<dyn std::error::Error>> {

    match ureq::post(&crate::url()).send_json(body) {
        Ok(res) => {
            let data = res.into_string()?;
            let data:Value = serde_json::from_str(&data)?;
            let mut data_new:Value = json!({});

            for item in data["result"].as_array().unwrap() {
                let gid = item["gid"].as_str().unwrap();
                let bittorrent = item["bittorrent"].as_object().unwrap();
                let info = bittorrent["info"].as_object().unwrap();
                let name = info["name"].as_str().unwrap();
                
                let path = item["dir"].as_str().unwrap();
                let path = format!("{}/{}", path, name);
                let path = path.replace("\\", "/");
                let dir = path.replace("//", "/");

                if name.find(&search_name) == None {
                    continue;
                }

                data_new[gid] = json!({
                    "gid": gid,
                    "name": name,
                    "status": item["status"].as_str().unwrap(),
                    "connections": item["connections"].as_str().unwrap(),
                    "numSeeders": item["numSeeders"].as_str().unwrap(),
                    "seeder": item["seeder"].as_str().unwrap(),
                    "completedLength": item["completedLength"].as_str().unwrap(),
                    "totalLength": item["totalLength"].as_str().unwrap(),
                    "downloadSpeed": item["downloadSpeed"].as_str().unwrap(),
                    "uploadSpeed": item["uploadSpeed"].as_str().unwrap(),
                    "dir": dir,
                    // "files": item["files"].as_array().unwrap(),
                });

            }

            crate::success(data_new);
        }
        Err(e) => {
            crate::error(e.to_string());
        }
    }

    Ok(())
}
