#![feature(impl_trait_in_assoc_type)]
use anyhow::Error;
use anyhow::Ok;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DB {
    kvs: HashMap<String, String>,
}

impl Serialize for DB {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.kvs.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DB {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kvs = HashMap::deserialize(deserializer)?;
        core::result::Result::Ok(DB { kvs })
    }
}
pub struct Tm {
    kts: HashMap<String, (u128, u128)>,
}

impl Serialize for Tm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.kts.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kts = HashMap::deserialize(deserializer)?;
        core::result::Result::Ok(Tm { kts })
    }
}

pub struct S {
    contents: RwLock<RefCell<DB>>,
    times: RwLock<RefCell<Tm>>,
    port: RwLock<RefCell<String>>,
    proxy: RwLock<RefCell<Vec<String>>>,
    master: RwLock<RefCell<Vec<String>>>,
}

impl S {
    pub fn store(&self) -> Result<(), volo_thrift::AnyhowError> {
        // 获取内容锁
        let contents = self.contents.read().unwrap();
        let times = self.times.read().unwrap();
        let port = self.port.read().unwrap();
        let proxy = self.proxy.read().unwrap();
        let master = self.master.read().unwrap();

        // 序列化内容
        let myport = port.borrow().clone();
        let db_json = serde_json::to_string(&*contents.borrow())?;
        let tm_json = serde_json::to_string(&*times.borrow())?;
        let port_json: String = serde_json::to_string(&*port.borrow())?;
        let proxy_json: String = serde_json::to_string(&*proxy.borrow())?;
        let master_json: String = serde_json::to_string(&*master.borrow())?;
        //println!("store{}", myport);

        // 写入文件
        //println!("1111111");
        let mut file = File::create("data".to_string() + &myport + ".json")?;
        file.write_all(db_json.as_bytes())?;
        file.write_all(b"\n")?; // 添加换行符以分隔内容
        file.write_all(tm_json.as_bytes())?;
        file.write_all(b"\n")?; // 添加换行符以分隔内容
        file.write_all(port_json.as_bytes())?;
        file.write_all(b"\n")?; // 添加换行符以分隔内容
        file.write_all(proxy_json.as_bytes())?;
        file.write_all(b"\n")?; // 添加换行符以分隔内容
        file.write_all(master_json.as_bytes())?;
        Ok(())
    }
    pub fn new(port: String, proxy: Vec<String>, master: Vec<String>) -> Self {
        let db: DB;
        let tm: Tm;
        let myport: String;
        let myproxy: Vec<String>;
        let mymaster: Vec<String>;

        let path = "data".to_string() + &port + ".json";
        if Path::new(&path).exists() {
            println!("A database is created from the file");
            // 如果文件存在，则从文件中导入内容
            let mut file = File::open(&path).expect("Error (1) in reading the file");
            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents)
                .expect("Error (2) in reading the file");

            let lines: Vec<&str> = file_contents.lines().collect();

            // 反序列化内容
            db = serde_json::from_str(lines[0]).expect("Error (3) in reading the file");
            tm = serde_json::from_str(lines[1]).expect("Error (4) in reading the file");
            myport = serde_json::from_str(lines[2]).expect("Error (5) in reading the file");
            myproxy = serde_json::from_str(lines[3]).expect("Error (6) in reading the file");
            mymaster = serde_json::from_str(lines[4]).expect("Error (7) in reading the file");
        } else {
            println!("A new database is created");
            // 如果文件不存在，则创建新的结构体
            db = DB {
                kvs: HashMap::new(),
            };
            tm = Tm {
                kts: HashMap::new(),
            };
            myport = port;
            myproxy = proxy;
			//println!("sernew{:?}", myproxy);
            mymaster = master;
        }
        // 创建 S 结构体并返回
        S {
            contents: RwLock::new(RefCell::new(db)),
            times: RwLock::new(RefCell::new(tm)),
            port: RwLock::new(RefCell::new(myport)),
            proxy: RwLock::new(RefCell::new(myproxy)),
            master: RwLock::new(RefCell::new(mymaster)),
        }
    }
    pub fn check(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let before = self.times.write().unwrap().borrow().kts.clone();
        self.times
            .write()
            .unwrap()
            .borrow_mut()
            .kts
            .retain(|_, timestamp| now - timestamp.1 <= timestamp.0);
        self.contents
            .write()
            .unwrap()
            .borrow_mut()
            .kvs
            .retain(|key, _| {
                (self.times.write().unwrap().borrow().kts.contains_key(key)
                    || (!before.contains_key(key)))
            });
    }
}

