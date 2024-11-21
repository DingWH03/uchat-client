use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;

fn main() {
    // 服务端地址
    let server_address = "127.0.0.1:8080";

    // 连接到服务端
    let stream = TcpStream::connect(server_address).expect("无法连接到服务端");
    println!("已连接到服务端: {}", server_address);

    // 创建共享的 TcpStream
    let stream = Arc::new(stream);

    // 创建一个线程用于接收消息
    let stream_clone = Arc::clone(&stream);
    thread::spawn(move || {
        receive_messages(stream_clone);
    });

    // 主线程用于发送消息
    send_messages(stream);
}

fn receive_messages(stream: Arc<TcpStream>) {
    let mut reader = io::BufReader::new(&*stream);

    loop {
        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                println!("服务端已断开连接");
                break;
            }
            Ok(_) => {
                print!("\r{}\n> ", buffer.trim()); // 保留输入框提示符
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                eprintln!("接收消息时出错: {}", e);
                break;
            }
        }
    }
}

fn send_messages(stream: Arc<TcpStream>) {
    let stdin = io::stdin();
    let mut writer = &*stream;

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut message = String::new();
        stdin.lock().read_line(&mut message).unwrap();
        let message = message.trim();

        if message.is_empty() {
            continue;
        }

        if message.eq_ignore_ascii_case("exit") {
            println!("退出客户端...");
            break;
        }

        if let Err(e) = writer.write_all(format!("{}\n", message).as_bytes()) {
            eprintln!("发送消息时出错: {}", e);
            break;
        }
    }
}
