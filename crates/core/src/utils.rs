use std::sync::Arc;

use indicatif::{MultiProgress, ProgressBar};

#[derive(Debug, Default, Clone)]
pub enum ProgressDisplay {
    Hidden,
    #[default]
    Standalone,
    Nested(Arc<MultiProgress>),
}

impl ProgressDisplay {
    fn new_progress_bar_inner(&self, len: Option<u64>, spinner: bool) -> ProgressBar {
        let progress = match self {
            ProgressDisplay::Hidden => ProgressBar::hidden(),
            ProgressDisplay::Standalone => {
                if let Some(l) = len {
                    ProgressBar::new(l)
                } else if spinner {
                    ProgressBar::new_spinner()
                } else {
                    ProgressBar::no_length()
                }
            }
            ProgressDisplay::Nested(multi_progress) => {
                let pb = ProgressDisplay::Standalone.new_progress_bar_inner(len, spinner);
                multi_progress.add(pb)
            }
        };
        progress
    }

    pub fn new_progress_bar(&self, len: Option<u64>) -> ProgressBar {
        self.new_progress_bar_inner(len, false)
    }

    pub fn new_spinner(&self) -> ProgressBar {
        self.new_progress_bar_inner(None, true)
    }
}
