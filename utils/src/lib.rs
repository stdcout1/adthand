pub mod prayer;
use bitcode::{Decode, Encode};

#[derive(Decode, Encode, Debug)]
pub enum Request {
    Kill,
    Ping,
    Next,
    Waybar,
    All,
}

#[derive(Decode, Encode, Debug)]
pub enum Answer<'a> {
    Ping,
    Next(&'a str, &'a str, &'a str),
    Waybar(&'a str, &'a str, &'a str, Vec<(&'a str, String)>),
    All(Vec<(&'a str, String)>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, std::mem::size_of::<Request>())
    }
}
