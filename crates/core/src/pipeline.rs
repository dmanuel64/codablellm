use statig::prelude::*;

use crate::extractor;

#[derive(Debug, Default)]
pub struct DatasetGenerator;

#[derive(Debug)]
pub enum Event {}

#[state_machine(
    initial = "State::extract_source_code()",
    before_transition = "Self::before_transition",
    state(derive(Debug)),
    superstate(derive(Debug))
)]
impl DatasetGenerator {
    #[state]
    fn extract_source_code(event: &Event) -> Outcome<State> {
        todo!()
    }

    #[state]
    fn build_dataset(event: &Event) -> Outcome<State> {
        todo!()
    }
}

impl DatasetGenerator {
    fn before_transition(&mut self, source: &State, target: &State, _context: &mut ()) {
        tracing::debug!("Transitioning from `{:?}` to `{:?}`", source, target);
    }
}

#[derive(Default)]
pub struct Options {
    pub extractor: extractor::Options,
}

pub fn run() {
    let mut generator = DatasetGenerator::default().state_machine();
}
