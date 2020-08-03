use crate::Result;
use async_std::io;
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::net::ToSocketAddrs;
use async_std::prelude::*;
use async_std::task;
use futures::future::FutureExt;
use httparse::Request;
use std::net::SocketAddr;

pub async fn run() -> Result<()> {
    let server = format!(":::5555");
    let server = TcpListener::bind(server).await?;
    let mut server = server.incoming();
    while let Some(stream) = server.next().await {
        let stream = stream?;
        task::spawn(async move {
            if let Err(e) = serve_conn(stream).await {
                eprintln!("Connection Error: {:?}", e);
            }
        });
    }
    Ok(())
}

async fn serve_conn(mut stream: TcpStream) -> Result<()> {
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut request = Request::new(&mut headers);

    request.parse(&buf[0..n])?;
    let host = match parse_headers(&request)? {
        Some(host) => host,
        None => {
            eprintln!("Invalid Request, lack of Host");
            return Ok(());
        }
    };

    let mut target = match parse_host(host).await {
        Some(addr) => TcpStream::connect(addr).await?,
        None => return Ok(()),
    };

    match request.method {
        Some("CONNECT") => response_connect(&mut stream).await?,
        _ => target.write_all(&buf[..n]).await?,
    }

    // Sync Local/Remote Read/Write
    let (lr, lw) = &mut (&stream, &stream);
    let (rr, rw) = &mut (&target, &target);

    let cp1 = io::copy(lr, rw);
    let cp2 = io::copy(rr, lw);

    futures::select! {
        r1 = cp1.fuse() => r1?,
        r2 = cp2.fuse() => r2?,
    };
    Ok(())
}

fn parse_headers(request: &Request) -> Result<Option<String>> {
    for header in request.headers.iter() {
        if header.name == "Host" {
            return Ok(Some(String::from_utf8(header.value.to_vec())?));
        }
    }
    Ok(None)
}

async fn parse_host(host: String) -> Option<SocketAddr> {
    let addr = match host.parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(_) => {
            let mut parts = host.splitn(2, ":");
            let host = parts.next()?;
            let port: u16 = parts.next().unwrap_or("80").parse().ok()?;
            format!("{}:{}", host, port)
                .to_socket_addrs()
                .await
                .ok()?
                .next()?
        }
    };
    Some(addr)
}

async fn response_connect(stream: &mut TcpStream) -> Result<()> {
    static CONNECT_RESP: &[&str] = &[
        "HTTP/1.1 200 Tunnel established",
        "Proxy: SuperHacker HTTP Proxy/0.1.0",
    ];
    const NEW_LINE: &str = "\r\n";

    for resp in CONNECT_RESP.iter() {
        let resp = [resp, NEW_LINE].concat();
        stream.write_all(resp.as_bytes()).await?;
    }
    stream.write_all(NEW_LINE.as_bytes()).await?;
    Ok(())
}
