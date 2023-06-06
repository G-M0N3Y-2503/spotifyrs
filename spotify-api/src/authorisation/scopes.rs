use strum_macros::*;

/// [Spotify authorisation scopes](https://developer.spotify.com/documentation/general/guides/authorization/scopes/)
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRefStr, EnumString, Display,
)]
#[strum(serialize_all = "kebab-case")]
pub enum Scopes {
    /// [app-remote-control](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#app-remote-control)
    AppRemoteControl,
    /// [playlist-modify-private](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#playlist-modify-private)
    PlaylistModifyPrivate,
    /// [playlist-modify-public](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#playlist-modify-public)
    PlaylistModifyPublic,
    /// [playlist-read-collaborative](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#playlist-read-collaborative)
    PlaylistReadCollaborative,
    /// [playlist-read-private](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#playlist-read-private)
    PlaylistReadPrivate,
    /// [streaming](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#streaming)
    Streaming,
    /// [ugc-image-upload](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#ugc-image-upload)
    UgcImageUpload,
    /// [user-follow-modify](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-follow-modify)
    UserFollowModify,
    /// [user-follow-read](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-follow-read)
    UserFollowRead,
    /// [user-library-modify](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-library-modify)
    UserLibraryModify,
    /// [user-library-read](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-library-read)
    UserLibraryRead,
    /// [user-modify-playback-state](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-modify-playback-state)
    UserModifyPlaybackState,
    /// [user-read-currently-playing](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-read-currently-playing)
    UserReadCurrentlyPlaying,
    /// [user-read-email](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-read-email)
    UserReadEmail,
    /// [user-read-playback-position](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-read-playback-position)
    UserReadPlaybackPosition,
    /// [user-read-playback-state](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-read-playback-state)
    UserReadPlaybackState,
    /// [user-read-private](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-read-private)
    UserReadPrivate,
    /// [user-read-recently-played](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-read-recently-played)
    UserReadRecentlyPlayed,
    /// [user-top-read](https://developer.spotify.com/documentation/general/guides/authorization/scopes/#user-top-read)
    UserTopRead,
}

impl FromIterator<Scopes> for String {
    fn from_iter<T: IntoIterator<Item = Scopes>>(iter: T) -> Self {
        iter.into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl<'a> FromIterator<&'a Scopes> for String {
    fn from_iter<T: IntoIterator<Item = &'a Scopes>>(iter: T) -> Self {
        iter.into_iter()
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

pub(super) mod serialize_scopes {
    use super::*;

    pub fn serialize<S: serde::Serializer>(
        value: &Vec<Scopes>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&String::from_iter(value))
    }

    pub struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Scopes>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A string containing 0 or more Scopes")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            use std::str::FromStr;
            if !value.is_empty() {
                value
                    .split(' ')
                    .map(Scopes::from_str)
                    .collect::<Result<Vec<Scopes>, <Scopes as FromStr>::Err>>()
                    .map_err(|_| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Str(value), &self)
                    })
            } else {
                Ok(Vec::new())
            }
        }
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<Scopes>, D::Error> {
        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    const EMPTY_SLICE: [&Scopes; 0] = [];

    #[wasm_bindgen_test]
    fn test_as_string() {
        assert_eq!(Scopes::AppRemoteControl.as_ref(), "app-remote-control");
        assert_eq!(Scopes::AppRemoteControl.to_string(), "app-remote-control");
    }

    #[wasm_bindgen_test]
    fn test_from_str() {
        assert_eq!(
            Scopes::from_str("app-remote-control"),
            Ok(Scopes::AppRemoteControl)
        );
    }

    #[wasm_bindgen_test]
    fn test_scope_ref_as_string() {
        assert_eq!(String::from_iter(EMPTY_SLICE), "");
        assert_eq!(
            String::from_iter([&Scopes::AppRemoteControl]),
            "app-remote-control"
        );
        assert_eq!(
            String::from_iter([&Scopes::AppRemoteControl, &Scopes::PlaylistModifyPrivate]),
            "app-remote-control playlist-modify-private"
        );
        assert_eq!(
            String::from_iter([
                &Scopes::AppRemoteControl,
                &Scopes::PlaylistModifyPrivate,
                &Scopes::PlaylistModifyPublic
            ]),
            "app-remote-control playlist-modify-private playlist-modify-public"
        );
    }

    #[wasm_bindgen_test]
    fn test_scope_as_string() {
        assert_eq!(String::from_iter(EMPTY_SLICE), "");
        assert_eq!(
            String::from_iter([Scopes::AppRemoteControl]),
            "app-remote-control"
        );
        assert_eq!(
            String::from_iter([Scopes::AppRemoteControl, Scopes::PlaylistModifyPrivate]),
            "app-remote-control playlist-modify-private"
        );
        assert_eq!(
            String::from_iter([
                Scopes::AppRemoteControl,
                Scopes::PlaylistModifyPrivate,
                Scopes::PlaylistModifyPublic
            ]),
            "app-remote-control playlist-modify-private playlist-modify-public"
        );
    }
}
