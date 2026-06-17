#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimesheetRange {
    Today,
    Week,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimesheetFormat {
    Full,
    Recent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimesheetOptions {
    pub range: TimesheetRange,
    pub format: TimesheetFormat,
}

impl TimesheetOptions {
    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn full(range: TimesheetRange) -> Self {
        Self {
            range,
            format: TimesheetFormat::Full,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub const fn recent() -> Self {
        Self {
            range: TimesheetRange::Today,
            format: TimesheetFormat::Recent,
        }
    }
}
