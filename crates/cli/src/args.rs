use clap::Args;
use codablellm::Language;
use strum::IntoEnumIterator;

#[derive(Debug, Args)]
pub struct DatasetOpts {
    /// Languages to only extract
    ///
    /// By default, codablellm will extract all possible languages.
    #[arg(long = "langs", alias = "lang", value_enum, value_delimiter = ',', default_values_t = Language::iter(), hide_default_value = true)]
    languages: Vec<Language>,
}
