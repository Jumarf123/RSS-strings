use std::cmp::min;
use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

use super::regions::MemoryRegion;

pub fn scan_region_chunks<F>(
    handle: HANDLE,
    region: &MemoryRegion,
    chunk_size: usize,
    overlap: usize,
    cancel: &AtomicBool,
    mut on_chunk: F,
) where
    F: FnMut(u64, &[u8]),
{
    let mut offset: usize = 0;
    let overlap = overlap.min(chunk_size.saturating_sub(1));

    while offset < region.size {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let remaining = region.size - offset;
        let to_read = min(chunk_size, remaining);
        if to_read == 0 {
            break;
        }

        let mut buffer = vec![0u8; to_read];
        let mut bytes_read = 0usize;

        let read_result = unsafe {
            // SAFETY: base address + offset is within the region; buffer is valid.
            ReadProcessMemory(
                handle,
                (region.base_address + offset as u64) as *const _,
                buffer.as_mut_ptr() as *mut _,
                to_read,
                Some(&mut bytes_read),
            )
        };

        if read_result.is_err() || bytes_read == 0 {
            offset = offset.saturating_add(to_read.max(0x1000));
            continue;
        }

        buffer.truncate(bytes_read);
        on_chunk(region.base_address + offset as u64, &buffer);

        let step = to_read.saturating_sub(overlap).max(1);
        offset = offset.saturating_add(step);
    }
}
