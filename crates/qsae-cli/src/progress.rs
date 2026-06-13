use indicatif::ProgressBar;

pub fn create_progress_bar(total: u64) -> ProgressBar {
    ProgressBar::new(total)
}
