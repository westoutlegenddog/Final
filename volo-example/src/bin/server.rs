#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use volo_example::LogLayer;
use tokio::spawn;
use futures::future::join_all;
//use std::collections::HashMap;
use volo_example::S;


#[volo::main]
async fn main() {
    let infos: Vec<(String, Vec<String>, Vec<String>)> = vec![
        ("127.0.0.1:8080".to_string(),vec!["y".to_string()],vec!["y".to_string()]),
        ("127.0.0.1:8081".to_string(),vec!["n".to_string(),"127.0.0.1:8080".to_string()],vec!["n".to_string(), "127.0.0.1:8084".to_string()]),
        ("127.0.0.1:8082".to_string(),vec!["n".to_string(),"127.0.0.1:8080".to_string()],vec!["n".to_string(), "127.0.0.1:8084".to_string()]),
        ("127.0.0.1:8083".to_string(),vec!["n".to_string(),"127.0.0.1:8080".to_string()],vec!["y".to_string()]),
        ("127.0.0.1:8084".to_string(),vec!["n".to_string(),"127.0.0.1:8080".to_string()],vec!["y".to_string(), "127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string()]),
    ]; 
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
