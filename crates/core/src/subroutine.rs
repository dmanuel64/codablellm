mod c;
mod function;

use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    ops::Range,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

pub use function::{AssociatedFunction, Function, Method};
use tempfile::{NamedTempFile, TempDir, tempfile};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Descriptor {
    pub name: String,
    pub definition: String,
    pub location: Option<Location>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub source: PathBuf,
    pub span: Span,
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.source.display(), self.span.start)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    #[serde(skip)]
    pub bytes: Range<u64>,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[typetag::serde(tag = "type")]
pub trait Subroutine {
    fn descriptor(&self) -> &Descriptor;

    fn name(&self) -> &str {
        &self.descriptor().name
    }

    fn definition(&self) -> &str {
        &self.descriptor().definition
    }
}

impl Display for dyn Subroutine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(location) = &self.descriptor().location {
            write!(f, "{} ({})", self.name(), location)
        } else {
            write!(f, "{}", self.name())
        }
    }
}

pub trait Edit: Subroutine {
    fn set_definition(&mut self, definition: String) -> Result<(), Error>;
}

pub struct Editor<S>
where
    S: Edit,
{
    subroutine: S,
    dirty: bool,
}

impl<S> Editor<S>
where
    S: Edit,
{
    pub fn new(subroutine: S) -> Self {
        Self {
            subroutine,
            dirty: false,
        }
    }

    pub fn edit(&mut self, definition: String) -> Result<(), Error> {
        self.subroutine.set_definition(definition)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        // Not thread-safe
        if self.dirty
            && let Some(location) = &self.subroutine.descriptor().location
        {
            let file = File::open(location.source)?;
            let reader = BufReader::new(file);
            let temp_file = NamedTempFile::new_in(location.source.parent().unwrap())?;
            let mut writer: BufWriter<NamedTempFile> = BufWriter::new(temp_file);

            let mut current_line = 1;
            let mut replaced_span = false;
            for line in reader.lines() {
                let line = line?;

                if current_line < location.span.start.line || current_line > location.span.end.line
                {
                    // Write unchanged lines to the temporary file
                    writeln!(writer, "{line}")?;
                } else if !replaced_span {
                    // Write the new content
                    write!(writer, "{}", self.subroutine.definition())?;
                    replaced_span = true
                }
                current_line += 1
            }

            // Flush remaining buffer data to disk
            writer.flush()?;
            drop(writer);

            // Replace the original file with the temporary one
            temp_file.persist(location.source)?;
        }
        Ok(())
    }
}
