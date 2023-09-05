use indexmap::{IndexMap, IndexSet};
use iso8601_timestamp::Timestamp;
use revolt_models::v0::{Embed, MessageAuthor, MessageSort, MessageWebhook, PushNotification};
use revolt_result::Result;
use ulid::Ulid;

use crate::{
    events::client::EventV1,
    tasks::{self, ack::AckEvent},
    Channel, Database, File,
};

auto_derived_partial!(
    /// Message
    pub struct Message {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Unique value generated by client sending this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub nonce: Option<String>,
        /// Id of the channel this message was sent in
        pub channel: String,
        /// Id of the user or webhook that sent this message
        pub author: String,
        /// The webhook that sent this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub webhook: Option<MessageWebhook>,
        /// Message content
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<String>,
        /// Message Components
        #[serde(skip_serializing_if = "Option::is_none")]
        pub components: Option<Vec<Component>>,
        /// System message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub system: Option<SystemMessage>,
        /// Array of attachments
        #[serde(skip_serializing_if = "Option::is_none")]
        pub attachments: Option<Vec<File>>,
        /// Time at which this message was last edited
        #[serde(skip_serializing_if = "Option::is_none")]
        pub edited: Option<Timestamp>,
        /// Attached embeds to this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub embeds: Option<Vec<Embed>>,
        /// Array of user ids mentioned in this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mentions: Option<Vec<String>>,
        /// Array of message ids this message is replying to
        #[serde(skip_serializing_if = "Option::is_none")]
        pub replies: Option<Vec<String>>,
        /// Hashmap of emoji IDs to array of user IDs
        #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
        pub reactions: IndexMap<String, IndexSet<String>>,
        /// Information about how this message should be interacted with
        #[serde(skip_serializing_if = "Interactions::is_default", default)]
        pub interactions: Interactions,
        /// Name and / or avatar overrides for this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub masquerade: Option<Masquerade>,
    },
    "PartialMessage"
);

auto_derived!(
    /// System Event
    #[serde(tag = "type")]
    pub enum SystemMessage {
        #[serde(rename = "text")]
        Text { content: String },
        #[serde(rename = "user_added")]
        UserAdded { id: String, by: String },
        #[serde(rename = "user_remove")]
        UserRemove { id: String, by: String },
        #[serde(rename = "user_joined")]
        UserJoined { id: String },
        #[serde(rename = "user_left")]
        UserLeft { id: String },
        #[serde(rename = "user_kicked")]
        UserKicked { id: String },
        #[serde(rename = "user_banned")]
        UserBanned { id: String },
        #[serde(rename = "channel_renamed")]
        ChannelRenamed { name: String, by: String },
        #[serde(rename = "channel_description_changed")]
        ChannelDescriptionChanged { by: String },
        #[serde(rename = "channel_icon_changed")]
        ChannelIconChanged { by: String },
        #[serde(rename = "channel_ownership_changed")]
        ChannelOwnershipChanged { from: String, to: String },
    }

    /// Name and / or avatar override information
    pub struct Masquerade {
        /// Replace the display name shown on this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        /// Replace the avatar shown on this message (URL to image file)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar: Option<String>,
        /// Replace the display role colour shown on this message
        ///
        /// Must have `ManageRole` permission to use
        #[serde(skip_serializing_if = "Option::is_none")]
        pub colour: Option<String>,
    }

    /// Information to guide interactions on this message
    #[derive(Default)]
    pub struct Interactions {
        /// Reactions which should always appear and be distinct
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pub reactions: Option<IndexSet<String>>,
        /// Whether reactions should be restricted to the given list
        ///
        /// Can only be set to true if reactions list is of at least length 1
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub restrict_reactions: bool,
    }

    /// Appended Information
    pub struct AppendMessage {
        /// Additional embeds to include in this message
        #[serde(skip_serializing_if = "Option::is_none")]
        pub embeds: Option<Vec<Embed>>,
    }

    /// Message Time Period
    ///
    /// Filter and sort messages by time
    #[serde(untagged)]
    pub enum MessageTimePeriod {
        Relative {
            /// Message id to search around
            ///
            /// Specifying 'nearby' ignores 'before', 'after' and 'sort'.
            /// It will also take half of limit rounded as the limits to each side.
            /// It also fetches the message ID specified.
            nearby: String,
        },
        Absolute {
            /// Message id before which messages should be fetched
            before: Option<String>,
            /// Message id after which messages should be fetched
            after: Option<String>,
            /// Message sort direction
            sort: Option<MessageSort>,
        },
    }

    /// Message Filter
    pub struct MessageFilter {
        /// Parent channel ID
        pub channel: Option<String>,
        /// Message author ID
        pub author: Option<String>,
        /// Search query
        pub query: Option<String>,
    }

    /// Message Query
    pub struct MessageQuery {
        /// Maximum number of messages to fetch
        ///
        /// For fetching nearby messages, this is \`(limit + 1)\`.
        pub limit: Option<i64>,
        /// Filter to apply
        #[serde(flatten)]
        pub filter: MessageFilter,
        /// Time period to fetch
        #[serde(flatten)]
        pub time_period: MessageTimePeriod,
    }

    #[serde(tag = "type")]
    pub enum Component {
        #[serde(rename = "button")]
        Button {
            label: String,
            style: String,
            enabled: bool,
        },
        #[serde(rename = "line_break")]
        LineBreak,
        #[serde(rename = "status")]
        Status { label: String },
    }

    pub struct Interaction {
        pub message_id: String,
        pub nonce: String,
        pub channel_id: String,
        pub author_id: String,
        pub content: String,
    }
);

