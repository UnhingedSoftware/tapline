#![allow(unsafe_code)]

use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::io::FromRawHandle;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::{
    CloseHandle, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, LocalFree,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertSidToStringSidW,
    ConvertStringSecurityDescriptorToSecurityDescriptorW, ConvertStringSidToSidW,
    GetNamedSecurityInfoW, SDDL_REVISION_1, SE_FILE_OBJECT,
};
use windows_sys::Win32::Security::{
    DACL_SECURITY_INFORMATION, EqualSid, GetTokenInformation, PSECURITY_DESCRIPTOR, PSID,
    SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows_sys::Win32::Storage::FileSystem::{CREATE_ALWAYS, CreateFileW, FILE_ATTRIBUTE_NORMAL};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

const ATTRIBUTES_SIZE: u32 = size_of::<SECURITY_ATTRIBUTES>() as u32;

pub fn current_user_sid() -> io::Result<String> {
    let mut token: HANDLE = ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let mut wanted = 0_u32;
    unsafe { GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut wanted) };
    if wanted == 0 {
        let failure = io::Error::last_os_error();
        unsafe { CloseHandle(token) };
        return Err(failure);
    }

    let mut buffer = vec![0_u8; wanted as usize];
    let read = unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            wanted,
            &mut wanted,
        )
    };
    let failure = io::Error::last_os_error();
    unsafe { CloseHandle(token) };
    if read == 0 {
        return Err(failure);
    }

    let mut text: *mut u16 = ptr::null_mut();
    let converted = unsafe {
        let user = buffer.as_ptr().cast::<TOKEN_USER>();
        ConvertSidToStringSidW((*user).User.Sid, &mut text)
    };
    if converted == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { take_wide(text) })
}

pub fn create_with_dacl(path: &Path, dacl: &str) -> io::Result<File> {
    let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();
    let wide_dacl = wide(OsStr::new(dacl));
    let built = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide_dacl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            ptr::null_mut(),
        )
    };
    if built == 0 {
        return Err(io::Error::last_os_error());
    }

    let attributes = SECURITY_ATTRIBUTES {
        nLength: ATTRIBUTES_SIZE,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let wide_path = wide(path.as_os_str());
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            GENERIC_WRITE,
            0,
            &attributes,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        )
    };
    let failure = io::Error::last_os_error();
    unsafe { LocalFree(descriptor.cast()) };

    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(failure);
    }
    Ok(unsafe { File::from_raw_handle(handle.cast()) })
}

pub fn dacl_of(path: &Path) -> io::Result<String> {
    let wide_path = wide(path.as_os_str());
    let mut descriptor: PSECURITY_DESCRIPTOR = ptr::null_mut();
    let status = unsafe {
        GetNamedSecurityInfoW(
            wide_path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }

    let mut text: *mut u16 = ptr::null_mut();
    let mut length = 0_u32;
    let converted = unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            descriptor,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            &mut length,
        )
    };
    let failure = io::Error::last_os_error();
    unsafe { LocalFree(descriptor.cast()) };

    if converted == 0 {
        return Err(failure);
    }
    Ok(unsafe { take_wide(text) })
}

/// Whether an SDDL grantee names the same account as `owner`.
///
/// Windows does not render a SID back the way it was written. A descriptor read
/// from a file substitutes the two-letter alias for every SID that has one, so
/// the local administrator writes `S-1-5-21-...-500` and reads back `LA`, and
/// the built-in accounts likewise. Comparing the text refuses an account its own
/// file, so both sides are resolved to real SIDs and Windows is asked whether
/// they match. A grantee that will not resolve is a grantee we cannot vouch for,
/// and reads as somebody else.
pub fn same_account(grantee: &str, owner: &str) -> bool {
    if grantee.eq_ignore_ascii_case(owner) {
        return true;
    }
    let (Some(theirs), Some(ours)) = (sid_of(grantee), sid_of(owner)) else {
        return false;
    };
    unsafe { EqualSid(theirs.0, ours.0) != 0 }
}

/// A SID Windows allocated for us, freed when it goes out of scope.
struct Sid(PSID);

impl Drop for Sid {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0.cast()) };
    }
}

fn sid_of(text: &str) -> Option<Sid> {
    let wide_text = wide(OsStr::new(text));
    let mut sid: PSID = ptr::null_mut();
    let converted = unsafe { ConvertStringSidToSidW(wide_text.as_ptr(), &mut sid) };
    if converted == 0 {
        return None;
    }
    Some(Sid(sid))
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

unsafe fn take_wide(text: *mut u16) -> String {
    let mut length = 0_usize;
    while unsafe { *text.add(length) } != 0 {
        length = length.saturating_add(1);
    }
    let value = OsString::from_wide(unsafe { std::slice::from_raw_parts(text, length) })
        .to_string_lossy()
        .into_owned();
    unsafe { LocalFree(text.cast()) };
    value
}
