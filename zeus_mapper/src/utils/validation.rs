use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

pub type ValidationResult = Result<(), ValidationError>;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// `path` is a dotted/indexed location relative to the struct `validate()` was called on (e.g.
    /// `real_episode_data[3].constant_1_0x00`), built up by [`ValidationError::add_parent`] as the error
    /// bubbles from a nested `validate()` call back out to the caller.
    path: String,
    message: String,
}

impl ValidationError {
    pub fn expected_exactly<T: Debug>(path: impl Into<String>, actual: T, expected: T) -> Self {
        return ValidationError::new(path, format!("encountered {actual:?}, expected {expected:?}"));
    }

    pub fn expected_range<T: Debug>(path: impl Into<String>, actual: T, min: T, max: T) -> Self {
        return ValidationError::new(path, format!("encountered {actual:?}, expected range [{min:?}, {max:?}]"));
    }

    pub fn expected_one_of<T: Debug>(path: impl Into<String>, actual: T, expected: &[T]) -> Self {
        return ValidationError::new(path, format!("encountered {actual:?}, expected one of {expected:?}"));
    }

    pub fn add_parent(parent: impl Display + 'static) -> impl Fn(ValidationError) -> ValidationError {
        return move |e| ValidationError::new(format!("{parent}.{}", e.path), e.message);
    }

    // Non public - instanced are created with one of the above `expected_...` methods for consistent error messages
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        return ValidationError {
            path: path.into(),
            message: message.into(),
        };
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}: {}", self.path, self.message);
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_expected_constant<T: Debug + Copy + PartialEq>(path: &str, values: &[T], expected: T) -> ValidationResult {
    if let Some((offset, actual)) = values.iter().enumerate().find(|(_, value)| **value != expected) {
        return Err(ValidationError::expected_exactly(format!("{path}[{offset}]"), *actual, expected));
    }

    return Ok(());
}
