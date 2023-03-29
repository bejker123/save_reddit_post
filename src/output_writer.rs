//TODO: fix json formatting

use std::io::Write;

pub struct OutputWriter {
    output: Box<dyn Write>,
    pub content: String,
}

impl OutputWriter {
    pub fn new() -> Self {
        Self {
            output: Box::new(std::io::stdout()),
            content: String::new(),
        }
    }
    pub fn set_output(mut self, output: Box<dyn Write>) -> Self {
        self.output = output;
        self
    }
    pub fn write(mut self) -> std::io::Result<()> {
        write!(self.output, "{}", self.content)
    }
}
