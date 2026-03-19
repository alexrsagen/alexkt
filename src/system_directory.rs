use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Foundation::{GetLastError, MAX_PATH};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::core::Error;

pub fn get_system_directory() -> Result<OsString, Error> {
	let mut buf = [0u16; MAX_PATH as usize + 1];
	let len = unsafe { GetSystemDirectoryW(Some(&mut buf)) } as usize;
	if len == 0 {
		return Err(unsafe { GetLastError() }.to_hresult().into());
	}
	Ok(OsString::from_wide(&buf[..len]))
}
