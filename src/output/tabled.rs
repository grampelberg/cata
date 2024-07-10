//! A collection of utilities that provide Display implementations for tables
//! outputs.
use std::fmt::{self, Display};

use serde::{de::Deserializer, Deserialize, Serialize};

/// A wrapper around `Option<T>` that implements `Display`.
///
/// The Tabled trait requires that all fields implement Display. This is a
/// bummer because Option doesn't by default. Note that deserialize is
/// implemented here to allow for serde's missing fields to work correctly with
/// the named type.
#[derive(Debug, Clone, Serialize)]
pub struct Option<T>(std::option::Option<T>);

impl<T> Display for Option<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{value}"),
            None => write!(f, ""),
        }
    }
}

impl<'de, T> Deserialize<'de> for Option<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<T> = Option::<T>::from(std::option::Option::deserialize(deserializer)?);

        Ok(opt)
    }
}

impl<T> From<std::option::Option<T>> for Option<T> {
    fn from(option: std::option::Option<T>) -> Self {
        match option {
            Some(value) => Self(Some(value)),
            None => Self(None),
        }
    }
}

/// Format a list of items for display.
///
/// Slices do not have Display implemented by default. This function will take
/// the Display for each item in the slice, sort them and then concatenate with
/// newlines into a single string. This works well with tabled output.
///
/// ```
/// use cata::output::tabled::display;
///
/// #[derive(serde::Serialize, tabled::Tabled)]
/// struct MyItem {
///   #[tabled(display_with = "display")]
///   field: Vec<String>,
/// }
/// ```
// TODO(thomas): This feels like it should be a newtype for Vec<T>
pub fn display<T>(value: &[T]) -> String
where
    T: Display,
{
    let display = &mut value
        .iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<String>>();

    display.sort();

    display.join("\n")
}
