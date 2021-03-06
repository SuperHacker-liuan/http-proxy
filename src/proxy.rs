use crate::config::SiteControl;
use crate::error::HttpProxyError;
use crate::Result;
use crate::CONFIG;
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
    let server = TcpListener::bind(CONFIG.listen).await?;
    let mut server = server.incoming();
    while let Some(stream) = server.next().await {
        let stream = stream?;
        task::spawn(async move {
            match serve_conn(stream).await {
                Ok(_) => {}
                Err(HttpProxyError::HttpParse(_)) => {}
                Err(e) => log::error!("Connection Error: {:?}", e),
            }
        });
    }
    Ok(())
}

async fn serve_conn(mut stream: TcpStream) -> Result<()> {
    // CONNECT a.jp HTTP/2\r\n\r\n, total 23 bytes,
    // shorter than this must not HTTP requests
    // Add this to drop Non-HTTP connects
    const HTTP_MINIMUM_BYTES: usize = 23;
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    if n < HTTP_MINIMUM_BYTES {
        return Ok(());
    }

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut request = Request::new(&mut headers);
    let from = format!("{}", stream.peer_addr()?);

    request.parse(&buf[0..n])?;
    let host = match parse_headers(&request)? {
        Some(host) => host,
        None => {
            log::warn!(
                "Cannot parse Host, {:?} {:?} {:?}, from {}",
                request.method,
                request.path,
                request.version,
                &from
            );
            return Ok(());
        }
    };

    let mut target = match parse_host(host, &from).await {
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

fn check_valid(host: &str, port: u16, from: &str) -> bool {
    match &CONFIG.site_control {
        SiteControl::Disable => true,
        SiteControl::Allow(list) => match list.iter().find(|policy| host.ends_with(*policy)) {
            Some(_) => true,
            None => {
                log::info!("Not Allowed {}:{}, from {}", host, port, from);
                false
            }
        },
        SiteControl::Block(list) => match list.iter().find(|policy| host.ends_with(*policy)) {
            Some(_) => false,
            None => {
                log::info!("Accept {}:{}, from {}", host, port, from);
                true
            }
        },
    }
}

fn parse_headers(request: &Request) -> Result<Option<String>> {
    for header in request.headers.iter() {
        if header.name == "Host" {
            return Ok(Some(String::from_utf8(header.value.to_vec())?));
        }
    }
    match request.method {
        Some("CONNECT") => Ok(request.path.map(|s| String::from(s))),
        _ => Ok(None),
    }
}

async fn parse_host(host: String, from: &str) -> Option<SocketAddr> {
    let addr = match host.parse::<SocketAddr>() {
        Ok(addr) => {
            // IP:port
            let host = format!("{}", addr.ip());
            if !check_valid(&host, addr.port(), from) {
                return None;
            }
            addr
        }
        Err(_) => {
            // no port(use default 80) || use domain
            let mut parts = host.splitn(2, ":");
            let host = parts.next()?;
            let port: u16 = parts.next().unwrap_or("80").parse().ok()?;
            if !check_valid(&host, port, from) {
                return None;
            }
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
    const NEW_LINE: &str = "\r\n";
    static CONNECT_RESP: &[&str] = &[
        "HTTP/1.1 200 Tunnel established",
        NEW_LINE,
        "Proxy: SuperHacker HTTP Proxy/",
        clap::crate_version!(),
        NEW_LINE,
        NEW_LINE,
    ];

    let resp = CONNECT_RESP.concat();
    stream.write_all(resp.as_bytes()).await?;
    Ok(())
}
