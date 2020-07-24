mod logtimedb;

fn main() {
    match logtimedb::open() {
        Ok(_) => {},
        Err(err) =>{ eprintln!("{}", err); }
    }
}
