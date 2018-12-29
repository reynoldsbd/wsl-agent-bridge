//! Proxies named pipe traffic over a UNIX domain socket

use std::fs;
use std::io::ErrorKind;

use futures::future::Future;

use tokio::io;
use tokio::prelude::*;

use tokio_named_pipe::PipeStream;

use tokio_uds_windows::UnixListener;


// This application only works on Windows
#[cfg(not(target_family = "windows"))]
compile_error!("nprox is a Windows-only utility");


// TODO: grab these values from CLI args, the registry, or some other configuration source
const PIPE_NAME: &'static str = "\\\\.\\pipe\\openssh-ssh-agent";
const SOCK_NAME: &'static str = ".\\nprox.sock";


fn main()
{
    // Clean up socket before starting
    match fs::remove_file(SOCK_NAME)
    {
        Err(ref e) if e.kind() != ErrorKind::NotFound =>
            panic!("failed to clean old socket: {}", e),
        _ => ()
    }

    let server = UnixListener::bind(SOCK_NAME)
        .expect("failed to bind socket")
        .incoming()
        .for_each(|client| {
            let server = PipeStream::connect(PIPE_NAME, None)?;

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
        })
        .map_err(|e| eprintln!("connection error: {}", e));

    // TODO: is graceful shutdown needed?
    tokio::run(server);
}
