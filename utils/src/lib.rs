pub mod prayer;
use bitcode::{Decode, Encode};
use prayer::Prayers;

#[derive(Decode, Encode, Debug)]
pub enum Request {
    Kill,
    Ping,
    Next,
    All
}

#[derive(Decode, Encode, Debug)]
pub enum Answer<'a> {
    Ping,
    Next(&'a str),
    All(Vec<&'a str>)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, std::mem::size_of::<Request>())
    }
}
