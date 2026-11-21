//! Just enough HTTP for the update check: a GET through WinHTTP, which comes with Windows.

use crate::sys::wide;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Networking::WinHttp::*;

pub use deskpet_host::catalog::site;

/// Closes a WinHTTP handle when dropped.
struct Handle(*mut core::ffi::c_void);
impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { WinHttpCloseHandle(self.0) };
        }
    }
}

/// GETs `url`; the body of a 200 response, up to `limit` bytes.
pub fn get(url: &str, limit: usize) -> Result<Vec<u8>, String> {
    let wurl = wide(url);
    let mut host = vec![0u16; 256];
    let mut path = vec![0u16; 2048];
    let mut parts: URL_COMPONENTS = unsafe { std::mem::zeroed() };
    parts.dwStructSize = std::mem::size_of::<URL_COMPONENTS>() as u32;
    parts.lpszHostName = host.as_mut_ptr();
    parts.dwHostNameLength = host.len() as u32;
    parts.lpszUrlPath = path.as_mut_ptr();
    parts.dwUrlPathLength = path.len() as u32;
    if unsafe { WinHttpCrackUrl(wurl.as_ptr(), 0, 0, &mut parts) } == 0 {
        return Err(format!("bad address {url}"));
    }
    let host = &host[..parts.dwHostNameLength as usize];
    let path = &path[..parts.dwUrlPathLength as usize];
    let (host, path) = ([host, &[0]].concat(), [path, &[0]].concat());
    let secure = parts.nScheme == WINHTTP_INTERNET_SCHEME_HTTPS;

    unsafe {
        let agent = wide("DeskPet");
        let session = Handle(WinHttpOpen(agent.as_ptr(), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, null(), null(), 0));
        if session.0.is_null() {
            return Err("no network".into());
        }
        WinHttpSetTimeouts(session.0, 10_000, 10_000, 15_000, 30_000);
        let connection = Handle(WinHttpConnect(session.0, host.as_ptr(), parts.nPort, 0));
        let verb = wide("GET");
        let flags = if secure { WINHTTP_FLAG_SECURE } else { 0 };
        let request = Handle(WinHttpOpenRequest(connection.0, verb.as_ptr(), path.as_ptr(), null(), null(), null(), flags));
        if connection.0.is_null() || request.0.is_null() {
            return Err("couldn't connect".into());
        }
        if WinHttpSendRequest(request.0, null(), 0, null(), 0, 0, 0) == 0 || WinHttpReceiveResponse(request.0, null_mut()) == 0 {
            return Err("no answer".into());
        }
        let mut status: u32 = 0;
        let mut len = 4u32;
        WinHttpQueryHeaders(request.0, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, null(), &mut status as *mut u32 as *mut _, &mut len, null_mut());
        if status != 200 {
            return Err(format!("HTTP {status}"));
        }
        let mut body = Vec::new();
        loop {
            let mut buf = [0u8; 16384];
            let mut read = 0u32;
            if WinHttpReadData(request.0, buf.as_mut_ptr() as *mut _, buf.len() as u32, &mut read) == 0 {
                return Err("download broke off".into());
            }
            if read == 0 {
                return Ok(body);
            }
            body.extend_from_slice(&buf[..read as usize]);
            if body.len() > limit {
                return Err("too big".into());
            }
        }
    }
}
