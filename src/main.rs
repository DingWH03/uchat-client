use anyhow::Result;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use std::io::{self, Write};

mod protocol;
mod utils;
use protocol::{LoginRequest, RegisterRequest, Request, SendMessageRequest, ServerResponse};
use utils::{read_packet, reader_packet, send_packet, writer_packet};

use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect("10.0.0.193:8080").await?;
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
                let (reader, writer) = stream.into_split();

                // 使用通道让输入输出并发运行
                let (tx, mut rx) = mpsc::channel::<String>(32);

                // 接收消息的任务
                let recv_task = tokio::spawn(async move {
                    let mut reader = BufReader::new(reader);
                    loop {
                        match reader_packet(&mut reader).await {
                            Ok(msg) => {
                                if let Ok(server_response) =
                                    serde_json::from_value::<ServerResponse>(msg)
                                {
                                    match server_response {
                                        ServerResponse::ReceiveMessage {
                                            sender,
                                            message,
                                            timestamp,
                                        } => {
                                            println!("\n[{}] {}: {}", timestamp, sender, message);
                                        }
                                        ServerResponse::Error { message } => {
                                            println!("\n[错误] {}", message);
                                        }
                                        _ => {
                                            println!("未知消息: {:?}", server_response);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("读取消息时出错: {}", e);
                                break;
                            }
                        }
                    }
                });

                // 发送消息的任务
                let send_task = tokio::spawn(async move {
                    let mut writer = writer;
                    while let Some(input) = rx.recv().await {
                        if input == "exit" {
                            break;
                        } else if input == "users" {
                            let request = Request {
                                action: "request".to_string(),
                                request: "online_users".to_string(),
                            };
                            let msg = serde_json::to_value(request).unwrap();
                            if let Err(e) = writer_packet(&mut writer, &msg).await {
                                println!("发送消息时出错: {}", e);
                                break;
                            }
                        }

                        // 简单格式: receiver: message
                        if let Some((receiver, message)) = input.split_once(":") {
                            let send_msg = SendMessageRequest {
                                action: "send_message".to_string(),
                                receiver: receiver.trim().to_string(),
                                message: message.trim().to_string(),
                            };
                            let msg = serde_json::to_value(send_msg).unwrap();
                            if let Err(e) = writer_packet(&mut writer, &msg).await {
                                println!("发送消息时出错: {}", e);
                                break;
                            }
                        } else {
                            println!("消息格式错误，使用 receiver: message");
                        }
                    }
                });

                // 用户输入的任务
                let input_task = tokio::spawn(async move {
                    let stdin = tokio::io::stdin();
                    let reader = BufReader::new(stdin);
                    let mut lines = reader.lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        let input = line.trim().to_string();
                        if tx.send(input).await.is_err() {
                            break;
                        }
                    }
                });

                // 等待所有任务完成
                tokio::select! {
                    _ = recv_task => {},
                    _ = send_task => {},
                    _ = input_task => {},
                }
            } else {
                println!("登录失败: {}", message);
            }
        }
    }

    Ok(())
}
