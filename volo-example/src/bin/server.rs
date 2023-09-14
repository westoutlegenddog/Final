#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use volo_example::LogLayer;
use tokio::spawn;
use futures::future::join_all;
//use std::collections::HashMap;
use volo_example::S;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[volo::main]
async fn main() {
    
    let infos = new();


    let server_tasks = infos.iter().map(|inner: &(String, Vec<String>, Vec<String>)| {
        let port = inner.0.clone();
        let proxy = inner.1.clone();
        let master = inner.2.clone();
        //println!("ser, {:?}", port);
        let addr: SocketAddr = port.parse().unwrap();
        let addr = volo::net::Address::from(addr);
        

        let task = async move {
            volo_gen::volo::example::ItemServiceServer::new(S::new(port, proxy, master))
                .layer_front(LogLayer)
                .run(addr)
                .await
                .unwrap();
        };
        spawn(task)
    });
    // 使用 join_all 来等待所有服务器任务完成
    let _ = join_all(server_tasks).await;
}

fn new() ->  Vec<(String, Vec<String>, Vec<String>)>{
    let myitem: Vec<(String, Vec<String>, Vec<String>)>;
    let path = "Server.json".to_string();
    if Path::new(&path).exists() {
        println!("Servers are created from the file");
        let mut file = File::open(&path).expect("Error (1) in reading the file");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).expect("Error (2) in reading the file");
        let lines: Vec<&str> = file_contents.lines().collect();
        myitem = serde_json::from_str(lines[0]).expect("Error (3) in reading the file");
    } else {
        println!("A new server is created");
        myitem = vec![
            ("127.0.0.1:8080".to_string(),vec!["y".to_string()],vec!["y".to_string()])
        ]; 
    }
    myitem
}
