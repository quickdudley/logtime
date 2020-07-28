pub mod fish;

pub trait Shell {
    fn cd(&mut self, path: &std::path::Path) -> Result<(),std::io::Error>;
    fn checkout(&mut self, branch: &str) -> Result<(),std::io::Error>;
    fn new_branch(&mut self, branch: &str, source: Option<&str>) -> Result<(),std::io::Error>;
}

impl<S: Shell> Shell for Vec<S> {
    fn cd(&mut self, path: &std::path::Path) -> Result<(),std::io::Error> {
        let mut i = 0;
        while i != self.len() {
            match self[i].cd(path) {
                Ok(_) => { i += 1; },
                Err(_) => { self.remove(i); },
            }
        }
        Ok(())
    }
    fn checkout(&mut self, branch: &str) -> Result<(),std::io::Error> {
        let mut i = 0;
        while i != self.len() {
            match self[i].checkout(branch) {
                Ok(_) => { i += 1; },
                Err(_) => { self.remove(i); },
            }
        }
        Ok(())
    }
    fn new_branch(&mut self, branch: &str, source: Option<&str>) -> Result<(),std::io::Error> {
        let mut i = 0;
        while i != self.len() {
            match self[i].checkout(branch) {
                Ok(_) => { i += 1; },
                Err(_) => { self.remove(i); },
            }
        }
        Ok(())
    }
}
