//! Helper URL functions for extending a single base URL for API endpoint variations

/// Url continence wrapper for extending URLs
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Url(url::Url);

impl TryFrom<url::Url> for Url {
    type Error = NotABaseError;
    fn try_from(url: url::Url) -> Result<Self, Self::Error> {
        Url::new(url)
    }
}

impl std::ops::Deref for Url {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Url {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error for when a URL is not a base URL, meaning that parsing a relative URL string
/// with this URL as the base will return an error.
///
/// This is the case if the scheme and `:` delimiter are not followed by a `/` slash,
/// as is typically the case of `data:` and `mailto:` URLs.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct NotABaseError;
impl std::fmt::Display for NotABaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "URL is not a base URL")
    }
}
impl std::error::Error for NotABaseError {}

impl Url {
    /// Creates a new URL from the current browser location.
    /// # Panics
    /// If the current browser location is invalid.
    pub fn from_browser_location() -> Result<Url, NotABaseError> {
        Url::new(
            url::Url::parse(&crate::browser_window().location().origin().unwrap())
                .expect("A valid browser location"),
        )
    }

    /// Errors if the given URL cannot be a base URL
    pub fn new(url: url::Url) -> Result<Url, NotABaseError> {
        if !url.cannot_be_a_base() {
            Ok(Url(url))
        } else {
            Err(NotABaseError)
        }
    }

    /// Creates a new URL with the given path segments appended
    pub fn with_path<I>(&self, path_segments: I) -> Url
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut new_url = (*self).clone();
        new_url
            .path_segments_mut()
            .expect("A base URL")
            .extend(path_segments);
        new_url
    }
}

impl serde::Serialize for Url {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Url {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Url;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A string containing a valid URL")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                url::Url::parse(value)
                    .map_err(|_| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Other(value), &self)
                    })?
                    .try_into()
                    .map_err(|_| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Other(value),
                            &"A URL that is a base URL",
                        )
                    })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_url() {
        let url = ::url::Url::parse(&browser_window().location().origin().unwrap())
            .expect("A valid browser location");
        let url = super::Url::new(url).expect("A base URL");
        assert_eq!(
            url.with_path([""]).to_string(),
            browser_window().location().origin().unwrap() + "/"
        );
        assert_eq!(
            url.with_path(["path"]).to_string(),
            browser_window().location().origin().unwrap() + "/path"
        );
        assert_eq!(
            url.with_path(["path", "path2"]).to_string(),
            browser_window().location().origin().unwrap() + "/path/path2"
        );
        assert_eq!(
            url.with_path(["path", "path2"]),
            url.with_path(["path"]).with_path(["path2"])
        );
    }

    #[wasm_bindgen_test]
    fn test_url_serde() -> serde_json::Result<()> {
        let url: super::Url = ::url::Url::parse(&browser_window().location().origin().unwrap())
            .expect("A valid browser location")
            .try_into()
            .expect("A base URL");
        let serialized = serde_json::to_string(&url)?;
        let deserialized: super::Url = serde_json::from_str(&serialized)?;
        assert_eq!(url, deserialized);

        Ok(())
    }
}
