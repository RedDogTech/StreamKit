use std::{error::Error, fmt::{Display, Formatter, self}};
//use thiserror::Error;
use anyhow::Result;
use libc::c_int;
use libsrt_sys;

#[derive(Clone, Copy, Debug)]
pub enum SrtError {
    Unknown,
    Success,
    ConnSetup,
    NoServer,
    ConnRej(SrtRejectReason),
    SockFail,
    SecFail,
    Closed,
    ConnFail,
    ConnLost,
    NoConn,
    Resource,
    Thread,
    NoBuf,
    SysObj,
    File,
    InvRdOff,
    RdPerm,
    InvWrOff,
    WrPerm,
    InvOp,
    BoundSock,
    ConnSock,
    InvParam,
    InvSock,
    UnboundSock,
    NoListen,
    RdvNoServ,
    RdvUnbound,
    InvalMsgApi,
    InvalBufferApi,
    DupListen,
    LargeMsg,
    InvPollId,
    PollEmpty,
    AsyncFail,
    AsyncSnd,
    AsyncRcv,
    Timeout,
    Congest,
    PeerErr,
}

#[derive(Clone, Copy, Debug)]
pub enum SrtRejectReason {
    Unknown,    // initial set when in progress
    System,     // broken due to system function error
    Peer,       // connection was rejected by peer
    Resource,   // internal problem with resource allocation
    Rogue,      // incorrect data in handshake messages
    Backlog,    // listener's backlog exceeded
    IPE,        // internal program error
    Close,      // socket is closing
    Version,    // peer is older version than agent's minimum set
    RdvCookie,  // rendezvous cookie collision
    BadSecret,  // wrong password
    Unsecure,   // password required or unexpected
    MessageAPI, // streamapi/messageapi collision
    Congestion, // incompatible congestion-controller type
    Filter,     // incompatible packet filter
    Group,      // incompatible group
    Timeout,    // connection timeout
}

fn error_msg(err: &SrtError) -> String {
    match err {
        SrtError::Unknown => "Internal error when setting the right error code".to_string(),
        SrtError::Success => "The value set when the last error was cleared and no error has occurred since then".to_string(),
        SrtError::ConnSetup => "General setup error resulting from internal system state".to_string(),
        SrtError::NoServer => "Connection timed out while attempting to connect to the remote address".to_string(),
        SrtError::ConnRej(reason) => format!("Connection has been rejected: {:?}", reason),
        SrtError::SockFail => "An error occurred when trying to call a system function on an internally used UDP socket".to_string(),
        SrtError::SecFail => "A possible tampering with the handshake packets was detected, or encryption request wasn't properly fulfilled.".to_string(),
        SrtError::Closed => "A socket that was vital for an operation called in blocking mode has been closed during the operation".to_string(),
        SrtError::ConnFail => "General connection failure of unknown details".to_string(),
        SrtError::ConnLost => "The socket was properly connected, but the connection has been broken".to_string(),
        SrtError::NoConn => "The socket is not connected".to_string(),
        SrtError::Resource => "System or standard library error reported unexpectedly for unknown purpose".to_string(),
        SrtError::Thread => "System was unable to spawn a new thread when requried".to_string(),
        SrtError::NoBuf => "System was unable to allocate memory for buffers".to_string(),
        SrtError::SysObj => "System was unable to allocate system specific objects".to_string(),
        SrtError::File => "General filesystem error (for functions operating with file transmission)".to_string(),
        SrtError::InvRdOff => "Failure when trying to read from a given position in the file".to_string(),
        SrtError::RdPerm => "Read permission was denied when trying to read from file".to_string(),
        SrtError::InvWrOff => "Failed to set position in the written file".to_string(),
        SrtError::WrPerm => "Write permission was denied when trying to write to a file".to_string(),
        SrtError::InvOp => "Invalid operation performed for the current state of a socket".to_string(),
        SrtError::BoundSock => "The socket is currently bound and the required operation cannot be performed in this state".to_string(),
        SrtError::ConnSock => "The socket is currently connected and therefore performing the required operation is not possible".to_string(),
        SrtError::InvParam => "Call parameters for API functions have some requirements that were not satisfied".to_string(),
        SrtError::InvSock => "The API function required an ID of an entity (socket or group) and it was invalid".to_string(),
        SrtError::UnboundSock => "The operation to be performed on a socket requires that it first be explicitly bound".to_string(),
        SrtError::NoListen => "The socket passed for the operation is required to be in the listen state".to_string(),
        SrtError::RdvNoServ => "The required operation cannot be performed when the socket is set to rendezvous mode".to_string(),
        SrtError::RdvUnbound => "An attempt was made to connect to a socket set to rendezvous mode that was not first bound".to_string(),
        SrtError::InvalMsgApi => "The function was used incorrectly in the message API".to_string(),
        SrtError::InvalBufferApi => "The function was used incorrectly in the stream (buffer) API".to_string(),
        SrtError::DupListen => "The port tried to be bound for listening is already busy".to_string(),
        SrtError::LargeMsg => "Size exceeded".to_string(),
        SrtError::InvPollId => "The epoll ID passed to an epoll function is invalid".to_string(),
        SrtError::PollEmpty => "The epoll container currently has no subscribed sockets".to_string(),
        SrtError::AsyncFail => "General asynchronous failure (not in use currently)".to_string(),
        SrtError::AsyncSnd => "Sending operation is not ready to perform".to_string(),
        SrtError::AsyncRcv => "Receiving operation is not ready to perform".to_string(),
        SrtError::Timeout => "The operation timed out".to_string(),
        SrtError::Congest => "With SRTO_TSBPDMODE and SRTO_TLPKTDROP set to true, some packets were dropped by sender".to_string(),
        SrtError::PeerErr => "Receiver peer is writing to a file that the agent is sending".to_string(),
    }
}

