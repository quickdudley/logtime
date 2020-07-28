use std::io::Write;

pub struct Fish<Output> {
    output: Output,
}

impl<Output: Write> super::Shell for Fish<Output> {
    fn cd(&mut self, path: &std::path::Path) -> Result<(),std::io::Error> {
        write!(self.output, "cd '{}'\n", escape_string(path.display().to_string().as_str()))
    }
    fn checkout(&mut self, branch: &str) -> Result<(),std::io::Error> {
        write!(self.output, "git checkout {}\n", escape_string(branch))
    }
    fn new_branch(&mut self, branch: &str, source: Option<&str>) -> Result<(),std::io::Error> {
        match source {
            None => write!(self.output, "git checkout -b {}\n", escape_string(branch)),
            Some(source) => write!(self.output, "git checkout -b {} --no-track {}\n", escape_string(branch), escape_string(source)),
        }
    }
}

impl<Output: Write> Fish<Output> {
    pub fn new(output: Output) -> Self {
        Fish { output: output }
    }
}

const ENCODE_DIRECT_BASE: char = '\u{F600}';
const ENCODE_DIRECT_CAP: char = '\u{F700}';

fn escape_string(raw: &str) -> String {
    use std::convert::TryFrom;
    let mut need_escape = false;
    let mut need_complex_escape = false;
    if raw.len() == 0 {
        return String::from("\'\'");
    }
    let mut out = String::new();
    for i in raw.chars() {
        if i >= ENCODE_DIRECT_BASE && i < ENCODE_DIRECT_CAP {
            let val = i as u32 - ENCODE_DIRECT_BASE as u32;
            out.push_str(format!("\\X{:02x}", val).as_str());
            need_escape = true;
            need_complex_escape = true;
        } else {
            match i {
                '\0' => {
                    out.push_str("\\0");
                    need_escape = true;
                    need_complex_escape = true;
                },
                '\t' => {
                    out.push_str("\\t");
                    need_escape = true;
                    need_complex_escape = true;
                },
                '\n' => {
                    out.push_str("\\n");
                    need_escape = true;
                    need_complex_escape = true;
                },
                '\u{0008}' => {
                    out.push_str("\\b");
                    need_escape = true;
                    need_complex_escape = true;
                },
                '\r' => {
                    out.push_str("\\r");
                    need_escape = true;
                    need_complex_escape = true;
                },
                '\u{001B}' => {
                    out.push_str("\\e");
                    need_escape = true;
                    need_complex_escape = true;
                },
                '\\' => {
                    out.push_str("\\\\");
                    need_escape = true;
                    need_complex_escape = true;
                },
                '\'' => {
                    out.push_str("\\\'");
                    need_escape = true;
                    need_complex_escape = true;
                },
                '&' | '$' | ' ' | '#' | '^'
                    | '<' | '>' | '(' | ')'
                    | '[' | ']' | '{' | '}'
                    | '?' | '*' | '|' | ';'
                    | '"' | '%' | '~' => {
                        out.push_str(format!("\\{}", i).as_str());
                        need_escape = true;
                },
                _ => {
                    if i < '\u{0020}' {
                        if i < '\u{001b}' && i > '\u{0000}' {
                            out.push_str(format!("\\c{}", char::from(96 + (i as u8))).as_str());
                        } else {
                            out.push_str(format!("\\x{:02x}", i as u8).as_str())
                        }
                        need_escape = true;
                        need_complex_escape = true;
                    } else {
                        out.push(i);
                    }
                }
            }
        }
    }
    if need_escape && !need_complex_escape {
        format!("\'{}\'", raw)
    } else {
        out
    }
}
