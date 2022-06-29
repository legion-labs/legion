use lgn_telemetry::types::Process;

#[derive(Clone, PartialEq)]
pub struct ProcessInstance {
    pub process_info: Option<Process>,
    pub child_count: u32,
    pub nb_cpu_blocks: u32,
    pub nb_log_blocks: u32,
    pub nb_metric_blocks: u32,
}

impl From<ProcessInstance> for crate::api::components::ProcessInstance {
    fn from(process: ProcessInstance) -> Self {
        Self {
            process_info: process.process_info.map(Into::into),
            child_count: process.child_count,
            nb_cpu_blocks: process.nb_cpu_blocks,
            nb_log_blocks: process.nb_log_blocks,
            nb_metric_blocks: process.nb_metric_blocks,
        }
    }
}

impl TryFrom<crate::api::components::ProcessInstance> for ProcessInstance {
    type Error = anyhow::Error;

    fn try_from(process: crate::api::components::ProcessInstance) -> anyhow::Result<Self> {
        Ok(Self {
            process_info: match process.process_info {
                Some(process_info) => Some(process_info.try_into()?),
                None => None,
            },
            child_count: process.child_count,
            nb_cpu_blocks: process.nb_cpu_blocks,
            nb_log_blocks: process.nb_log_blocks,
            nb_metric_blocks: process.nb_metric_blocks,
        })
    }
}
