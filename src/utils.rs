use tokio::net::tcp;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::TcpStream;
use anyhow::Result;

const LENGTH_PREFIX_SIZE: usize = 4;

pub async fn send_packet(stream: &mut TcpStream, msg: &serde_json::Value) -> Result<()> {
    let msg_str = serde_json::to_string(msg)?;
    let msg_bytes = msg_str.as_bytes();
    let msg_len = msg_bytes.len() as u32;
    let len_bytes = msg_len.to_be_bytes();

    stream.write_all(&len_bytes).await?;
    stream.write_all(msg_bytes).await?;
    Ok(())
}

pub async fn read_packet(stream: &mut TcpStream) -> Result<serde_json::Value> {
    let mut len_buf = [0u8; LENGTH_PREFIX_SIZE];
    stream.read_exact(&mut len_buf).await?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut msg_buf = vec![0u8; msg_len];
    stream.read_exact(&mut msg_buf).await?;

    let msg: serde_json::Value = serde_json::from_slice(&msg_buf)?;
    Ok(msg)
}

pub async fn writer_packet(writer: &mut tcp::OwnedWriteHalf, msg: &serde_json::Value) -> Result<()> {
    let msg_str = serde_json::to_string(msg)?;
    let msg_bytes = msg_str.as_bytes();
    let msg_len = msg_bytes.len() as u32;
    let len_bytes = msg_len.to_be_bytes();

    writer.write_all(&len_bytes).await?;
    writer.write_all(msg_bytes).await?;
    Ok(())
}

// 修改后的 reader_packet 函数，支持 BufReader
pub async fn reader_packet(reader: &mut tokio::io::BufReader<tcp::OwnedReadHalf>) -> Result<serde_json::Value> {
    let mut len_buf = [0u8; LENGTH_PREFIX_SIZE];
    reader.read_exact(&mut len_buf).await?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut msg_buf = vec![0u8; msg_len];
    reader.read_exact(&mut msg_buf).await?;

    let msg: serde_json::Value = serde_json::from_slice(&msg_buf)?;
    Ok(msg)
}
