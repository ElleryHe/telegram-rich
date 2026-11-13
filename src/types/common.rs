//! Small shared objects referenced by rich message types.

use serde::{Deserialize, Serialize};

/// A Telegram user (subset of fields relevant to rich text mentions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for this user or bot.
    pub id: i64,
    /// True, if this user is a bot.
    pub is_bot: bool,
    /// User's or bot's first name.
    pub first_name: String,
    /// Optional. User's or bot's last name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// Optional. User's or bot's username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Optional. IETF language tag of the user's language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

/// A point on the map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Latitude.
    pub latitude: f64,
    /// Longitude.
    pub longitude: f64,
    /// Optional. The radius of uncertainty for the location, in meters; 0-1500.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal_accuracy: Option<f64>,
}

/// Describes a Web App.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebAppInfo {
    /// An HTTPS URL of a Web App.
    pub url: String,
}

/// A parameter of a button used to automatically authorize a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginUrl {
    /// An HTTPS URL to be opened with user authorization data added to the query string.
    pub url: String,
    /// Optional. New text of the button in forwarded messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_text: Option<String>,
    /// Optional. Username of a bot used for user authorization; not supported in rich message buttons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot_username: Option<String>,
    /// Optional. Request the permission for the bot to send messages to the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_write_access: Option<bool>,
}

/// An inline button that switches the user to inline mode in a chosen chat.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SwitchInlineQueryChosenChat {
    /// Optional. The default inline query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Optional. True, if private chats with users can be chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_user_chats: Option<bool>,
    /// Optional. True, if private chats with bots can be chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_bot_chats: Option<bool>,
    /// Optional. True, if group and supergroup chats can be chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_group_chats: Option<bool>,
    /// Optional. True, if channel chats can be chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_channel_chats: Option<bool>,
}

/// A button that copies the specified text to the clipboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CopyTextButton {
    /// The text to be copied; 1-256 characters.
    pub text: String,
}

/// A disabled button which does nothing. Currently holds no information.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DisabledButton {}
