// chat_client/src/main.rs
mod protocol;
mod utils;
use utils::{send_packet, read_packet, reader_packet, writer_packet};
use protocol::{RegisterRequest, LoginRequest, SendMessageRequest, ServerResponse};

use tokio::net::TcpStream;
use anyhow::Result;

use std::io::{self, Write};






#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("连接到服务器");

    // 选择注册或登录
    println!("请选择操作: 1. 注册  2. 登录");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    if choice == "1" {
        // 注册
        print!("输入用户名: ");
        io::stdout().flush()?;
        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        let username = username.trim().to_string();

        print!("输入密码: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        let password = password.trim().to_string();

        let register = RegisterRequest {
            action: "register".to_string(),
            username,
            password,
        };

        let msg = serde_json::to_value(register)?;
        send_packet(&mut stream, &msg).await?;

        let response = read_packet(&mut stream).await?;
        let server_response: ServerResponse = serde_json::from_value(response)?;
        println!("服务器响应: {:?}", server_response);
        return Ok(());
    } else if choice == "2" {
        // 登录
        print!("输入用户名: ");
        io::stdout().flush()?;
        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        let username = username.trim().to_string();

        print!("输入密码: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        let password = password.trim().to_string();

        let login = LoginRequest {
            action: "login".to_string(),
            username,
            password,
        };

        let msg = serde_json::to_value(login)?;
        send_packet(&mut stream, &msg).await?;

        let response = read_packet(&mut stream).await?;
        let server_response: ServerResponse = serde_json::from_value(response)?;
        println!("服务器响应: {:?}", server_response);

        if let ServerResponse::AuthResponse { status, message } = server_response {
            if status == "success" {
                println!("登录成功，开始聊天。输入 'exit' 退出。");
                let (mut reader, mut writer) = stream.into_split(); // 拆分读写，可能是错误的

                // 读取服务器消息
                tokio::spawn(async move {
                    loop {
                        match reader_packet(&mut reader).await {
                            Ok(msg) => {
                                if let Ok(server_response) = serde_json::from_value::<ServerResponse>(msg) {
                                    // println!("收到消息: {:?}", server_response);
                                    match server_response {
                                        ServerResponse::ReceiveMessage { sender, message, timestamp } => {
                                            println!("\n[{}] {}: {}", timestamp, sender, message);
                                        },
                                        ServerResponse::Error { message } => {
                                            println!("\n[错误] {}", message);
                                        },
                                        _ => {println!("未知消息: {:?}", server_response);},
                                    }
                                }
                            },
                            Err(e) => {
                                println!("读取消息时出错: {}", e);
                                break;
                            },
                        }
                    }
                });

                // 发送消息
                loop {
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let input = input.trim();
                    if input == "exit" {
                        break;
                    }

                    // 简单格式: receiver: message
                    if let Some((receiver, message)) = input.split_once(":") {
                        let send_msg = SendMessageRequest {
                            action: "send_message".to_string(),
                            receiver: receiver.trim().to_string(),
                            message: message.trim().to_string(),
                        };
                        // println!("发送消息: {} 给 {}", send_msg.message, send_msg.receiver);
                        let msg = serde_json::to_value(send_msg)?;
                        // writer.write_all(&msg.to_string().as_bytes()).await?;
                        writer_packet(&mut writer, &msg).await?;
                    } else {
                        println!("消息格式错误，使用 receiver: message");
                    }
                }
            } else {
                println!("登录失败: {}", message);
            }
        }
    }

    Ok(())
}
