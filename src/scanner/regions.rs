use anyhow::Result;
use std::mem::size_of;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_IMAGE, MEM_MAPPED, MEM_PRIVATE, MEMORY_BASIC_INFORMATION, PAGE_GUARD,
    PAGE_NOACCESS, VirtualQueryEx,
};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

use super::RegionKind;

#[derive(Clone, Debug)]
pub struct MemoryRegion {
    pub base_address: u64,
    pub size: usize,
    pub kind: RegionKind,
    pub _protect: u32,
}

#[derive(Clone, Debug)]
pub struct RegionFilter {
    pub include_private: bool,
    pub include_image: bool,
    pub include_mapped: bool,
}

impl RegionFilter {
    pub fn allows(&self, kind: RegionKind) -> bool {
        match kind {
            RegionKind::Private => self.include_private,
            RegionKind::Image => self.include_image,
            RegionKind::Mapped => self.include_mapped,
            RegionKind::Other => false,
        }
    }
}

pub fn collect_regions(handle: HANDLE, filter: &RegionFilter) -> Result<Vec<MemoryRegion>> {
    let (min_addr, max_addr) = address_range();
    let mut current = min_addr;
    let mut regions = Vec::new();

    while current < max_addr {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();
        let res = unsafe {
            // SAFETY: buffer is valid; handle is assumed valid.
            VirtualQueryEx(
                handle,
                Some(current as *const _),
                &mut mbi,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if res == 0 {
            current = current.saturating_add(0x1000);
            continue;
        }

        let region_size = mbi.RegionSize;
        let base = mbi.BaseAddress as usize;
        let next = base.saturating_add(region_size);
        current = next;

        if region_size == 0 {
            current = current.saturating_add(0x1000);
            continue;
        }

        if mbi.State != MEM_COMMIT {
            continue;
        }
        let protect = mbi.Protect;
        if protect.0 & PAGE_NOACCESS.0 != 0 || protect.0 & PAGE_GUARD.0 != 0 {
            continue;
        }

        let kind = match mbi.Type {
            MEM_PRIVATE => RegionKind::Private,
            MEM_IMAGE => RegionKind::Image,
            MEM_MAPPED => RegionKind::Mapped,
            _ => RegionKind::Other,
        };

        if !filter.allows(kind) {
            continue;
        }

        regions.push(MemoryRegion {
            base_address: base as u64,
            size: region_size,
            kind,
            _protect: protect.0,
        });
    }

    Ok(regions)
}

fn address_range() -> (usize, usize) {
    let mut info = SYSTEM_INFO::default();
    unsafe {
        GetSystemInfo(&mut info);
    }
    (
        info.lpMinimumApplicationAddress as usize,
        info.lpMaximumApplicationAddress as usize,
    )
}
