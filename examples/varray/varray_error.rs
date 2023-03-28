#[derive(Debug)]
pub enum Error {
    BincodeError(bincode::Error),
    IoError(std::io::Error),
}

impl From<bincode::Error> for Error {
    fn from(error: bincode::Error) -> Self {
        Error::BincodeError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}
