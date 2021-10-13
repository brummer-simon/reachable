use std::error::Error;
use std::fmt::{self};
use std::io::{self};
use std::num::{self};

type ErrorMessage = &'static str;

// ParseTargetError
#[derive(Debug)]
pub enum ParseTargetError {
    /// ParseTargetError containing a Message
    Message(ErrorMessage),
    /// ParseTargetError containing a Message and a ParseIntError
    ParseIntError(ErrorMessage, num::ParseIntError),
    /// ParseTargetError containing a Message and a trait object implementing Error
    GenericError(ErrorMessage, Box<dyn Error>),
}

impl Error for ParseTargetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseTargetError::Message(_) => None,
            ParseTargetError::ParseIntError(_, ref error) => Some(error),
            ParseTargetError::GenericError(_, ref error) => Some(error.as_ref()),
        }
    }
}

impl fmt::Display for ParseTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let error_message = match self {
            ParseTargetError::Message(error_message)
            | ParseTargetError::ParseIntError(error_message, _)
            | ParseTargetError::GenericError(error_message, _) => error_message,
        };

        match self.source() {
            None => write!(formatter, "{}", error_message),
            Some(error) => write!(formatter, "{} caused by: {}", error_message, error),
        }
    }
}

impl From<ErrorMessage> for ParseTargetError {
    fn from(message: ErrorMessage) -> Self {
        ParseTargetError::Message(message)
    }
}

impl From<(ErrorMessage, num::ParseIntError)> for ParseTargetError {
    fn from(pieces: (ErrorMessage, num::ParseIntError)) -> Self {
        let (msg, error) = pieces;
        ParseTargetError::ParseIntError(msg, error)
    }
}

impl From<(ErrorMessage, Box<dyn Error>)> for ParseTargetError {
    fn from(pieces: (ErrorMessage, Box<dyn Error>)) -> Self {
        let (msg, error) = pieces;
        ParseTargetError::GenericError(msg, error)
    }
}

impl From<Box<dyn Error>> for ParseTargetError {
    fn from(error: Box<dyn Error>) -> Self {
        ParseTargetError::from(("GenericError", error))
    }
}

// ResolveTargetError
#[derive(Debug)]
pub enum ResolveTargetError {
    /// ResolveTargetError containing a Message
    Message(ErrorMessage),
    /// ResolveTargetError containing a Message and an io::Error
    IoError(ErrorMessage, io::Error),
    /// CheckTargetError containing a Message and a trait object implementing Error
    GenericError(ErrorMessage, Box<dyn Error>),
}

impl Error for ResolveTargetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ResolveTargetError::Message(_) => None,
            ResolveTargetError::IoError(_, ref error) => Some(error),
            ResolveTargetError::GenericError(_, ref error) => Some(error.as_ref()),
        }
    }
}

impl fmt::Display for ResolveTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let error_message = match self {
            ResolveTargetError::Message(error_message)
            | ResolveTargetError::IoError(error_message, _)
            | ResolveTargetError::GenericError(error_message, _) => error_message,
        };

        match self.source() {
            None => write!(formatter, "{}", error_message),
            Some(error) => write!(formatter, "{} caused by: {}", error_message, error),
        }
    }
}

impl From<ErrorMessage> for ResolveTargetError {
    fn from(message: ErrorMessage) -> Self {
        ResolveTargetError::Message(message)
    }
}

impl From<(ErrorMessage, io::Error)> for ResolveTargetError {
    fn from(pieces: (ErrorMessage, io::Error)) -> Self {
        let (msg, error) = pieces;
        ResolveTargetError::IoError(msg, error)
    }
}

impl From<io::Error> for ResolveTargetError {
    fn from(error: io::Error) -> Self {
        ResolveTargetError::from(("IoError", error))
    }
}

impl From<(ErrorMessage, Box<dyn Error>)> for ResolveTargetError {
    fn from(pieces: (ErrorMessage, Box<dyn Error>)) -> Self {
        let (msg, error) = pieces;
        ResolveTargetError::GenericError(msg, error)
    }
}

impl From<Box<dyn Error>> for ResolveTargetError {
    fn from(error: Box<dyn Error>) -> Self {
        ResolveTargetError::from(("GenericError", error))
    }
}

// TargetCheckError
#[derive(Debug)]
pub enum CheckTargetError {
    /// CheckTargetError containing a Message
    Message(ErrorMessage),
    /// CheckTargetError containing a Message and a ResolveTargetError
    ResolveTargetError(ErrorMessage, ResolveTargetError),
    /// CheckTargetError containing a Message and a trait object implementing Error
    GenericError(ErrorMessage, Box<dyn Error>),
}

impl Error for CheckTargetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CheckTargetError::Message(_) => None,
            CheckTargetError::ResolveTargetError(_, ref error) => Some(error),
            CheckTargetError::GenericError(_, ref error) => Some(error.as_ref()),
        }
    }
}

impl fmt::Display for CheckTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let error_message = match self {
            CheckTargetError::Message(error_message)
            | CheckTargetError::ResolveTargetError(error_message, _)
            | CheckTargetError::GenericError(error_message, _) => error_message,
        };

        match self.source() {
            None => write!(formatter, "{}", error_message),
            Some(error) => write!(formatter, "{} caused by: {}", error_message, error),
        }
    }
}

impl From<ErrorMessage> for CheckTargetError {
    fn from(message: ErrorMessage) -> Self {
        CheckTargetError::Message(message)
    }
}

