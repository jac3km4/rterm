use rterm::buffer::DefaultHandler;
use rterm::Configuration;

fn main() {
    rterm::run(Configuration::default(), DefaultHandler).unwrap();
}
