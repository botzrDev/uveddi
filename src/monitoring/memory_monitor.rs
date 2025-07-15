//! MemoryMonitor for process-level memory usage tracking (UV-2)
//!
//! Uses sysinfo to sample memory usage before/after component analysis.

use sysinfo::{Pid, System};

pub struct MemoryMonitor {
    system: System,
    process_id: Pid,
}

impl MemoryMonitor {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_all();
        let process_id = Pid::from(std::process::id() as usize);
        Self { system, process_id }
    }

    /// Returns current memory usage in bytes for the process
    pub fn get_current_memory_usage(&mut self) -> u64 {
        self.system.refresh_process(self.process_id);
        if let Some(process) = self.system.process(self.process_id) {
            process.memory() * 1024 // sysinfo returns KB
        } else {
            0
        }
    }
}