impl From<(ErrorMessage, ResolveTargetError)> for CheckTargetError {
    fn from(pieces: (ErrorMessage, ResolveTargetError)) -> Self {
        let (msg, error) = pieces;
        CheckTargetError::ResolveTargetError(msg, error)
    }
}

impl From<ResolveTargetError> for CheckTargetError {
    fn from(error: ResolveTargetError) -> Self {
        CheckTargetError::from(("ResolveTargetError", error))
    }
}

impl From<(ErrorMessage, Box<dyn Error>)> for CheckTargetError {
    fn from(pieces: (ErrorMessage, Box<dyn Error>)) -> Self {
        let (msg, error) = pieces;
        CheckTargetError::GenericError(msg, error)
    }
}

impl From<Box<dyn Error>> for CheckTargetError {
    fn from(error: Box<dyn Error>) -> Self {
        CheckTargetError::from(("GenericError", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ParseTargetError tests
    #[test]
    fn parse_target_error_from_str() {
        // Expectency: A ParseTargetError must contain its error message.
        assert_eq!(
            format!("{}", ParseTargetError::from("Error Message!")),
            "Error Message!"
        );
    }

    #[test]
    fn parse_target_error_from_parse_int_error() {
        // Expectency: A ParseTargetError must contain its error message and the description
        //             of the inner ParseIntError.
        let error = i32::from_str_radix("invalid", 10).unwrap_err();
        assert_eq!(
            format!("{}", ParseTargetError::from(("ParseIntError!", error))),
            "ParseIntError! caused by: invalid digit found in string"
        );
    }

    #[test]
    fn parse_target_error_from_boxed_error_trait_object() {
        // Expectency: A ParseTargetError must contain its error message and the description
        //             of the inner boxed error trait object.
        let boxed_error: Box<dyn Error> = Box::new(io::Error::from(io::ErrorKind::AddrNotAvailable));
        assert_eq!(
            format!("{}", ParseTargetError::from(boxed_error)),
            "GenericError caused by: address not available"
        );
    }

    #[test]
    fn parse_target_error_chain_multiple_errors() {
        // Expectency: A ParseTargetError must recursively resolve its all its stored inner errors.
        //             chaining them together into a single message
        let error1: Box<dyn Error> = Box::new(ParseTargetError::from("Layer1!"));
        let error2: Box<dyn Error> = Box::new(ParseTargetError::from(("Layer2!", error1)));
        assert_eq!(
            format!("{}", ParseTargetError::from(("Layer3!", error2))),
            "Layer3! caused by: Layer2! caused by: Layer1!"
        );
    }

    // ResolveTargetError tests
    #[test]
    fn resolve_target_error_from_str() {
        // Expectency: A ResolveTargetError must contain its error message
        assert_eq!(
            format!("{}", ResolveTargetError::from("Error Message!")),
            "Error Message!"
        );
    }

    #[test]
    fn resolve_target_error_from_parse_int_error() {
        // Expectency: A ResolveTargetError must contain its error message and the description
        //             of the inner io::Error.
        assert_eq!(
            format!("{}", ResolveTargetError::from(io::Error::from(io::ErrorKind::Other))),
            "IoError caused by: other error"
        );
    }

    #[test]
    fn resolve_target_error_from_boxed_error_trait_object() {
        // Expectency: A ResolveTargetError must contain its error message and the description
        //             of the inner boxed error trait object.
        let boxed_error: Box<dyn Error> = Box::new(ParseTargetError::from("ParseTargetError"));
        assert_eq!(
            format!("{}", ResolveTargetError::from(boxed_error)),
            "GenericError caused by: ParseTargetError"
        );
    }

    // CheckTargetError tests
    #[test]
    fn check_target_error_from_str() {
        // Expectency: A CheckTargetError must contain its error message.
        assert_eq!(
            format!("{}", CheckTargetError::from("Error Message!")),
            "Error Message!"
        );
    }

    #[test]
    fn check_target_error_from_resolve_target_error() {
        // Expectency: A CheckTargetError must contain its error message and an instance of
        //             ResolveTargetError
        let resolve_target_error = ResolveTargetError::from(io::Error::from(io::ErrorKind::AddrNotAvailable));
        assert_eq!(
            format!("{}", CheckTargetError::from(resolve_target_error)),
            "ResolveTargetError caused by: IoError caused by: address not available"
        );
    }

    #[test]
    fn check_target_error_from_boxed_error_trait_object() {
        // Expectency: A CheckTargetError must contain its error message and the description
        //             of the inner boxed error trait object.
        let boxed_error: Box<dyn Error> = Box::new(io::Error::from(io::ErrorKind::AddrNotAvailable));
        assert_eq!(
            format!("{}", CheckTargetError::from(boxed_error)),
            "GenericError caused by: address not available"
        );
    }

    #[test]
    fn check_target_error_via_questionmark_operator() {
        // Expectency: Ensure conversion via Questionmark operator: Construct ResolveTargetError
        //             from io::Error and then construct CheckTargetError from ResolveTargetError
        fn returns_io_error() -> Result<u32, io::Error> {
            Err(io::Error::from(io::ErrorKind::TimedOut))
        }
        fn returns_resolve_target_error() -> Result<u32, ResolveTargetError> {
            Ok(returns_io_error()?)
        }
        fn returns_check_target_error() -> Result<u32, CheckTargetError> {
            Ok(returns_resolve_target_error()?)
        }
        assert_eq!(
            format!("{}", returns_check_target_error().unwrap_err()),
            "ResolveTargetError caused by: IoError caused by: timed out"
        );
    }
}
