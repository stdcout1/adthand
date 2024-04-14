mod prayer;
use bitcode::{Decode, Encode};

#[derive(Decode, Encode, Debug)]
pub enum Request {
    Kill,
    Ping
}

#[derive(Decode, Encode, Debug)]
pub enum Answer {
    Ping,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1, std::mem::size_of::<Request>())
    }
}
