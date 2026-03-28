use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;

use anyhow::Result;
use flume::{Receiver, Sender};
use rayon::prelude::*;
use thiserror::Error;
use windows::Win32::System::Threading::{
    PROCESS_ACCESS_RIGHTS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};

use crate::process::ProcessHandle;

pub mod extract;
pub mod reader;
pub mod regions;
pub mod search;

use extract::{extract_ascii_strings, extract_utf16_strings};
use reader::scan_region_chunks;
use regions::{MemoryRegion, RegionFilter, collect_regions};
use search::AhoMatcher;

const DEFAULT_CHUNK_SIZE: usize = 4 * 1024 * 1024;
const MAX_RESULTS: usize = 200_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Encoding {
    Ascii,
    Utf16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RegionKind {
    Private,
    Image,
    Mapped,
    Other,
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub query: String,
    pub matched: String,
    pub address: u64,
    pub region: RegionKind,
    pub encoding: Encoding,
}

#[derive(Clone, Debug, Default)]
pub struct ScannerProgress {
    pub processed_regions: usize,
    pub total_regions: usize,
    pub matches: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SearchSettings {
    pub min_length: usize,
    pub detect_unicode: bool,
    pub extended_unicode: bool,
    pub include_private: bool,
    pub include_image: bool,
    pub include_mapped: bool,
}

impl Default for SearchSettings {
    fn default() -> Self {
        Self {
            min_length: 4,
            detect_unicode: true,
            extended_unicode: false,
            include_private: true,
            include_image: true,
            include_mapped: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum StartScanError {
    #[error("Query list is empty")]
    EmptyList,
    #[error("No valid search strings")]
    NoValidQueries,
    #[error("Failed to build Aho-Corasick matcher")]
    MatcherBuild(#[source] anyhow::Error),
    #[error("Failed to open process {pid}")]
    OpenProcess {
        pid: u32,
        #[source]
        source: anyhow::Error,
    },
    #[error("Failed to enumerate memory regions")]
    Regions(#[source] anyhow::Error),
}

#[derive(Clone, Debug)]
pub struct ScanRequest {
    pub pid: u32,
    pub process_name: String,
    pub queries: Vec<String>,
    pub settings: SearchSettings,
}

#[derive(Clone, Debug)]
pub enum ScannerEvent {
    Progress(ScannerProgress),
    Results(Vec<SearchResult>),
    Finished { cancelled: bool },
}

pub struct ScannerHandle {
    pub receiver: Receiver<ScannerEvent>,
    cancel: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl ScannerHandle {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}

impl Drop for ScannerHandle {
    fn drop(&mut self) {
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

pub fn start_scan(request: ScanRequest) -> Result<ScannerHandle, StartScanError> {
    if request.queries.is_empty() {
        return Err(StartScanError::EmptyList);
    }

    let matcher = AhoMatcher::new(&request.queries).map_err(StartScanError::MatcherBuild)?;
    if matcher.patterns_len() == 0 {
        return Err(StartScanError::NoValidQueries);
    }

    let access = PROCESS_ACCESS_RIGHTS(PROCESS_QUERY_INFORMATION.0 | PROCESS_VM_READ.0);
    let process_handle = ProcessHandle::open(request.pid, access).map_err(|err| {
        StartScanError::OpenProcess {
            pid: request.pid,
            source: err,
        }
    })?;

    let region_filter = RegionFilter {
        include_private: request.settings.include_private,
        include_image: request.settings.include_image,
        include_mapped: request.settings.include_mapped,
    };

    let regions =
        collect_regions(process_handle.raw(), &region_filter).map_err(StartScanError::Regions)?;

    let (tx, rx) = flume::unbounded();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let cancel_for_thread = cancel_flag.clone();
    let settings = request.settings.clone();
    let process_name = request.process_name.clone();
    let join = thread::spawn(move || {
        let _ = run_scan(
            process_handle,
            matcher,
            regions,
            settings,
            tx.clone(),
            cancel_for_thread,
            process_name,
        );
    });

    Ok(ScannerHandle {
        receiver: rx,
        cancel: cancel_flag,
        join: Some(join),
    })
}

fn run_scan(
    process_handle: ProcessHandle,
    matcher: AhoMatcher,
    regions: Vec<MemoryRegion>,
    settings: SearchSettings,
    tx: Sender<ScannerEvent>,
    cancel: Arc<AtomicBool>,
    _process_name: String,
) -> Result<()> {
    let total_regions = regions.len();
    let progress_counter = AtomicUsize::new(0);
    let matches_counter = AtomicUsize::new(0);

    let handle = Arc::new(process_handle);

    regions.par_iter().for_each(|region| {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let mut region_results =
            scan_single_region(Arc::clone(&handle), region, &matcher, &settings, &cancel);

        if !region_results.is_empty() {
            matches_counter.fetch_add(region_results.len(), Ordering::SeqCst);
            let _ = tx.send(ScannerEvent::Results(std::mem::take(&mut region_results)));
        }

        let done = progress_counter.fetch_add(1, Ordering::SeqCst) + 1;
        if done % 4 == 0 || done == total_regions {
            let _ = tx.send(ScannerEvent::Progress(ScannerProgress {
                processed_regions: done,
                total_regions,
                matches: matches_counter.load(Ordering::Relaxed),
            }));
        }
    });

    let _ = tx.send(ScannerEvent::Progress(ScannerProgress {
        processed_regions: total_regions,
        total_regions,
        matches: matches_counter.load(Ordering::Relaxed),
    }));
    let cancelled = cancel.load(Ordering::Relaxed);
    let _ = tx.send(ScannerEvent::Finished { cancelled });
    Ok(())
}

fn scan_single_region(
    handle: Arc<ProcessHandle>,
    region: &MemoryRegion,
    matcher: &AhoMatcher,
    settings: &SearchSettings,
    cancel: &AtomicBool,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();
    let overlap = matcher.max_pattern_len().saturating_sub(1);

    scan_region_chunks(
        handle.raw(),
        region,
        DEFAULT_CHUNK_SIZE,
        overlap,
        cancel,
        |chunk_base, data| {
            if cancel.load(Ordering::Relaxed) || results.len() >= MAX_RESULTS {
                return;
            }
            push_matches(
                matcher,
                data,
                chunk_base,
                region,
                settings,
                &mut results,
                &mut seen,
            );
        },
    );

    results
}

#[derive(Hash, PartialEq, Eq)]
struct ResultKey {
    addr: u64,
    pattern_idx: usize,
    encoding: Encoding,
}

fn push_matches(
    matcher: &AhoMatcher,
    data: &[u8],
    chunk_base: u64,
    region: &MemoryRegion,
    settings: &SearchSettings,
    results: &mut Vec<SearchResult>,
    seen: &mut HashSet<ResultKey>,
) {
    for extracted in extract_ascii_strings(data, settings.min_length) {
        let lower = extracted.text.to_ascii_lowercase();
        let matches = matcher.find_indices(&lower);
        for idx in matches {
            let addr = chunk_base + extracted.offset as u64;
            let key = ResultKey {
                addr,
                pattern_idx: idx,
                encoding: Encoding::Ascii,
            };
            if seen.insert(key) {
                results.push(SearchResult {
                    query: matcher.pattern(idx).to_string(),
                    matched: extracted.text.clone(),
                    address: addr,
                    region: region.kind,
                    encoding: Encoding::Ascii,
                });
                if results.len() >= MAX_RESULTS {
                    return;
                }
            }
        }
    }

    if !settings.detect_unicode {
        return;
    }

    for extracted in extract_utf16_strings(data, settings.min_length, settings.extended_unicode) {
        let lower = extracted.text.to_ascii_lowercase();
        let matches = matcher.find_indices(&lower);
        for idx in matches {
            let addr = chunk_base + extracted.offset as u64;
            let key = ResultKey {
                addr,
                pattern_idx: idx,
                encoding: Encoding::Utf16,
            };
            if seen.insert(key) {
                results.push(SearchResult {
                    query: matcher.pattern(idx).to_string(),
                    matched: extracted.text.clone(),
                    address: addr,
                    region: region.kind,
                    encoding: Encoding::Utf16,
                });
                if results.len() >= MAX_RESULTS {
                    return;
                }
            }
        }
    }
}
