use anyhow::{Result, anyhow};
use std::mem::size_of;
use windows::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::SystemInformation::{
    IMAGE_FILE_MACHINE, IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64,
    IMAGE_FILE_MACHINE_UNKNOWN,
};
use windows::Win32::System::Threading::{
    IsWow64Process, IsWow64Process2, OpenProcess, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::core::BOOL;

use super::handle::ProcessHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessStatus {
    Ok,
    AccessDenied,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub is_64_bit: bool,
    pub status: ProcessStatus,
}

pub fn enumerate_processes() -> Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };
    if snapshot.is_invalid() {
        return Ok(processes);
    }

    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut ok = unsafe { Process32FirstW(snapshot, &mut entry).is_ok() };
    while ok {
        let pid = entry.th32ProcessID;
        if pid != 0 {
            processes.push(process_info_from_entry(&entry));
        }
        ok = unsafe { Process32NextW(snapshot, &mut entry).is_ok() };
    }

    unsafe {
        let _ = CloseHandle(snapshot);
    }

    processes.sort_by_key(|p| p.pid);
    Ok(processes)
}

fn process_info_from_entry(entry: &PROCESSENTRY32W) -> ProcessInfo {
    let pid = entry.th32ProcessID;
    let name = utf16_to_string(&entry.szExeFile);
    let access = PROCESS_QUERY_LIMITED_INFORMATION;

    let raw_handle = unsafe { OpenProcess(access, false, pid) };
    if let Err(err) = raw_handle {
        let status = if err.code() == ERROR_ACCESS_DENIED.to_hresult() {
            ProcessStatus::AccessDenied
        } else {
            ProcessStatus::Unknown
        };
        return ProcessInfo {
            pid,
            name,
            path: None,
            is_64_bit: false,
            status,
        };
    }

    let handle = unsafe { ProcessHandle::from_raw(raw_handle.unwrap()) };
    let raw = handle.raw();
    let path = query_process_path(raw);
    let is_64_bit = is_process_64_bit(raw).unwrap_or(false);

    ProcessInfo {
        pid,
        name,
        path,
        is_64_bit,
        status: ProcessStatus::Ok,
    }
}

fn query_process_path(handle: HANDLE) -> Option<String> {
    let mut buf = vec![0u16; 260];
    let mut size = buf.len() as u32;
    if unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
    }
    .is_err()
    {
        return None;
    }
    if size == 0 {
        return None;
    }
    buf.truncate(size as usize);
    String::from_utf16(&buf).ok()
}

fn is_process_64_bit(handle: HANDLE) -> Result<bool> {
    unsafe {
        let mut process_machine = IMAGE_FILE_MACHINE::default();
        let mut native_machine = IMAGE_FILE_MACHINE::default();
        if IsWow64Process2(handle, &mut process_machine, Some(&mut native_machine)).is_ok() {
            let native = native_machine;
            let wow = process_machine;
            let native_is_64 =
                native == IMAGE_FILE_MACHINE_AMD64 || native == IMAGE_FILE_MACHINE_ARM64;
            return Ok(native_is_64 && wow == IMAGE_FILE_MACHINE_UNKNOWN);
        }

        let mut wow64 = BOOL::default();
        if IsWow64Process(handle, &mut wow64).is_ok() {
            return Ok(!wow64.as_bool());
        }
    }
    Err(anyhow!("IsWow64Process failed"))
}

fn utf16_to_string(buf: &[u16]) -> String {
    let nul_pos = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..nul_pos])
}
