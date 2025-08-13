use crate::Finite;

#[derive(Finite)]
#[__finite_foreign(std::alloc::System)]
struct _System;

#[derive(Finite)]
#[__finite_foreign(std::sync::mpsc::RecvTimeoutError)]
enum _RecvTimeoutError {
    Timeout,
    Disconnected,
}

#[derive(Finite)]
#[__finite_foreign(std::sync::mpsc::TryRecvError)]
enum _TryRecvError {
    Empty,
    Disconnected,
}

#[derive(Finite)]
#[__finite_foreign(std::net::Shutdown)]
enum _Shutdown {
    Read,
    Write,
    Both,
}
