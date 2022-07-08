use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Process {
    pub process_id: String,
    pub exe: String,
    pub username: String,
    pub realname: String,
    pub computer: String,
    pub distro: String,
    pub cpu_brand: String,
    pub tsc_frequency: u64,
    pub start_time: String, // RFC3339
    pub start_ticks: i64,
    pub parent_process_id: String,
}

impl TryFrom<crate::api::components::Process> for Process {
    type Error = anyhow::Error;

    fn try_from(process: crate::api::components::Process) -> Result<Self> {
        Ok(Process {
            process_id: process.process_id,
            exe: process.exe,
            username: process.username,
            realname: process.realname,
            computer: process.computer,
            distro: process.distro,
            cpu_brand: process.cpu_brand,
            tsc_frequency: process.tsc_frequency.parse()?,
            start_time: process.start_time,
            start_ticks: process.start_ticks.parse()?,
            parent_process_id: process.parent_process_id,
        })
    }
}

impl From<Process> for crate::api::components::Process {
    fn from(process: Process) -> Self {
        crate::api::components::Process {
            process_id: process.process_id,
            exe: process.exe,
            username: process.username,
            realname: process.realname,
            computer: process.computer,
            distro: process.distro,
            cpu_brand: process.cpu_brand,
            tsc_frequency: process.tsc_frequency.to_string(),
            start_time: process.start_time,
            start_ticks: process.start_ticks.to_string(),
            parent_process_id: process.parent_process_id,
        }
    }
}
