use crate::SshInfo;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::{Read, Result, Write};
use std::time::Duration;
use telnet::{Event, Telnet};
use tokio::task;

const SCU_TELNET_PORT: u16 = 6800;

pub fn connect_telnet(
    ip: String,
    port: u16,
    user: String,
    passwd: String,
) -> tokio::io::Result<()> {
    let date = Local::now();
    let filename = format!("log_{}.txt", date.format("%Y_%m_%d"));
    let mut log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&filename)
        .unwrap();
    //let (tx, mut rx) = mpsc::channel(100);
    {
        let mut telnet_stream =
            Telnet::connect((ip, port), 1024).expect("Couldn't connect to the server...");

        let event = telnet_stream.read_timeout(Duration::from_secs(1)).unwrap();
        match event {
            Event::Data(buffer) => {
                println!("{}", String::from_utf8_lossy(&buffer));
            }
            Event::NoData => {
                println!("No data");
            }
            Event::TimedOut => {}
            Event::Error(err) => {
                println!("Error: {:?}", err);
            }
            _ => {}
        }

        telnet_stream.write(user.as_bytes()).unwrap();
        telnet_stream.write(b"\n").unwrap();
        telnet_stream.write(passwd.as_bytes()).unwrap();
        telnet_stream.write(b"\n").unwrap();
        telnet_stream.write("?".as_bytes()).unwrap();
        telnet_stream.write(b"\n").unwrap();

        for _ in 0..50 {
            let event =
                task::block_in_place(|| telnet_stream.read_timeout(Duration::from_micros(100)));
            match event {
                Ok(Event::Data(buffer)) => {
                    let received_data = String::from_utf8_lossy(&buffer).to_string();
                    println!("{}", received_data);
                    let date = Local::now();
                    let timestamp = date.format("%Y-%m-%d %H:%M:%S").to_string();
                    writeln!(log_file, "[{}] {}", timestamp, received_data).unwrap();
                    //tx.send("1").await.unwrap();
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error reading from telnet stream: {:?}", e);
                    break;
                }
            }
        }
        //return telnet_stream;
    };
    /*
    tokio::spawn(async move {
        while let Some(received_data) = rx.recv().await {
            println!("{}", received_data);
            //处理收到的数据
        }
    });*/
    Ok(())
}

pub fn cmd_via_telnet(tel_auth: &SshInfo, cmd: &String) -> Result<String> {
    let date = Local::now();
    let filename = format!("log_{}.txt", date.format("%Y_%m_%d"));
    let mut log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&filename)
        .unwrap();
    //let (tx, mut rx) = mpsc::channel(100);

    let mut telnet_stream = Telnet::connect((tel_auth.ip.as_str(), SCU_TELNET_PORT), 1024)
        .expect("Couldn't connect to the server...");

    let event = telnet_stream.read_timeout(Duration::from_secs(1)).unwrap();
    match event {
        Event::Data(buffer) => {
            println!("{}", String::from_utf8_lossy(&buffer));
        }
        Event::NoData => {
            println!("No data");
        }
        Event::TimedOut => {}
        Event::Error(err) => {
            println!("Error: {:?}", err);
        }
        _ => {}
    }

    telnet_stream.write(tel_auth.username.as_bytes()).unwrap();
    telnet_stream.write(b"\n").unwrap();
    telnet_stream.write(tel_auth.password.as_bytes()).unwrap();
    telnet_stream.write(b"\n").unwrap();
    telnet_stream.write(cmd.as_bytes()).unwrap();
    telnet_stream.write(b"\n").unwrap();

    let mut s_recv = String::new();
    for _ in 0..50 {
        let event = task::block_in_place(|| telnet_stream.read_timeout(Duration::from_micros(100)));
        match event {
            Ok(Event::Data(buffer)) => {
                let received_data = String::from_utf8_lossy(&buffer).to_string();
                println!("{}", received_data);
                let date = Local::now();
                let timestamp = date.format("%Y-%m-%d %H:%M:%S").to_string();
                s_recv.push_str(&received_data);
                writeln!(log_file, "[{}] {}", timestamp, received_data).unwrap();
                //tx.send("1").await.unwrap();
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading from telnet stream: {:?}", e);
                break;
            }
        }
    }
    //return telnet_stream;
    /*
    tokio::spawn(async move {
        while let Some(received_data) = rx.recv().await {
            println!("{}", received_data);
            //处理收到的数据
        }
    });*/
    Ok(s_recv)
}

#[allow(dead_code)]
fn connect_via_telnet(
    address: String,
    port: String,
    command: String,
) -> impl std::future::Future<Output = String> {
    async move {
        let mut telnet =
            Telnet::connect((address.as_str(), port.parse::<u16>().unwrap()), 256).unwrap();
        telnet.write(command.as_bytes()).unwrap();
        let event = telnet.read().unwrap();
        match event {
            telnet::Event::Data(buffer) => String::from_utf8_lossy(&buffer).into_owned(),
            _ => "Failed to receive response".into(),
        }
    }
}
