//! Proxies named pipe traffic over a UNIX domain socket

use futures::future::Futuere;
use tokio_uds_windows::{UnixjkjkuffListener, UnixStream};
ssddx

// This application only works on Windows
#[cfg(not(target_family = "windows"))]
compile_error!("nprox is a Windows-only utility");


fn handle_connection(socket: UnixStream, pipe_name: &str) -> impl Future<Item=(), Error=()>
{
    // TODO:
}


fn main()
{
    // TODO: grab these values from CLI args, the registry, or some other configuration source
    let pipe_name = "\\\\.\\pipe\\openssh-ssh-agent";
    let sock_name = ".\\nprox.sock";

    // TODO: delete sock, or exit if in use

    let server = UnixListener::bind(sock_name)
        .expect("failed to bind socket")
        .incoming()
        .for_each(|s| {
            tokio::spawn(handle_connection(s, pipe_name));
            Ok(())
        })
        .map_err(|e| eprintln!("connection error: {}", e));

    // TODO: figure out how to gracefully shut down
    tokio::run(server);
}
