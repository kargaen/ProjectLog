use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct TimesheetPreview {
    pub title: String,
    pub generated_at: String,
    pub generated_at_epoch_ms: i64,
    pub sheets: Vec<TimesheetPreviewSheet>,
}

#[derive(Clone, Serialize)]
pub struct TimesheetPreviewSheet {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<TimesheetPreviewRow>,
}

#[derive(Clone, Serialize)]
pub struct TimesheetPreviewRow {
    pub label: String,
    pub values: Vec<f64>,
    pub total: f64,
    pub is_comment: bool,
    pub is_total: bool,
}