unsafe impl Send for S {}
unsafe impl Sync for S {}

#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
    // 这部分是我们需要增加的代码

    async fn get_item(
        &self,
        _req: volo_gen::volo::example::GetItemRequest,
    ) -> core::result::Result<volo_gen::volo::example::GetItemResponse, volo_thrift::AnyhowError>
    {
        let mut resp = volo_gen::volo::example::GetItemResponse {
            op: " ".into(),
            key: " ".into(),
            value: " ".into(),
            state: false,
        };
        let opstr = &(_req.op[..]);
        let key = (&_req.key[..]).to_string();
        let value = (&_req.value[..]).to_string();
        let life: u128 = _req.life.try_into().unwrap();
        let otherport = (&_req.otherport[..]).to_string();
        self.check();
        self.store().expect("Error in storing the data");
        //println!("{}",life);
        match opstr {
            "get" => {
				if self.proxy.read().unwrap().borrow()[0].eq("y") {
					let mut dis:HashMap<i32, String> = HashMap::new();
					let ports = self.proxy.read().unwrap().borrow().clone();
					let mut cnt = 0;
					//println!("{}\n{:?}",ports.len(), self.proxy.read().unwrap().borrow());
					for s_ports in ports.iter().skip(1) {
						dis.insert(cnt, s_ports.clone());
						cnt += 1;
					}
					let which = i32::from(key.as_bytes()[0])%cnt;
					//println!("which {}", which);
					//println!("{:?}", dis);
					println!("get distributed to {}", dis[&which]);
					resp.state = true;
                    //self.store().expect("Error in storing the data");
                    let port = dis[&which].clone();
                    let keyport = key.clone();
                    //let value = resp.value.clone();
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    tokio::spawn(async move {
                        let message = other("get".to_string(), keyport, " ".into(), &port).await;
                        //println!("lib{:?}", message);
                        if tx.send(message).is_err() {
                            println!("Failed to send result to the channel");
                        }
                    });
                    let message = rx.await.expect("Failed to receive result from the channel");
                    return Ok(message);
				}
                else if self
                    .contents
                    .read()
                    .unwrap()
                    .borrow()
                    .kvs
                    .contains_key(&key)
                {
                    resp.op = "get".into();
                    resp.key = key.clone().into();
                    resp.value = self.contents.read().unwrap().borrow().kvs[&key]
                        .clone()
                        .into();
                    resp.state = true;
                    return Ok(resp);
                } else {
                    //println!("hhh");
                    resp.op = "get".into();
                    resp.key = key.clone().into();
                    resp.state = false;
                    return Ok(resp);
                }
            }
            "getport" => {
                if self.proxy.read().unwrap().borrow()[0].eq("n") {
                    //println!("fffff");
                    resp.op = "getportfail".into();
                    resp.value = self.proxy.read().unwrap().borrow()[1].clone().into();
                    resp.state = false;
                    return Ok(resp);
                } else {
                    resp.state = true;
                    //self.store().expect("Error in storing the data");
                    let port = otherport.clone();
                    let keyport = key.clone();
                    //let value = resp.value.clone();
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    tokio::spawn(async move {
                        let message = other("get".to_string(), keyport, " ".into(), &port).await;
                        //println!("lib{:?}", message);
                        if tx.send(message).is_err() {
                            println!("Failed to send result to the channel");
                        }
                    });
                    let message = rx.await.expect("Failed to receive result from the channel");
                    return Ok(message);
                }
            }
            "set" => {
                //println!("hhhhhhhhhhh{}", value);
                resp.op = "set".into();
                resp.key = key.clone().into();
                resp.value = value.clone().into();
				if self.proxy.read().unwrap().borrow()[0].eq("y"){
					let mut dis:HashMap<i32, String> = HashMap::new();
					let ports = self.proxy.read().unwrap().borrow().clone();
					let mut cnt = 0;
					//println!("{}\n{:?}",ports.len(), self.proxy.read().unwrap().borrow());
					for s_ports in ports.iter().skip(1) {
						dis.insert(cnt, s_ports.clone());
						cnt += 1;
					}
					let which = i32::from(key.as_bytes()[0])%cnt;
					//println!("which {}", which);
					//println!("{:?}", dis);
					println!("set distributed to {}", dis[&which]);
					resp.state = true;
                    //self.store().expect("Error in storing the data");
                    let port = dis[&which].clone();
                    let key = resp.key.clone();
                    let value = resp.value.clone();
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    tokio::spawn(async move {
                        let message = other(
                            "set".to_string(),
                            key.into_string(),
                            value.into_string(),
                            &port,
                        )
                        .await;
                        //println!("lib{:?}", message);
                        if tx.send(message).is_err() {
                            println!("Failed to send result to the channel");
                        }
                    });
                    let message = rx.await.expect("Failed to receive result from the channel");
                    return Ok(message);

				}
                else if self.master.read().unwrap().borrow()[0].eq("n") {
                    //println!("fffff");
                    resp.op = "setslave".into();
                    resp.value = self.master.read().unwrap().borrow()[1].clone().into();
                    resp.state = false;
                    return Ok(resp);
                } else if self
                    .contents
                    .read()
                    .unwrap()
                    .borrow()
                    .kvs
                    .contains_key(&key)
                {
                    resp.value = self.contents.read().unwrap().borrow().kvs[&key]
                        .clone()
                        .into();
                    resp.state = false;
                    return Ok(resp);
                } else {
                    self.contents
                        .write()
                        .unwrap()
                        .borrow_mut()
                        .kvs
                        .insert(key, value);
                    resp.state = true;
                    self.store().expect("Error in storing the data");
                    let ports = self.master.read().unwrap().borrow().clone();
                    for s_ports in ports.iter().skip(1) {
                        let port = s_ports.clone();
                        let key = resp.key.clone();
                        let value = resp.value.clone();
                        tokio::spawn(async move {
                            slave(
                                "setslave".to_string(),
                                key.into_string(),
                                value.into_string(),
                                &port,
                            )
                            .await;
                        });
                    }
					
                    return Ok(resp);
                }
            }
            "setport" => {
                //println!("hhhhhhhhhhh{}", value);
                resp.op = "setport".into();
                resp.key = key.clone().into();
                resp.value = value.clone().into();
                if self.proxy.read().unwrap().borrow()[0].eq("n") {
                    //println!("fffff");
                    resp.op = "setportfail".into();
                    resp.value = self.proxy.read().unwrap().borrow()[1].clone().into();
                    resp.state = false;
                    return Ok(resp);
                } else {
                    resp.state = true;
                    //self.store().expect("Error in storing the data");
                    let port = otherport.clone();
                    let key = resp.key.clone();
                    let value = resp.value.clone();
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    tokio::spawn(async move {
                        let message = other(
                            "set".to_string(),
                            key.into_string(),
                            value.into_string(),
                            &port,
                        )
                        .await;
                        //println!("lib{:?}", message);
                        if tx.send(message).is_err() {
                            println!("Failed to send result to the channel");
                        }
                    });
                    let message = rx.await.expect("Failed to receive result from the channel");
                    return Ok(message);
                }
            }
            "setslave" => {
                //println!("setsla {} {} {}", self.port.read().unwrap().borrow(),key, value);
                self.contents
                    .write()
                    .unwrap()
                    .borrow_mut()
                    .kvs
                    .insert(key, value);
                resp.state = true;
                self.store().expect("Error in storing the data");
                return Ok(resp);
            }
            "setex" => {
                resp.op = "setex".into();
                resp.key = key.clone().into();
                if self
                    .contents
                    .read()
                    .unwrap()
                    .borrow()
                    .kvs
                    .contains_key(&key)
                {
                    resp.value = self.contents.read().unwrap().borrow().kvs[&key]
                        .clone()
                        .into();
                    resp.state = false;
                    return Ok(resp);
                } else {
                    self.contents
                        .write()
                        .unwrap()
                        .borrow_mut()
                        .kvs
                        .insert(key.clone(), value);
                    self.times.write().unwrap().borrow_mut().kts.insert(
                        key,
                        (
                            life * 1000,
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis(),
                        ),
                    );
                    resp.state = true;
                    self.store().expect("Error in storing the data");
                    return Ok(resp);
                }
            }
            "del" => {
                resp.op = "del".into();
                resp.key = key.clone().into();
                if self
                    .contents
                    .read()
                    .unwrap()
                    .borrow()
                    .kvs
                    .contains_key(&key)
                {
                    self.contents.read().unwrap().borrow_mut().kvs.remove(&key);
                    resp.state = true;
                    self.store().expect("Error in storing the data");
                    return Ok(resp);
                } else {
                    resp.state = false;
                    return Ok(resp);
                }
            }
            "ping" => {
                //println!("hhh");
                //slave("8081").await;
                resp.op = "ping".into();
                resp.key = key.clone().into();
                resp.state = true;
                return Ok(resp);
            }
            "shutdown" => {
                //println!("dddddd");
                resp.op = "shutdown".into();
                resp.state = true;
                return Ok(resp);
            }

            _ => {
                tracing::error!("Invalid operation!");
            }
        }
        Ok(Default::default())
    }
}

