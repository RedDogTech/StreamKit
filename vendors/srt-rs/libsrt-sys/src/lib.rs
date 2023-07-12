#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

use libc::{c_char as char, c_int as int, c_void as void, sockaddr, sockaddr_storage};

pub type SRTSOCKET = int;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SRT_SOCKOPT {
    SRTO_SNDSYN = 1,
    SRTO_RCVSYN = 2,
    SRTO_STREAMID = 46
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum SRT_EPOLL_OPT {
    SRT_EPOLL_OPT_NONE = 0x0, // fallback

    // Values intended to be the same as in `<sys/epoll.h>`.
    // so that if system values are used by mistake, they should have the same effect
    // This applies to: IN, OUT, ERR and ET.
    /// Ready for 'recv' operation:
    ///
    /// - For stream mode it means that at least 1 byte is available.
    /// In this mode the buffer may extract only a part of the packet,
    /// leaving next data possible for extraction later.
    ///
    /// - For message mode it means that there is at least one packet
    /// available (this may change in future, as it is desired that
    /// one full message should only wake up, not single packet of a
    /// not yet extractable message).
    ///
    /// - For live mode it means that there's at least one packet
    /// ready to play.
    ///
    /// - For listener sockets, this means that there is a new connection
    /// waiting for pickup through the `srt_accept()` call, that is,
    /// the next call to `srt_accept()` will succeed without blocking
    /// (see an alias SRT_EPOLL_ACCEPT below).
    SRT_EPOLL_IN = 0x1,

    /// Ready for 'send' operation.
    ///
    /// - For stream mode it means that there's a free space in the
    /// sender buffer for at least 1 byte of data. The next send
    /// operation will only allow to send as much data as it is free
    /// space in the buffer.
    ///
    /// - For message mode it means that there's a free space for at
    /// least one UDP packet. The edge-triggered mode can be used to
    /// pick up updates as the free space in the sender buffer grows.
    ///
    /// - For live mode it means that there's a free space for at least
    /// one UDP packet. On the other hand, no readiness for OUT usually
    /// means an extraordinary congestion on the link, meaning also that
    /// you should immediately slow down the sending rate or you may get
    /// a connection break soon.
    ///
    /// - For non-blocking sockets used with `srt_connect*` operation,
    /// this flag simply means that the connection was established.
    SRT_EPOLL_OUT = 0x4,

    /// The socket has encountered an error in the last operation
    /// and the next operation on that socket will end up with error.
    /// You can retry the operation, but getting the error from it
    /// is certain, so you may as well close the socket.
    SRT_EPOLL_ERR = 0x8,

    SRT_EPOLL_UPDATE = 0x10,
    SRT_EPOLL_ET = 1 << 31,
}

impl TryFrom<i32> for SRT_EPOLL_OPT {
    type Error = ();

    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(SRT_EPOLL_OPT::SRT_EPOLL_OPT_NONE),
            1 => Ok(SRT_EPOLL_OPT::SRT_EPOLL_IN),
            4 => Ok(SRT_EPOLL_OPT::SRT_EPOLL_OUT),
            8 => Ok(SRT_EPOLL_OPT::SRT_EPOLL_ERR),
            10 => Ok(SRT_EPOLL_OPT::SRT_EPOLL_UPDATE),
            _ => Err(()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum SRT_ERRNO {
    SRT_EUNKNOWN = -1,
    SRT_SUCCESS = 0,
    SRT_ECONNSETUP = 1000,
    SRT_ENOSERVER = 1001,
    SRT_ECONNREJ = 1002,
    SRT_ESOCKFAIL = 1003,
    SRT_ESECFAIL = 1004,
    SRT_ESCLOSED = 1005,
    SRT_ECONNFAIL = 2000,
    SRT_ECONNLOST = 2001,
    SRT_ENOCONN = 2002,
    SRT_ERESOURCE = 3000,
    SRT_ETHREAD = 3001,
    SRT_ENOBUF = 3002,
    SRT_ESYSOBJ = 3003,
    SRT_EFILE = 4000,
    SRT_EINVRDOFF = 4001,
    SRT_ERDPERM = 4002,
    SRT_EINVWROFF = 4003,
    SRT_EWRPERM = 4004,
    SRT_EINVOP = 5000,
    SRT_EBOUNDSOCK = 5001,
    SRT_ECONNSOCK = 5002,
    SRT_EINVPARAM = 5003,
    SRT_EINVSOCK = 5004,
    SRT_EUNBOUNDSOCK = 5005,
    SRT_ENOLISTEN = 5006,
    SRT_ERDVNOSERV = 5007,
    SRT_ERDVUNBOUND = 5008,
    SRT_EINVALMSGAPI = 5009,
    SRT_EINVALBUFFERAPI = 5010,
    SRT_EDUPLISTEN = 5011,
    SRT_ELARGEMSG = 5012,
    SRT_EINVPOLLID = 5013,
    SRT_EPOLLEMPTY = 5014,
    SRT_EBINDCONFLICT = 5015,
    SRT_EASYNCFAIL = 6000,
    SRT_EASYNCSND = 6001,
    SRT_EASYNCRCV = 6002,
    SRT_ETIMEOUT = 6003,
    SRT_ECONGEST = 6004,
    SRT_EPEERERR = 7000,
}

impl TryFrom<i32> for SRT_ERRNO {
    type Error = ();

    fn try_from(n: i32) -> Result<Self, Self::Error> {
        match n {
            -1 => Ok(SRT_ERRNO::SRT_EUNKNOWN),
            0 => Ok(SRT_ERRNO::SRT_SUCCESS),
            1000 => Ok(SRT_ERRNO::SRT_ECONNSETUP),
            1001 => Ok(SRT_ERRNO::SRT_ENOSERVER),
            1002 => Ok(SRT_ERRNO::SRT_ECONNREJ),
            1003 => Ok(SRT_ERRNO::SRT_ESOCKFAIL),
            1004 => Ok(SRT_ERRNO::SRT_ESECFAIL),
            1005 => Ok(SRT_ERRNO::SRT_ESCLOSED),
            2000 => Ok(SRT_ERRNO::SRT_ECONNFAIL),
            2001 => Ok(SRT_ERRNO::SRT_ECONNLOST),
            2002 => Ok(SRT_ERRNO::SRT_ENOCONN),
            3000 => Ok(SRT_ERRNO::SRT_ERESOURCE),
            3001 => Ok(SRT_ERRNO::SRT_ETHREAD),
            3002 => Ok(SRT_ERRNO::SRT_ENOBUF),
            3003 => Ok(SRT_ERRNO::SRT_ESYSOBJ),
            4000 => Ok(SRT_ERRNO::SRT_EFILE),
            4001 => Ok(SRT_ERRNO::SRT_EINVRDOFF),
            4002 => Ok(SRT_ERRNO::SRT_ERDPERM),
            4003 => Ok(SRT_ERRNO::SRT_EINVWROFF),
            4004 => Ok(SRT_ERRNO::SRT_EWRPERM),
            5000 => Ok(SRT_ERRNO::SRT_EINVOP),
            5001 => Ok(SRT_ERRNO::SRT_EBOUNDSOCK),
            5002 => Ok(SRT_ERRNO::SRT_ECONNSOCK),
            5003 => Ok(SRT_ERRNO::SRT_EINVPARAM),
            5004 => Ok(SRT_ERRNO::SRT_EINVSOCK),
            5005 => Ok(SRT_ERRNO::SRT_EUNBOUNDSOCK),
            5006 => Ok(SRT_ERRNO::SRT_ENOLISTEN),
            5007 => Ok(SRT_ERRNO::SRT_ERDVNOSERV),
            5008 => Ok(SRT_ERRNO::SRT_ERDVUNBOUND),
            5009 => Ok(SRT_ERRNO::SRT_EINVALMSGAPI),
            5010 => Ok(SRT_ERRNO::SRT_EINVALBUFFERAPI),
            5011 => Ok(SRT_ERRNO::SRT_ELARGEMSG),
            5012 => Ok(SRT_ERRNO::SRT_ELARGEMSG),
            5013 => Ok(SRT_ERRNO::SRT_EINVPOLLID),
            5014 => Ok(SRT_ERRNO::SRT_EPOLLEMPTY),
            5015 => Ok(SRT_ERRNO::SRT_EBINDCONFLICT),
            6000 => Ok(SRT_ERRNO::SRT_EASYNCFAIL),
            6001 => Ok(SRT_ERRNO::SRT_EASYNCSND),
            6002 => Ok(SRT_ERRNO::SRT_EASYNCRCV),
            6003 => Ok(SRT_ERRNO::SRT_ETIMEOUT),
            6004 => Ok(SRT_ERRNO::SRT_ECONGEST),
            7000 => Ok(SRT_ERRNO::SRT_EPEERERR),
            _ => Err(()),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct SRT_EPOLL_EVENT {
    pub fd: SRTSOCKET,
    pub events: SRT_EPOLL_OPT,
}

pub type SRT_LOG_HANDLER_FN = Option<
    unsafe extern "C" fn(
        opaque: *mut void,
        level: int,
        file: *const char,
        line: int,
        area: *const char,
        message: *const char,
    ),
>;

pub const SRT_SOCKSTATUS_SRTS_INIT: SRT_SOCKSTATUS = 1;
pub const SRT_SOCKSTATUS_SRTS_OPENED: SRT_SOCKSTATUS = 2;
pub const SRT_SOCKSTATUS_SRTS_LISTENING: SRT_SOCKSTATUS = 3;
pub const SRT_SOCKSTATUS_SRTS_CONNECTING: SRT_SOCKSTATUS = 4;
pub const SRT_SOCKSTATUS_SRTS_CONNECTED: SRT_SOCKSTATUS = 5;
pub const SRT_SOCKSTATUS_SRTS_BROKEN: SRT_SOCKSTATUS = 6;
pub const SRT_SOCKSTATUS_SRTS_CLOSING: SRT_SOCKSTATUS = 7;
pub const SRT_SOCKSTATUS_SRTS_CLOSED: SRT_SOCKSTATUS = 8;
pub const SRT_SOCKSTATUS_SRTS_NONEXIST: SRT_SOCKSTATUS = 9;
pub type SRT_SOCKSTATUS = int;

pub const SRT_MemberStatus_SRT_GST_PENDING: SRT_MemberStatus = 0;
pub const SRT_MemberStatus_SRT_GST_IDLE: SRT_MemberStatus = 1;
pub const SRT_MemberStatus_SRT_GST_RUNNING: SRT_MemberStatus = 2;
pub const SRT_MemberStatus_SRT_GST_BROKEN: SRT_MemberStatus = 3;
pub type SRT_MemberStatus = int;
pub use self::SRT_MemberStatus as SRT_MEMBERSTATUS;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SRT_SOCKGROUPDATA {
    pub id: SRTSOCKET,
    pub peeraddr: sockaddr_storage,
    pub sockstate: SRT_SOCKSTATUS,
    pub weight: u16,
    pub memberstate: SRT_MEMBERSTATUS,
    pub result: int,
    pub token: int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SRT_MSGCTRL {
    pub flags: int,
    pub msgttl: int,
    pub inorder: int,
    pub boundary: int,
    pub srctime: i64,
    pub pktseq: i32,
    pub msgno: i32,
    pub grpdata: *mut SRT_SOCKGROUPDATA,
    pub grpdata_size: usize,
}

#[link(name = "srt")]
extern "C" {
    ///
    /// Get SRT version value
    /// 
    pub fn srt_getversion()  -> int;

    ///
    /// Called at the start of an application that uses the SRT library
    /// 
    pub fn srt_startup() -> int;

    ///
    /// Cleans up global SRT resources before exiting an application
    /// 
    pub fn srt_cleanup() -> int;

    ///
    /// Creates an SRT socket.
    /// 
    pub fn srt_create_socket() -> SRTSOCKET;

    ///
    /// Closes the socket or group and frees all used resources.
    /// Note that underlying UDP sockets may be shared between sockets,
    /// so these are freed only with the last user closed.
    /// 
    pub fn srt_close(srt_socket: SRTSOCKET) -> int;

    pub fn srt_bind(u: SRTSOCKET, name: *const sockaddr, namelen: int) -> int;

    pub fn srt_listen(u: SRTSOCKET, backlog: int) -> int;

    pub fn srt_accept(u: SRTSOCKET, addr: *mut sockaddr, addrlen: *mut int) -> SRTSOCKET;

    pub fn srt_recv(souck: SRTSOCKET, buf: *mut char, len: int) -> int;

    pub fn srt_recvmsg2(u: SRTSOCKET, buf: *mut char, len: int, mctrl: *mut SRT_MSGCTRL) -> int;

    pub fn srt_getsockflag(u: SRTSOCKET, opt: SRT_SOCKOPT, optval: *mut void, optlen: *mut int) -> int;

    pub fn srt_setsockflag(u: SRTSOCKET, opt: SRT_SOCKOPT, optval: *const void, optlen: int) -> int;

    pub fn srt_getsockname(u: SRTSOCKET, name: *mut sockaddr, namelen: *mut int) -> int;

    ///
    /// Sets the minimum severity for logging. A particular log entry
    /// is displayed only if it has a severity greater than or equal 
    /// to the minimum. Setting this value to LOG_DEBUG turns on all levels.
    /// 
    pub fn srt_setloglevel(ll: int);

    pub fn srt_setloghandler(opaque: *mut void, handler: SRT_LOG_HANDLER_FN);

    pub fn srt_getlasterror_str() -> *const char;

    pub fn srt_getlasterror(errno_loc: *mut int) -> int;

    ///
    /// Creates a new epoll container
    /// 
    pub fn srt_epoll_create() -> int;

    pub fn srt_epoll_add_usock(eid: int, sid: SRTSOCKET, events: *const int) -> int;

    pub fn srt_epoll_remove_usock(eid: int, sid: SRTSOCKET) -> int;

    pub fn srt_epoll_update_usock(eid: int, sid: SRTSOCKET, events: *const int) -> int;

    pub fn srt_epoll_uwait( eid: int, fdsSet: *mut SRT_EPOLL_EVENT, fdsSize: int, msTimeOut: i64) -> int;

    pub fn srt_epoll_clear_usocks(eid: int) -> int;

    pub fn srt_epoll_release(eid: int) -> int;
}