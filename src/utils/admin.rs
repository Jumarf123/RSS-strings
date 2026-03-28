use anyhow::{Context, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, GetTokenInformation, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION, TOKEN_PRIVILEGES, TOKEN_PRIVILEGES_ATTRIBUTES,
    TOKEN_QUERY, TokenElevation,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
use windows::core::{PCWSTR, w};

use crate::i18n::{Language, UiText};

pub fn ensure_admin() -> Result<()> {
    if is_elevated()? {
        return Ok(());
    }

    let text = UiText::new(Language::from_system_or_english());
    let _ = message_box(
        text.admin_title(),
        text.admin_body(),
    );
    std::process::exit(1);
}

pub fn is_elevated() -> Result<bool> {
    let mut token: HANDLE = HANDLE::default();
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
            .context("OpenProcessToken failed")?;
    }
    let mut elevation = TOKEN_ELEVATION::default();
    let mut ret_len = 0u32;
    unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        )
        .context("GetTokenInformation(TokenElevation) failed")?;
        let _ = CloseHandle(token);
    }
    Ok(elevation.TokenIsElevated != 0)
}

pub fn enable_debug_privilege(enable: bool) -> Result<()> {
    let mut token: HANDLE = HANDLE::default();
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        )
        .context("OpenProcessToken (adjust) failed")?;
    }

    let mut luid = LUID::default();
    unsafe {
        LookupPrivilegeValueW(None, w!("SeDebugPrivilege"), &mut luid)
            .context("LookupPrivilegeValueW failed")?;
    }

    let attributes: TOKEN_PRIVILEGES_ATTRIBUTES = if enable {
        SE_PRIVILEGE_ENABLED
    } else {
        TOKEN_PRIVILEGES_ATTRIBUTES(0)
    };
    let tp = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [windows::Win32::Security::LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: attributes,
        }],
    };

    unsafe {
        AdjustTokenPrivileges(token, false, Some(&tp), 0, None, None)
            .context("AdjustTokenPrivileges failed")?;
        let _ = CloseHandle(token);
    }

    Ok(())
}

fn message_box(title: &str, body: &str) -> Result<()> {
    let title_w: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let body_w: Vec<u16> = body.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        // SAFETY: pointers are valid and null-terminated.
        MessageBoxW(
            None,
            PCWSTR::from_raw(body_w.as_ptr()),
            PCWSTR::from_raw(title_w.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
    Ok(())
}
