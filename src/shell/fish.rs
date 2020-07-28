use std::io::Write;

pub struct Fish<Output> {
    output: Output,
}

impl<Output: Write> super::Shell for Fish<Output> {
    fn cd<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(),std::io::Error> {
        write!(self.output, "cd '{}'\n", path.as_ref().display())
    }
    fn checkout<B: AsRef<str>>(&mut self, branch: B) -> Result<(),std::io::Error> {
        write!(self.output, "git checkout {}\n", branch.as_ref())
    }
    fn new_branch<B: AsRef<str>>(&mut self, branch: B) -> Result<(),std::io::Error> {
        write!(self.output, "git checkout -b {}\n", branch.as_ref())
    }
}

impl<Output: Write> Fish<Output> {
    fn new(output: Output) -> Self {
        Fish { output: output }
    }
}
