use std::collections::HashMap;
use std::path::Path;

use itertools::Itertools;
use memory_amount::MemoryAmount;

use crate::errors::{AsyncError, ParseError, SyncError};

pub mod errors;

pub struct MemInfo {
    pub total: MemoryAmount,
    pub free: MemoryAmount,
    pub available: MemoryAmount,
    other_entries: HashMap<String, String>,
}


pub async fn async_current_meminfo(path: impl AsRef<Path>) -> Result<MemInfo, AsyncError> {
    let meminfo_string = tokio::fs::read_to_string(path.as_ref()).await?;
    Ok(parse_meminfo(meminfo_string)?)
}

pub fn sync_current_meminfo(path: impl AsRef<Path>) -> Result<MemInfo, SyncError> {
    let meminfo_string = std::fs::read_to_string(path.as_ref())?;
    Ok(parse_meminfo(meminfo_string)?)
}


pub fn parse_meminfo(meminfo: String) -> Result<MemInfo, ParseError> {
    let lines = meminfo.split("\n");
    let mut entries = HashMap::new();
    for line in lines {
        if line.is_empty(){
            continue
        }
        match line.split(":").next_tuple() {
            Some((entry_name, entry_value)) => {
                entries.entry(entry_name.to_string()).or_insert(entry_value.to_string());
            }
            None => {
                return Err(ParseError::MultipleColonsPerLine { line: line.to_string() });
            }
        }
    }
    let mem_total_string = entries.remove(&"MemTotal".to_string()).ok_or(ParseError::MissingTotal)?;
    let free_string = entries.remove(&"MemFree".to_string()).ok_or(ParseError::MissingFree)?;
    let available_string = entries.remove(&"MemAvailable".to_string()).ok_or(ParseError::MissingAvailable)?;

    Ok(MemInfo {
        total: MemoryAmount::parse(mem_total_string)?,
        free: MemoryAmount::parse(free_string)?,
        available: MemoryAmount::parse(available_string)?,
        other_entries: entries,
    })
}


#[cfg(test)]
pub mod test {
    use crate::parse_meminfo;

    const SAMPLE_MEMINFO: &str = "MemTotal:       32846784 kB
MemFree:         6871196 kB
MemAvailable:   12433644 kB
Buffers:          757024 kB
Cached:          5176524 kB
SwapCached:       100552 kB
Active:          3117952 kB
Inactive:       19971396 kB
Active(anon):      99348 kB
Inactive(anon): 17348780 kB
Active(file):    3018604 kB
Inactive(file):  2622616 kB
Unevictable:        2788 kB
Mlocked:            2788 kB
SwapTotal:      51278844 kB
SwapFree:       50988728 kB
Dirty:             25032 kB
Writeback:             0 kB
AnonPages:      16937660 kB
Mapped:          1306532 kB
Shmem:            290096 kB
KReclaimable:     390332 kB
Slab:            1509524 kB
SReclaimable:     390332 kB
SUnreclaim:      1119192 kB
KernelStack:       30880 kB
PageTables:       103676 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:    67702236 kB
Committed_AS:   30556340 kB
VmallocTotal:   34359738367 kB
VmallocUsed:      411904 kB
VmallocChunk:          0 kB
Percpu:             8864 kB
HardwareCorrupted:     0 kB
AnonHugePages:  11180032 kB
ShmemHugePages:        0 kB
ShmemPmdMapped:        0 kB
FileHugePages:         0 kB
FilePmdMapped:         0 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
DirectMap4k:     8841924 kB
DirectMap2M:    24676352 kB
DirectMap1G:           0 kB
";

    #[test]
    pub fn test_parse() {
        let parsed = parse_meminfo(SAMPLE_MEMINFO.to_string()).unwrap();
        assert_eq!(parsed.total.kilobytes(),32846784);
        assert_eq!(parsed.free.kilobytes(),6871196);
        assert_eq!(parsed.available.kilobytes(),12433644);
    }
}