impl From<libsrt_sys::SRT_ERRNO> for SrtError {
    fn from(err_no: libsrt_sys::SRT_ERRNO) -> Self {
        match err_no {
            libsrt_sys::SRT_ERRNO::SRT_EUNKNOWN => SrtError::Unknown,
            libsrt_sys::SRT_ERRNO::SRT_SUCCESS => SrtError::Success,
            libsrt_sys::SRT_ERRNO::SRT_ECONNSETUP => SrtError::ConnSetup,
            libsrt_sys::SRT_ERRNO::SRT_ENOSERVER => SrtError::NoServer,
            libsrt_sys::SRT_ERRNO::SRT_ECONNREJ => SrtError::ConnRej(SrtRejectReason::Unknown),
            libsrt_sys::SRT_ERRNO::SRT_ESOCKFAIL => SrtError::SockFail,
            libsrt_sys::SRT_ERRNO::SRT_ESECFAIL => SrtError::SecFail,
            libsrt_sys::SRT_ERRNO::SRT_ESCLOSED => SrtError::Closed,
            libsrt_sys::SRT_ERRNO::SRT_ECONNFAIL => SrtError::ConnFail,
            libsrt_sys::SRT_ERRNO::SRT_ECONNLOST => SrtError::ConnLost,
            libsrt_sys::SRT_ERRNO::SRT_ENOCONN => SrtError::NoConn,
            libsrt_sys::SRT_ERRNO::SRT_ERESOURCE => SrtError::Resource,
            libsrt_sys::SRT_ERRNO::SRT_ETHREAD => SrtError::Thread,
            libsrt_sys::SRT_ERRNO::SRT_ENOBUF => SrtError::NoBuf,
            libsrt_sys::SRT_ERRNO::SRT_ESYSOBJ => SrtError::SysObj,
            libsrt_sys::SRT_ERRNO::SRT_EFILE => SrtError::File,
            libsrt_sys::SRT_ERRNO::SRT_ERDPERM => SrtError::RdPerm,
            libsrt_sys::SRT_ERRNO::SRT_EINVWROFF => SrtError::InvWrOff,
            libsrt_sys::SRT_ERRNO::SRT_EINVRDOFF => SrtError::InvRdOff,
            libsrt_sys::SRT_ERRNO::SRT_EWRPERM => SrtError::WrPerm,
            libsrt_sys::SRT_ERRNO::SRT_EINVOP => SrtError::InvOp,
            libsrt_sys::SRT_ERRNO::SRT_EBOUNDSOCK => SrtError::BoundSock,
            libsrt_sys::SRT_ERRNO::SRT_ECONNSOCK => SrtError::ConnSock,
            libsrt_sys::SRT_ERRNO::SRT_EINVPARAM => SrtError::InvParam,
            libsrt_sys::SRT_ERRNO::SRT_EINVSOCK => SrtError::InvSock,
            libsrt_sys::SRT_ERRNO::SRT_EUNBOUNDSOCK => SrtError::UnboundSock,
            libsrt_sys::SRT_ERRNO::SRT_ENOLISTEN => SrtError::NoListen,
            libsrt_sys::SRT_ERRNO::SRT_ERDVNOSERV => SrtError::RdvNoServ,
            libsrt_sys::SRT_ERRNO::SRT_ERDVUNBOUND => SrtError::RdvUnbound,
            libsrt_sys::SRT_ERRNO::SRT_EINVALMSGAPI => SrtError::InvalMsgApi,
            libsrt_sys::SRT_ERRNO::SRT_EINVALBUFFERAPI => SrtError::InvalBufferApi,
            libsrt_sys::SRT_ERRNO::SRT_EDUPLISTEN => SrtError::DupListen,
            libsrt_sys::SRT_ERRNO::SRT_ELARGEMSG => SrtError::LargeMsg,
            libsrt_sys::SRT_ERRNO::SRT_EINVPOLLID => SrtError::InvPollId,
            libsrt_sys::SRT_ERRNO::SRT_EPOLLEMPTY => SrtError::PollEmpty,
            libsrt_sys::SRT_ERRNO::SRT_EASYNCFAIL => SrtError::AsyncFail,
            libsrt_sys::SRT_ERRNO::SRT_EASYNCSND => SrtError::AsyncSnd,
            libsrt_sys::SRT_ERRNO::SRT_EASYNCRCV => SrtError::AsyncRcv,
            libsrt_sys::SRT_ERRNO::SRT_ETIMEOUT => SrtError::Timeout,
            libsrt_sys::SRT_ERRNO::SRT_ECONGEST => SrtError::Congest,
            libsrt_sys::SRT_ERRNO::SRT_EPEERERR => SrtError::PeerErr,
            _ => unreachable!("unrecognized error no"),
        }
    }
}

impl Error for SrtError {}

impl Display for SrtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", error_msg(self))
    }
}

pub fn handle_result<T>(ok: T, return_code: i32) -> Result<T, anyhow::Error> {
    match return_code {
        0 => Ok(ok),
        -1 => Err(get_last_error().into()),
        e => unreachable!("unrecognized return code {}", e),
    }
}

pub fn get_last_error() -> SrtError {
    let mut _errno_loc = 0;
    let err_no = unsafe { libsrt_sys::srt_getlasterror(&mut _errno_loc as *mut c_int) };
    let err = libsrt_sys::SRT_ERRNO::try_from(err_no).unwrap();
    SrtError::from(err)
}