#[allow(clippy::derivable_impls)]
impl Default for Message {
    fn default() -> Self {
        Self {
            id: Default::default(),
            nonce: None,
            channel: Default::default(),
            author: Default::default(),
            webhook: None,
            content: None,
            system: None,
            attachments: None,
            edited: None,
            embeds: None,
            mentions: None,
            replies: None,
            reactions: Default::default(),
            interactions: Default::default(),
            masquerade: None,
            components: Default::default(),
        }
    }
}

#[allow(clippy::disallowed_methods)]
impl Message {
    /// Send a message without any notifications
    pub async fn send_without_notifications(
        &mut self,
        db: &Database,
        is_dm: bool,
        generate_embeds: bool,
    ) -> Result<()> {
        db.insert_message(self).await?;

        // Fan out events
        EventV1::Message(self.clone().into())
            .p(self.channel.to_string())
            .await;

        // Update last_message_id
        tasks::last_message_id::queue(self.channel.to_string(), self.id.to_string(), is_dm).await;

        // Add mentions for affected users
        if let Some(mentions) = &self.mentions {
            for user in mentions {
                tasks::ack::queue(
                    self.channel.to_string(),
                    user.to_string(),
                    AckEvent::AddMention {
                        ids: vec![self.id.to_string()],
                    },
                )
                .await;
            }
        }

        // Generate embeds
        if generate_embeds {
            if let Some(content) = &self.content {
                tasks::process_embeds::queue(
                    self.channel.to_string(),
                    self.id.to_string(),
                    content.clone(),
                )
                .await;
            }
        }

        Ok(())
    }

    /// Send a message
    pub async fn send(
        &mut self,
        db: &Database,
        author: MessageAuthor<'_>,
        channel: &Channel,
        generate_embeds: bool,
    ) -> Result<()> {
        self.send_without_notifications(
            db,
            matches!(channel, Channel::DirectMessage { .. }),
            generate_embeds,
        )
        .await?;

        // Push out Web Push notifications
        crate::tasks::web_push::queue(
            {
                match channel {
                    Channel::DirectMessage { recipients, .. }
                    | Channel::Group { recipients, .. } => recipients.clone(),
                    Channel::TextChannel { .. } => self.mentions.clone().unwrap_or_default(),
                    _ => vec![],
                }
            },
            PushNotification::from(self.clone().into(), Some(author), &channel.id()).await,
        )
        .await;

        Ok(())
    }

    /// Append content to message
    pub async fn append(
        db: &Database,
        id: String,
        channel: String,
        append: AppendMessage,
    ) -> Result<()> {
        db.append_message(&id, &append).await?;

        EventV1::MessageAppend {
            id,
            channel: channel.to_string(),
            append: append.into(),
        }
        .p(channel)
        .await;

        Ok(())
    }
}

impl SystemMessage {
    pub fn into_message(self, channel: String) -> Message {
        Message {
            id: Ulid::new().to_string(),
            channel,
            author: "00000000000000000000000000".to_string(),
            system: Some(self),

            ..Default::default()
        }
    }
}

impl Interactions {
    /// Check if we can use a given emoji to react
    pub fn can_use(&self, emoji: &str) -> bool {
        if self.restrict_reactions {
            if let Some(reactions) = &self.reactions {
                reactions.contains(emoji)
            } else {
                false
            }
        } else {
            true
        }
    }

    /// Check if default initialisation of fields
    pub fn is_default(&self) -> bool {
        !self.restrict_reactions && self.reactions.is_none()
    }
}
