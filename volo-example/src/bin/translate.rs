//用于写server的配置文件

use std::fs::File;
use std::io::Write;
fn main() {
    let infos: Vec<(String, Vec<String>, Vec<String>)> = vec![
        (
            "127.0.0.1:8080".to_string(),
            vec!["y".to_string(), "127.0.0.1:8083".to_string(),"127.0.0.1:8084".to_string()],
            vec!["y".to_string()],
        ),
        (
            "127.0.0.1:8081".to_string(),
            vec!["n".to_string(), "127.0.0.1:8080".to_string()],
            vec!["n".to_string(), "127.0.0.1:8084".to_string()],
        ),
        (
            "127.0.0.1:8082".to_string(),
            vec!["n".to_string(), "127.0.0.1:8080".to_string()],
            vec!["n".to_string(), "127.0.0.1:8084".to_string()],
        ),
        (
            "127.0.0.1:8083".to_string(),
            vec!["n".to_string(), "127.0.0.1:8080".to_string()],
            vec!["y".to_string()],
        ),
        (
            "127.0.0.1:8084".to_string(),
            vec!["n".to_string(), "127.0.0.1:8080".to_string()],
            vec![
                "y".to_string(),
                "127.0.0.1:8081".to_string(),
                "127.0.0.1:8082".to_string(),
            ],
        ),
    ];

    store(infos.clone());
}
fn store(item: Vec<(String, Vec<String>, Vec<String>)>) {
    // 序列化内容
    let item_json = serde_json::to_string(&item).expect("Error (0) in server store");
    let mut file = File::create("Server.json".to_string()).expect("Error (1) in server store");
    file.write_all(item_json.as_bytes())
        .expect("Error (2) in server store");
}
