use std::io::Write;

pub struct Fish<Output> {
    output: Output,
}

impl<Output: Write> super::Shell for Fish<Output> {
    fn cd(&mut self, path: &std::path::Path) -> Result<(),std::io::Error> {
        write!(self.output, "cd '{}'\n", path.display())
    }
    fn checkout(&mut self, branch: &str) -> Result<(),std::io::Error> {
        write!(self.output, "git checkout {}\n", branch)
    }
    fn new_branch(&mut self, branch: &str, source: Option<&str>) -> Result<(),std::io::Error> {
        match source {
            None => write!(self.output, "git checkout -b {}\n", branch),
            Some(source) => write!(self.output, "git checkout -b {} --no-track {}\n", branch, source),
        }
    }
}

impl<Output: Write> Fish<Output> {
    pub fn new(output: Output) -> Self {
        Fish { output: output }
    }
}
