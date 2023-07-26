use std::fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Pid {
    PAT,
    NULL,
    Other(u16),
}

impl Pid {
    #[inline(always)]
    pub fn is_section(self) -> bool {
        match self {
            Pid::Other(..) | Pid::NULL => false,
            _ => true,
        }
    }

    #[inline(always)]
    pub fn is_null(self) -> bool {
        match self {
            Pid::NULL => true,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn is_other(self) -> bool {
        match self {
            Pid::Other(..) => true,
            _ => false,
        }
    }
}

impl From<u16> for Pid {
    fn from(d: u16) -> Self {
        match d {
            0x0000 => Pid::PAT,
            0x1FFF => Pid::NULL,
            _ => Pid::Other(d),
        }
    }
}

impl From<Pid> for u16 {
    fn from(pid: Pid) -> u16 {
        match pid {
            Pid::PAT => 0x0000,
            Pid::NULL => 0x1FFF,
            Pid::Other(d) => d,
        }
    }
}

impl fmt::Debug for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            Pid::PAT => write!(f, "PAT()"),
            Pid::NULL => write!(f, "NUll()"),
            Pid::Other(id) => write!(f, "PID({:04x})", id)
        }
    }
}