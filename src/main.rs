#[macro_use]
extern crate http_header;

use http_header::{Header, ResponseBuilder};
use std::convert::TryFrom;
use std::error::Error;
use tokio::fs;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use uriparse::URI;

#[tokio::main]
async fn main() {
    let address = "127.0.0.1:8080";
    let mut listener = TcpListener::bind(address).await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            match process_request(&mut socket).await {
                Err(e) => println!("Err {}", e),
                _ => (),
            }
        });
    }
}

async fn http_request_header(socket: &mut TcpStream) -> Result<Header, Box<dyn Error>> {
    const CHUNK: usize = 25;
    let mut bytes = vec![];
    let mut peek: [u8; CHUNK] = [0; CHUNK];
    let mut bytes_read = 0;
    let mut last_chunk = false;
    while !last_chunk {
        let bytes_to_read = socket.peek(&mut peek).await?;
        if bytes_to_read != CHUNK {
            last_chunk = true;
        }
        bytes.resize(bytes.len() + bytes_to_read, 0);
        bytes_read += socket
            .read(&mut bytes[bytes_read..(bytes_read + bytes_to_read)])
            .await?;
    }
    let header = Header::parse(&bytes)?;
    Ok(header)
}

async fn process_request(socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let (method, path, _) = http_request_header(socket).await?.status_line;
    let as_uri = format!("schema://server{}", path.as_ref() as &str);
    let uri = URI::try_from(&as_uri[..])?;
    let path = uri.path();

    match method.as_ref() as &str {
        "GET" => serve_static(&format!(".{}", path), socket).await,
        _ => not_implemented(socket).await,
    }
}

async fn not_implemented(socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut content = b"not implemented".to_vec();

    let mut response_header = ResponseBuilder::new()
        .version(data!("HTTP/1.1"))
        .status(501)
        .reason(data!("Not Implemented"))
        .build()?
        .to_vec();
    response_header.append(&mut content);
    let ok = socket.write_all(&response_header).await?;
    Ok(ok)
}

async fn serve_static(path: &str, socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    match fs::read(path.as_ref() as &str).await {
        Ok(mut content) => {
            let mut response_header = ResponseBuilder::new()
                .version(data!("HTTP/1.1"))
                .status(200)
                .reason(data!("OK"))
                .field(data!("content-type"), data!("text/html"))
                .field(data!("content-length"), data!(content.len().to_string()))
                .build()?
                .to_vec();

            response_header.append(&mut content);
            let ok = socket.write_all(&response_header).await?;
            return Ok(ok);
        }
        Err(_) => {
            let mut content = b"not found".to_vec();
            let mut response_header = ResponseBuilder::new()
                .version(data!("HTTP/1.1"))
                .status(404)
                .reason(data!("Not Found"))
                .build()?
                .to_vec();

            response_header.append(&mut content);
            let ok = socket.write_all(&response_header).await?;
            return Ok(ok);
        }
    }
}
