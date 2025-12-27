/// Progress information for a step
#[derive(Clone, Debug)]
pub struct StepProgress {
    pub step_id: String,
    pub status: StepStatus,
    pub output: String,
    pub progress_percent: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StepStatus {
    Running,
    Completed,
    Failed,
    Skipped,
}

