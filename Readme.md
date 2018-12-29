nprox is a Windows utility that proxies data between a named pipe and a Unix domain socket. It
facilitates interop between native Windows programs using named pipes and other programs which
expect Unix domain sockets (e.g. Linux programs running under WSL).

# Usage

To use, specify the name of the pipe to proxy and the path to the Unix socket to create:

```
> nprox.exe \\.\pipe\openssh-ssh-agent .\agent.sock
```