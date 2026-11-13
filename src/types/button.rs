//! Buttons embedded in rich messages (Bot API 10.3).

use serde::{Deserialize, Serialize};

use super::common::{
    CopyTextButton, DisabledButton, LoginUrl, SwitchInlineQueryChosenChat, WebAppInfo,
};
use super::rich_text::RichText;

/// Style of a [`RichMessageButton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonStyle {
    /// Danger (usually red).
    Danger,
    /// Success (usually green).
    Success,
    /// Primary (accent color).
    Primary,
    /// Shown as a regular link without borders; callback buttons only.
    Link,
}

/// A button in a rich message.
///
/// Exactly one of the fields other than `text` and `style` must be set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RichMessageButton {
    /// Text of the button; only plain text, custom emoji and date-time entities.
    pub text: RichText,
    /// Optional. Style of the button.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<ButtonStyle>,
    /// Optional. HTTP or `tg://` URL to be opened when the button is pressed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Optional. Data sent in a callback query, 1-64 bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_data: Option<String>,
    /// Optional. Web App launched when the button is pressed (private chats only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_app: Option<WebAppInfo>,
    /// Optional. HTTPS URL used to automatically authorize the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login_url: Option<LoginUrl>,
    /// Optional. Switch to inline mode in a chosen chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_inline_query: Option<String>,
    /// Optional. Switch to inline mode in the current chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_inline_query_current_chat: Option<String>,
    /// Optional. Switch to inline mode in a chat of the specified type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_inline_query_chosen_chat: Option<SwitchInlineQueryChosenChat>,
    /// Optional. Copy the specified text to the clipboard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy_text: Option<CopyTextButton>,
    /// Optional. The button is disabled and does nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<DisabledButton>,
}

impl RichMessageButton {
    fn empty(text: RichText) -> Self {
        Self {
            text,
            style: None,
            url: None,
            callback_data: None,
            web_app: None,
            login_url: None,
            switch_inline_query: None,
            switch_inline_query_current_chat: None,
            switch_inline_query_chosen_chat: None,
            copy_text: None,
            disabled: None,
        }
    }

    /// Creates a URL button.
    pub fn url(text: impl Into<RichText>, url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..Self::empty(text.into())
        }
    }

    /// Creates a callback button.
    pub fn callback(text: impl Into<RichText>, data: impl Into<String>) -> Self {
        Self {
            callback_data: Some(data.into()),
            ..Self::empty(text.into())
        }
    }

    /// Creates a copy-text button.
    pub fn copy_text(text: impl Into<RichText>, copy: impl Into<String>) -> Self {
        Self {
            copy_text: Some(CopyTextButton { text: copy.into() }),
            ..Self::empty(text.into())
        }
    }

    /// Sets the button style.
    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = Some(style);
        self
    }
}
