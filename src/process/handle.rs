use anyhow::{Context, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS};

#[derive(Debug)]
pub struct ProcessHandle {
    handle: HANDLE,
}

impl ProcessHandle {
    pub fn open(pid: u32, access: PROCESS_ACCESS_RIGHTS) -> Result<Self> {
        let handle = unsafe { OpenProcess(access, false, pid) }
            .with_context(|| format!("Failed to open process PID={pid}"))?;
        Ok(Self { handle })
    }

    /// # Safety
    /// Caller must provide a valid process handle obtained from Windows API.
    pub(crate) unsafe fn from_raw(handle: HANDLE) -> Self {
        Self { handle }
    }

    pub fn raw(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                // SAFETY: handle was obtained from OpenProcess.
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

unsafe impl Send for ProcessHandle {}
unsafe impl Sync for ProcessHandle {}