#[derive(Clone)]
pub struct LogService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug + From<Error>,
    Cx: Send + 'static,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let command = format!("{:?}", &req);
        if command.contains("114514") {
            return Err(S::Error::from(Error::msg(
                "There are inappropriate words and they have been filtered",
            )));
        }
        let resp = self.0.call(cx, req).await;
        resp
    }
}

pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}

use std::net::SocketAddr;

static mut ADDR_STR: String = String::new();

async fn slave(opstr: String, key: String, value: String, port: &str) {
    //println!("1111{}", port);
    unsafe {
        ADDR_STR = port.to_string();
    }
    //tracing_subscriber::fmt::init();

    //println!("{:?}", ADDR_STR);}
    let client: volo_gen::volo::example::ItemServiceClient = unsafe {
        let addr: SocketAddr = ADDR_STR.parse().unwrap();
        //println!("addr {:?}", addr);
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };

    let req = volo_gen::volo::example::GetItemRequest {
        op: opstr.into(),
        key: key.into(),
        value: value.into(),
        life: 0i32,
        otherport: " ".into(),
    };
    //println!("3333");
    let resp = client.get_item(req).await;
    //println!("4444");
    match resp {
        core::result::Result::Ok(info) => {
            let state: bool = info.state;
            if state == true {
                println!("Succcessfully connect to slaves");
            } else {
                println!("Fail to connect to slaves");
            }
        }
        Err(e) => tracing::error!("{:?}", e),
    }
}

use volo_gen::volo::example::GetItemResponse;

async fn other(opstr: String, key: String, value: String, port: &str) -> GetItemResponse {
    unsafe {
        ADDR_STR = port.to_string();
    }
    let client: volo_gen::volo::example::ItemServiceClient = unsafe {
        let addr: SocketAddr = ADDR_STR.parse().unwrap();
        //println!("addr {:?}", addr);
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
    let req = volo_gen::volo::example::GetItemRequest {
        op: opstr.into(),
        key: key.into(),
        value: value.into(),
        life: 0i32,
        otherport: " ".into(),
    };
    //println!("3333");
    client.get_item(req).await.expect("Error in other()!")
}
