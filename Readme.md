# WSL Agent Bridge

This utility allows [the Windows port of ssh-agent](https://github.com/PowerShell/Win32-OpenSSH/wiki)
to be used by WSL programs.

It works by creating a WSL-accessible Unix socket that can be used as `$SSH_AUTH_SOCK`. All
connections are simply redirected to the true ssh-agent process via
[its named pipe](https://github.com/PowerShell/Win32-OpenSSH/wiki/About-Win32-OpenSSH-and-Design-Details#af_unix-domain-sockets).

## Usage

To use, simply run *wsl-agent-bridge.exe*. No Arguments are required.

By default, the bridge socket is created at the path `$HOME\AppData\Local\ssh-agent.sock`. This path
may be customized by setting the `$SSH_AUTH_SOCK` environment variable (in Windows).

## Installation

To install the program, you need [Git](https://git-scm.com/) and [Rust](https://rustup.rs/).

```powershell
git clone https://github.com/reynoldsbd/wsl-agent-bridge
cd wsl-agent-bridge
cargo install --path .
```

You may also wish to schedule a task to start the bridge at login. Just search for "Task Scheduler"
in the start menu.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.