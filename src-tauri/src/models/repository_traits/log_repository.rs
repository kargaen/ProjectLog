pub trait LogRepository {
    /// Append a new timestamped entry: datetime tab project [tab comment].
    /// The single timestamp is both the stop time of the previous entry
    /// and the start time of this one.
    fn append_entry(&self, project: &str, comment: &str);

    /// Append a comment to the last line of the log (which has no comment yet).
    fn append_comment_to_last(&self, comment: &str);

    /// Empty the log file completely.
    fn reset(&self);
}
