mod fish;

pub trait Shell {
    fn cd<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<(),std::io::Error>;
    fn checkout<B: AsRef<str>>(&mut self, branch: B) -> Result<(),std::io::Error>;
    fn new_branch<B: AsRef<str>>(&mut self, branch: B) -> Result<(),std::io::Error>;
}

