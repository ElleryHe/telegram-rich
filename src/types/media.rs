//! Input media objects used by rich message blocks and [`InputRichMessageMedia`](super::InputRichMessageMedia).
//!
//! Each struct serializes its own `type` tag. Captions are accepted for
//! completeness but are ignored by Telegram inside rich messages; use the
//! block's `caption` field instead. `caption_entities` is intentionally not
//! modelled.

use serde::{Deserialize, Serialize};

/// A file to send: a `file_id`, an HTTP URL, or `attach://<name>` for a
/// multipart upload.
pub type InputFileRef = String;

/// An animation file (GIF or H.264/MPEG-4 AVC video without sound).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "animation")]
pub struct InputMediaAnimation {
    /// File to send.
    pub media: InputFileRef,
    /// Optional. Thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<InputFileRef>,
    /// Optional. Width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Optional. Height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Optional. Duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    /// Optional. Cover with a spoiler animation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

/// An audio file to be treated as music.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "audio")]
pub struct InputMediaAudio {
    /// File to send.
    pub media: InputFileRef,
    /// Optional. Thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<InputFileRef>,
    /// Optional. Duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    /// Optional. Performer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performer: Option<String>,
    /// Optional. Title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// A general file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "document")]
pub struct InputMediaDocument {
    /// File to send.
    pub media: InputFileRef,
    /// Optional. Thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<InputFileRef>,
    /// Optional. Disable server-side content type detection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_content_type_detection: Option<bool>,
}

/// A photo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "photo")]
pub struct InputMediaPhoto {
    /// File to send.
    pub media: InputFileRef,
    /// Optional. Cover with a spoiler animation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

/// A video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "video")]
pub struct InputMediaVideo {
    /// File to send.
    pub media: InputFileRef,
    /// Optional. Thumbnail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<InputFileRef>,
    /// Optional. Cover image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<InputFileRef>,
    /// Optional. Start timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_timestamp: Option<u32>,
    /// Optional. Width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Optional. Height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Optional. Duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
    /// Optional. Suitable for streaming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_streaming: Option<bool>,
    /// Optional. Cover with a spoiler animation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_spoiler: Option<bool>,
}

/// A voice message (Bot API 10.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename = "voice_note")]
pub struct InputMediaVoiceNote {
    /// File to send.
    pub media: InputFileRef,
    /// Optional. Duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,
}

/// Any media that can be referenced from a rich message.
///
/// Serializes through the inner struct (which emits its own `type` tag) and
/// deserializes by dispatching on the `type` field.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum InputMedia {
    /// Animation.
    Animation(InputMediaAnimation),
    /// Audio.
    Audio(InputMediaAudio),
    /// Document.
    Document(InputMediaDocument),
    /// Photo.
    Photo(InputMediaPhoto),
    /// Video.
    Video(InputMediaVideo),
    /// Voice note.
    VoiceNote(InputMediaVoiceNote),
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum InputMediaTagged {
    Animation(InputMediaAnimation),
    Audio(InputMediaAudio),
    Document(InputMediaDocument),
    Photo(InputMediaPhoto),
    Video(InputMediaVideo),
    VoiceNote(InputMediaVoiceNote),
}

impl<'de> Deserialize<'de> for InputMedia {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(match InputMediaTagged::deserialize(d)? {
            InputMediaTagged::Animation(m) => InputMedia::Animation(m),
            InputMediaTagged::Audio(m) => InputMedia::Audio(m),
            InputMediaTagged::Document(m) => InputMedia::Document(m),
            InputMediaTagged::Photo(m) => InputMedia::Photo(m),
            InputMediaTagged::Video(m) => InputMedia::Video(m),
            InputMediaTagged::VoiceNote(m) => InputMedia::VoiceNote(m),
        })
    }
}

macro_rules! simple_ctor {
    ($t:ident) => {
        impl $t {
            /// Creates the media from a file reference with all optional fields unset.
            pub fn new(media: impl Into<InputFileRef>) -> Self {
                Self {
                    media: media.into(),
                    ..Default::default()
                }
            }
        }
    };
}

macro_rules! default_impl {
    ($t:ident { $($f:ident),* }) => {
        impl Default for $t {
            fn default() -> Self {
                Self { media: String::new(), $($f: None),* }
            }
        }
        simple_ctor!($t);
    };
}

default_impl!(InputMediaAnimation {
    thumbnail,
    width,
    height,
    duration,
    has_spoiler
});
default_impl!(InputMediaAudio {
    thumbnail,
    duration,
    performer,
    title
});
default_impl!(InputMediaDocument {
    thumbnail,
    disable_content_type_detection
});
default_impl!(InputMediaPhoto { has_spoiler });
default_impl!(InputMediaVideo {
    thumbnail,
    cover,
    start_timestamp,
    width,
    height,
    duration,
    supports_streaming,
    has_spoiler
});
default_impl!(InputMediaVoiceNote { duration });
