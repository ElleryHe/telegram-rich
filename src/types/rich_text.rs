//! Rich formatted text ([`RichText`]) and its entity types.

use serde::{Deserialize, Serialize};

use super::button::RichMessageButton;
use super::common::User;

/// A rich formatted text.
///
/// It can be either a plain string, an array of [`RichText`], or a formatting
/// entity ([`RichTextEntity`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RichText {
    /// Plain text.
    Plain(String),
    /// Concatenation of rich texts.
    Concat(Vec<RichText>),
    /// A formatting entity.
    Entity(Box<RichTextEntity>),
}

impl RichText {
    /// Creates plain text.
    pub fn plain(s: impl Into<String>) -> Self {
        RichText::Plain(s.into())
    }

    /// Wraps an entity.
    pub fn entity(e: RichTextEntity) -> Self {
        RichText::Entity(Box::new(e))
    }
}

impl From<&str> for RichText {
    fn from(s: &str) -> Self {
        RichText::Plain(s.to_owned())
    }
}

impl From<String> for RichText {
    fn from(s: String) -> Self {
        RichText::Plain(s)
    }
}

impl From<Vec<RichText>> for RichText {
    fn from(v: Vec<RichText>) -> Self {
        RichText::Concat(v)
    }
}

impl From<RichTextEntity> for RichText {
    fn from(e: RichTextEntity) -> Self {
        RichText::entity(e)
    }
}

/// A formatting entity inside a [`RichText`], tagged by the `type` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RichTextEntity {
    /// A bold text.
    Bold {
        /// The text.
        text: RichText,
    },
    /// An italicized text.
    Italic {
        /// The text.
        text: RichText,
    },
    /// An underlined text.
    Underline {
        /// The text.
        text: RichText,
    },
    /// A strikethrough text.
    Strikethrough {
        /// The text.
        text: RichText,
    },
    /// A text covered by a spoiler.
    Spoiler {
        /// The text.
        text: RichText,
    },
    /// Formatted date and time.
    DateTime {
        /// The text.
        text: RichText,
        /// The Unix time associated with the entity.
        unix_time: i64,
        /// Date-time formatting string.
        date_time_format: String,
    },
    /// A mention of a Telegram user by their identifier.
    TextMention {
        /// The text.
        text: RichText,
        /// The mentioned user.
        user: User,
    },
    /// A subscript text.
    Subscript {
        /// The text.
        text: RichText,
    },
    /// A superscript text.
    Superscript {
        /// The text.
        text: RichText,
    },
    /// A marked text.
    Marked {
        /// The text.
        text: RichText,
    },
    /// A monowidth text.
    Code {
        /// The text.
        text: RichText,
    },
    /// A custom emoji.
    CustomEmoji {
        /// Unique identifier of the custom emoji.
        custom_emoji_id: String,
        /// Alternative emoji for the custom emoji.
        alternative_text: String,
    },
    /// A mathematical expression.
    MathematicalExpression {
        /// The expression in LaTeX format.
        expression: String,
    },
    /// A text with a link.
    Url {
        /// The text.
        text: RichText,
        /// URL of the link.
        url: String,
    },
    /// A text with an email address.
    EmailAddress {
        /// The text.
        text: RichText,
        /// The email address.
        email_address: String,
    },
    /// A text with a phone number.
    PhoneNumber {
        /// The text.
        text: RichText,
        /// The phone number.
        phone_number: String,
    },
    /// A text with a bank card number.
    BankCardNumber {
        /// The text.
        text: RichText,
        /// The bank card number.
        bank_card_number: String,
    },
    /// An address in the TON blockchain (Bot API 10.4).
    TonAddress {
        /// Text of the address.
        text: RichText,
        /// The address.
        address: String,
    },
    /// A mention by a username.
    Mention {
        /// The text.
        text: RichText,
        /// The username.
        username: String,
    },
    /// A hashtag.
    Hashtag {
        /// The text.
        text: RichText,
        /// The hashtag.
        hashtag: String,
    },
    /// A cashtag.
    Cashtag {
        /// The text.
        text: RichText,
        /// The cashtag.
        cashtag: String,
    },
    /// A bot command.
    BotCommand {
        /// The text.
        text: RichText,
        /// The bot command.
        bot_command: String,
    },
    /// A button (Bot API 10.3).
    Button {
        /// The button.
        button: RichMessageButton,
    },
    /// An anchor.
    Anchor {
        /// The name of the anchor.
        name: String,
    },
    /// A link to an anchor.
    AnchorLink {
        /// The link text.
        text: RichText,
        /// The name of the anchor; empty links back to the top of the message.
        anchor_name: String,
    },
    /// A reference.
    Reference {
        /// Text of the reference.
        text: RichText,
        /// The name of the reference.
        name: String,
    },
    /// A link to a reference.
    ReferenceLink {
        /// The link text.
        text: RichText,
        /// The name of the reference.
        reference_name: String,
    },
}
