use std::ffi::CString;
use windows::core::{PCSTR, PCWSTR};

#[allow(unused)]
pub(crate) trait AsPCSTR {
    fn as_pcstr(&self) -> PCSTR;
}

impl AsPCSTR for CString {
    fn as_pcstr(&self) -> PCSTR {
        PCSTR(self.as_bytes_with_nul().as_ptr())
    }
}
