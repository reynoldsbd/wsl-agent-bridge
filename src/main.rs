use std::env;
use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;
use futures::future::Future;
use tokio::io;
use tokio::prelude::*;
use tokio_named_pipe::PipeStream;
use tokio_uds_windows::UnixListener;


#[cfg(not(target_family = "windows"))]
compile_error!("this is a Windows-only utility");


const DEFAULT_SOCK: &str = "ssh-agent.sock";

/// Gets path for the bridge socket
fn get_sock_path() -> PathBuf {

    // $SSH_AUTH_SOCK may be used to override default path
    if let Some(sock_path) = env::var_os("SSH_AUTH_SOCK") {
        PathBuf::from(sock_path)

    // Otherwise, pick a well-known, private place to put the socket
    } else {
        let base = BaseDirs::new().expect("failed to load base dirs");
        let mut sock_path = PathBuf::from(base.data_local_dir());
        sock_path.push(DEFAULT_SOCK);
        sock_path
    }
}


const AGENT_PIPE: &str = "\\\\.\\pipe\\openssh-ssh-agent";

/// Proxies an incoming connection to the real ssh-agent
fn proxy_to_agent_pipe<C>(client: C) -> Result<(), std::io::Error>
where C: AsyncRead + AsyncWrite + Send + 'static {

    // Connect to the real pipe
    let server = PipeStream::connect(AGENT_PIPE, None)?;

    // Create futures to pipe data between the connections
    let (client_reader, client_writer) = client.split();
    let (server_reader, server_writer) = server.split();
    let c_to_s = io::copy(client_reader, server_writer)
        .and_then(|(_, _, server_writer)| io::shutdown(server_writer));
    let s_to_c = io::copy(server_reader, client_writer)
        .and_then(|(_, _, client_writer)| io::shutdown(client_writer));

    // Add the composite future to the current runtime
    tokio::spawn(
        c_to_s.join(s_to_c)
            .map(|(_, _)| ())
            .map_err(|e| eprintln!("failed to proxy traffic: {}", e))
    );
    Ok(())
}


fn main() {

    // Get and prepare socket path
    let sock_path = get_sock_path();
    if sock_path.exists() {
        fs::remove_file(&sock_path)
            .expect("failed to remove existing socket");
    }

    // Open the socket, and proxy incoming connections to ssh-agent's pipe
    tokio::run(
        UnixListener::bind(&sock_path)
            .expect("failed to bind socket")
            .incoming()
            .for_each(proxy_to_agent_pipe)
            .map_err(|e| eprintln!("connection error: {}", e))
    );
}
