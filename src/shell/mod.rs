pub mod fish;
pub mod zsh;

use std::ops::IndexMut;

pub trait Shell {
    fn cd(&mut self, path: &std::path::Path) -> Result<(),std::io::Error>;
    fn cmd(&mut self, exe: &str, args: &Vec<String>) -> Result<(),std::io::Error>;
    fn env(&mut self, key: &str, val: &str) -> Result<(),std::io::Error>;
    fn checkout(&mut self, branch: &str) -> Result<(),std::io::Error> {
        self.cmd("git", &vec![String::from("checkout"), String::from(branch)])
    }
    fn new_branch(&mut self, branch: &str, source: Option<&str>) -> Result<(),std::io::Error> {
        let mut args = vec![String::from("checkout"), String::from("-b"), String::from(branch)];
        match source {
            None => (),
            Some(source) => {
                args.push(String::from("--no-track"));
                args.push(String::from(source));
            },
        }
        self.cmd("git", &args)
    }
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
   fn cmd(&mut self, exe: &str, args: &Vec<String>) -> Result<(),std::io::Error> {
       let mut i = 0;
       while i != self.len() {
           match self.index_mut(i).cmd(exe, args) {
               Ok(_) => { i += 1 },
               Err(_) => { self.remove(i); },
           }
       }
       Ok(())
   }
   fn env(&mut self, key: &str, val: &str) -> Result<(),std::io::Error> {
       let mut i = 0;
       while i != self.len() {
           match self.index_mut(i).env(key, val) {
               Ok(_) => { i += 1 },
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
    fn cmd(&mut self, exe: &str, args: &Vec<String>) -> Result<(),std::io::Error> {
        self.as_mut().cmd(exe, args)
    }
    fn env(&mut self, key: &str, val: &str) -> Result<(),std::io::Error> {
        self.as_mut().env(key, val)
    }
}
