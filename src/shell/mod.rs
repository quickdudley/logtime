pub mod fish;
use std::ops::IndexMut;

pub trait Shell {
    fn cd(&mut self, path: &std::path::Path) -> Result<(),std::io::Error>;
    fn checkout(&mut self, branch: &str) -> Result<(),std::io::Error>;
    fn new_branch(&mut self, branch: &str, source: Option<&str>) -> Result<(),std::io::Error>;
}

impl<S: Shell> Shell for Vec<S> {
    fn cd(&mut self, path: &std::path::Path) -> Result<(),std::io::Error> {
        let mut i = 0;
        while i != self.len() {
            match self.index_mut(i).cd(path) {
                Ok(_) => { i += 1; },
                Err(_) => { self.remove(i); },
            }
        }
        Ok(())
    }
    fn checkout(&mut self, branch: &str) -> Result<(),std::io::Error> {
        let mut i = 0;
        while i != self.len() {
            match self.index_mut(i).checkout(branch) {
                Ok(_) => { i += 1; },
                Err(_) => { self.remove(i); },
            }
        }
        Ok(())
    }
    fn new_branch(&mut self, branch: &str, source: Option<&str>) -> Result<(),std::io::Error> {
        let mut i = 0;
        while i != self.len() {
            match self.index_mut(i).new_branch(branch, source) {
                Ok(_) => { i += 1; },
                Err(_) => { self.remove(i); },
            }
        }
        Ok(())
    }
}

impl Shell for Box<dyn Shell> {
    fn cd(&mut self, path: &std::path::Path) -> Result<(),std::io::Error> {
        self.as_mut().cd(path)
    }
    fn checkout(&mut self, branch: &str) -> Result<(),std::io::Error> {
        self.as_mut().checkout(branch)
    }
    fn new_branch(&mut self, branch: &str, source: Option<&str>) -> Result<(),std::io::Error> {
        self.as_mut().new_branch(branch, source)
    }
}
