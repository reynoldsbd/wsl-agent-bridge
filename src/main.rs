//! Proxies named pipe traffic over a UNIX domain socket

use std::fs;
use std::io::ErrorKind;

use clap::{App, Arg};

use futures::future::Future;

use tokio::io;
use tokio::prelude::*;

use tokio_named_pipe::PipeStream;

use tokio_uds_windows::UnixListener;


// This application only works on Windows
#[cfg(not(target_family = "windows"))]
compile_error!("nprox is a Windows-only utility");


/// Proxies a connection upstream
///
/// Returns a closure that proxies connections. Each time the closure is invoked, the incoming
/// client connection is proxied to a new server connection (the latter is retrieved using the
/// `build_server` argument)
fn proxy_to<B, C, S>(build_server: B) -> impl Fn(C) -> Result<(), io::Error>
where B: Fn() -> Result<S, io::Error>,
    C: AsyncRead + AsyncWrite + Send + 'static,
    S: AsyncRead + AsyncWrite + Send + 'static
{
    move |client| {
        let server = build_server()?;

        // Based on Tokio's provided proxy example:
        // https://github.com/tokio-rs/tokio/blob/master/examples/proxy.rs

        let (client_reader, client_writer) = client.split();
        let (server_reader, server_writer) = server.split();

        let c_to_s = io::copy(client_reader, server_writer)
            .and_then(|(n, _, server_writer)| {
                io::shutdown(server_writer)
                    .map(move |_| n)
            });
        let s_to_c = io::copy(server_reader, client_writer)
            .and_then(|(n, _, client_writer)| {
                io::shutdown(client_writer)
                    .map(move |_| n)
            });

        let proxy = c_to_s.join(s_to_c)
            .map(|(from_client, from_server)| {
                println!(
                    "proxied {} bytes from client and {} bytes from server",
                    from_client,
                    from_server);
            })
            .map_err(|e| {
                eprintln!("failed to proxy traffic: {}", e);
            });

        tokio::spawn(proxy);
        Ok(())
    }
}

fn main()
{
    let args = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("PIPE")
            .help("Named pipe to proxy")
            .required(true)
            .index(1))
        .arg(Arg::with_name("SOCKET")
            .help("Unix socket providing proxy")
            .required(true)
            .index(2))
        .get_matches();
    let pipe_name = args.value_of("PIPE")
        .unwrap()
        .to_owned();
    let sock_name = args.value_of("SOCKET")
        .unwrap();

    // Clean up socket before starting
    match fs::remove_file(sock_name)
    {
        Err(ref e) if e.kind() != ErrorKind::NotFound =>
            panic!("failed to clean old socket: {}", e),
        _ => ()
    }

    let server = UnixListener::bind(sock_name)
        .expect("failed to bind socket")
        .incoming()
        .for_each(proxy_to(move || PipeStream::connect(&pipe_name, None)))
        .map_err(|e| eprintln!("connection error: {}", e));

    // TODO: clean up socket before exiting
    tokio::run(server);
}
