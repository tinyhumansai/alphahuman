# AlphaHuman Tools

This document lists all available tools that AlphaHuman can use to interact with external services and perform actions. Tools are organized by integration and automatically updated when the app loads.

## Overview

AlphaHuman has access to **152 tools** across **4 integrations**.

**Quick Statistics:**
- **Gmail**: 7 tools
- **Telegram**: 48 tools
- **Notion**: 25 tools
- **Github**: 72 tools

## Available Tools


### Gmail Tools

This skill provides 7 tools for gmail integration.

#### get-email

**Description**: Get full details of a specific email by its ID, including headers, body content, and attachments.

**Parameters**:
- **format** (string): Message format level (default: full)
- **include_body** (boolean): Include email body content (default: true)
- **message_id** (string) **(required)**: The Gmail message ID to retrieve

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-email",
  "parameters": {
    "format": "example_format",
    "include_body": true,
    "message_id": "example_message_id"
  }
}
```

---

#### get-emails

**Description**: Get emails from Gmail with optional filtering by query, labels, and pagination. Supports Gmail search syntax. When accessToken is provided (e.g. from frontend after OAuth), uses it directly; otherwise uses the skill OAuth credential.

**Parameters**:
- **accessToken** (string): Optional OAuth access token. When provided (e.g. by frontend after OAuth), Gmail API is called directly with this token instead of the skill credential.
- **include_spam_trash** (boolean): Include emails from spam and trash (default: false)
- **label_ids** (array): Filter by specific label IDs (e.g., ["INBOX", "IMPORTANT"])
- **maxResults** (number): Alias for max_results (e.g. for frontend calls)
- **max_results** (number): Maximum number of emails to return (default: 20, max: 100)
- **page_token** (string): Token for pagination (returned from previous request)
- **query** (string): Search query using Gmail search syntax (e.g., "from:example@gmail.com", "subject:meeting", "is:unread")

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-emails",
  "parameters": {
    "accessToken": "example_accessToken",
    "include_spam_trash": true,
    "label_ids": [],
    "maxResults": 10,
    "max_results": 10,
    "page_token": "example_page_token",
    "query": "example_query"
  }
}
```

---

#### get-labels

**Description**: Get all Gmail labels including system and user-created labels with message counts and details.

**Parameters**:
- **include_hidden** (boolean): Include hidden labels (default: false)
- **type** (string): Filter labels by type (default: all)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-labels",
  "parameters": {
    "include_hidden": true,
    "type": "example_type"
  }
}
```

---

#### get-profile

**Description**: Get Gmail user profile information including email address, total message counts, and account details. Optional accessToken for frontend calls.

**Parameters**:
- **accessToken** (string): Optional OAuth access token (e.g. from frontend).

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-profile",
  "parameters": {
    "accessToken": "example_accessToken"
  }
}
```

---

#### mark-email

**Description**: Mark emails with specific status (read/unread, important, starred) or add/remove labels.

**Parameters**:
- **action** (string) **(required)**: Action to perform on the messages
- **label_ids** (array): Label IDs to add or remove (required for add_labels/remove_labels actions)
- **message_ids** (array) **(required)**: Array of message IDs to modify

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "mark-email",
  "parameters": {
    "action": "example_action",
    "label_ids": [],
    "message_ids": []
  }
}
```

---

#### search-emails

**Description**: Search emails using advanced Gmail query syntax. Supports complex queries with operators like from:, to:, subject:, has:attachment, is:unread, etc.

**Parameters**:
- **include_spam_trash** (boolean): Include results from spam and trash folders (default: false)
- **max_results** (number): Maximum number of results to return (default: 20, max: 100)
- **page_token** (string): Token for pagination (from previous search)
- **query** (string) **(required)**: Gmail search query (e.g., "from:john@example.com subject:meeting is:unread", "has:attachment after:2023/01/01")

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-emails",
  "parameters": {
    "include_spam_trash": true,
    "max_results": 10,
    "page_token": "example_page_token",
    "query": "example_query"
  }
}
```

---

#### send-email

**Description**: Send an email through Gmail with support for HTML/text content, attachments, CC/BCC recipients, and reply threading.

**Parameters**:
- **attachments** (array): File attachments (optional)
- **bcc** (array): BCC recipients (optional)
- **body_html** (string): HTML email body (optional if body_text provided)
- **body_text** (string): Plain text email body (optional if body_html provided)
- **cc** (array): CC recipients (optional)
- **reply_to_message_id** (string): Message ID being replied to (optional)
- **subject** (string) **(required)**: Email subject line
- **thread_id** (string): Thread ID for replies (optional)
- **to** (array) **(required)**: Primary recipients

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "send-email",
  "parameters": {
    "attachments": [],
    "bcc": [],
    "body_html": "example_body_html",
    "body_text": "example_body_text",
    "cc": [],
    "reply_to_message_id": "example_reply_to_message_id",
    "subject": "example_subject",
    "thread_id": "example_thread_id",
    "to": []
  }
}
```

---


### Telegram Tools

This skill provides 48 tools for telegram integration.

#### telegram-status

**Description**: Get current Telegram connection and authentication status.

**Parameters**: *None*

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "telegram-status",
  "parameters": {}
}
```

---

#### get-me

**Description**: Get Telegram user information for the currently logged-in account. Returns profile including name, username, and phone number.

**Parameters**: *None*

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-me",
  "parameters": {}
}
```

---

#### get-chats

**Description**: Get Telegram chat list with optional filtering. Returns chats sorted by pinned status and recent activity. Use this to browse conversations, find specific chats, or filter by type (private, group, channel).

**Parameters**:
- **limit** (string): Maximum number of chats to return (default: 50, max: 100)
- **offset** (string): Number of chats to skip for pagination
- **search** (string): Search term to filter chats by title or username
- **type** (string): Filter by chat type
- **unread_only** (string): Only return chats with unread messages (true/false)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-chats",
  "parameters": {
    "limit": "example_limit",
    "offset": "example_offset",
    "search": "example_search",
    "type": "example_type",
    "unread_only": "example_unread_only"
  }
}
```

---

#### get-messages

**Description**: Get messages from a specific Telegram chat. Supports filtering by content type, text search, and pagination. Returns messages in reverse chronological order (newest first).

**Parameters**:
- **before_id** (string): Get messages before this message ID (for pagination)
- **chat_id** (string) **(required)**: The chat ID to get messages from (required)
- **content_type** (string): Filter by message content type
- **limit** (string): Maximum number of messages to return (default: 50, max: 100)
- **search** (string): Search term to filter messages by text content

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-messages",
  "parameters": {
    "before_id": "example_before_id",
    "chat_id": "example_chat_id",
    "content_type": "example_content_type",
    "limit": "example_limit",
    "search": "example_search"
  }
}
```

---

#### get-contacts

**Description**: Get Telegram contacts and users. Can filter to show only saved contacts or search by name/username. Returns user profiles including status, premium status, and bot flag.

**Parameters**:
- **contacts_only** (string): Only return users who are saved contacts (true/false)
- **limit** (string): Maximum number of contacts to return (default: 50, max: 100)
- **offset** (string): Number of contacts to skip for pagination
- **search** (string): Search term to filter by name or username

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-contacts",
  "parameters": {
    "contacts_only": "example_contacts_only",
    "limit": "example_limit",
    "offset": "example_offset",
    "search": "example_search"
  }
}
```

---

#### get-chat-stats

**Description**: Get detailed statistics for a Telegram chat. Returns message counts, content type breakdown, top senders, and activity date range. Useful for understanding chat activity and composition.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID to get statistics for (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-chat-stats",
  "parameters": {
    "chat_id": "example_chat_id"
  }
}
```

---

#### send-message

**Description**: Send a text message to a Telegram chat. Supports replying to a specific message. Returns the sent message details.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID to send the message to (required)
- **reply_to_message_id** (string): Message ID to reply to (optional)
- **text** (string) **(required)**: The message text to send (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "send-message",
  "parameters": {
    "chat_id": "example_chat_id",
    "reply_to_message_id": "example_reply_to_message_id",
    "text": "example_text"
  }
}
```

---

#### edit-message

**Description**: Edit a previously sent text message in a Telegram chat. Only messages sent by you can be edited.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID containing the message (required)
- **message_id** (string) **(required)**: The message ID to edit (required)
- **text** (string) **(required)**: The new message text (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "edit-message",
  "parameters": {
    "chat_id": "example_chat_id",
    "message_id": "example_message_id",
    "text": "example_text"
  }
}
```

---

#### delete-messages

**Description**: Delete one or more messages from a Telegram chat. By default deletes for all users (revoke). Pass revoke=false to delete only for yourself.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID containing the messages (required)
- **message_ids** (string) **(required)**: Comma-separated list of message IDs to delete (required)
- **revoke** (string): Delete for all users (true) or only yourself (false). Default: true

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "delete-messages",
  "parameters": {
    "chat_id": "example_chat_id",
    "message_ids": "example_message_ids",
    "revoke": "example_revoke"
  }
}
```

---

#### forward-messages

**Description**: Forward one or more messages from one Telegram chat to another. The forwarded messages will show the original sender.

**Parameters**:
- **chat_id** (string) **(required)**: The destination chat ID to forward messages to (required)
- **from_chat_id** (string) **(required)**: The source chat ID to forward messages from (required)
- **message_ids** (string) **(required)**: Comma-separated list of message IDs to forward (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "forward-messages",
  "parameters": {
    "chat_id": "example_chat_id",
    "from_chat_id": "example_from_chat_id",
    "message_ids": "example_message_ids"
  }
}
```

---

#### search-chat-messages

**Description**: Search messages within a specific Telegram chat by keyword. Returns matching messages in reverse chronological order.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID to search in (required)
- **from_message_id** (string): Start searching from this message ID (for pagination)
- **limit** (string): Maximum number of results (default: 20, max: 50)
- **query** (string) **(required)**: Search query text (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-chat-messages",
  "parameters": {
    "chat_id": "example_chat_id",
    "from_message_id": "example_from_message_id",
    "limit": "example_limit",
    "query": "example_query"
  }
}
```

---

#### search-messages-global

**Description**: Search messages across all Telegram chats by keyword. Returns matching messages from any conversation.

**Parameters**:
- **limit** (string): Maximum number of results (default: 20, max: 50)
- **query** (string) **(required)**: Search query text (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-messages-global",
  "parameters": {
    "limit": "example_limit",
    "query": "example_query"
  }
}
```

---

#### pin-message

**Description**: Pin or unpin a message in a Telegram chat. Pinned messages appear at the top of the chat for all members.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **disable_notification** (string): Disable notification for pin (true/false). Default: false
- **message_id** (string) **(required)**: The message ID to pin/unpin (required)
- **unpin** (string): Set to "true" to unpin instead of pin. Default: false

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "pin-message",
  "parameters": {
    "chat_id": "example_chat_id",
    "disable_notification": "example_disable_notification",
    "message_id": "example_message_id",
    "unpin": "example_unpin"
  }
}
```

---

#### mark-as-read

**Description**: Mark messages in a Telegram chat as read. If no message_ids are provided, marks all messages in the chat as read by reading the latest message.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID to mark as read (required)
- **message_ids** (string): Comma-separated list of message IDs to mark as read. If omitted, marks latest messages.

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "mark-as-read",
  "parameters": {
    "chat_id": "example_chat_id",
    "message_ids": "example_message_ids"
  }
}
```

---

#### get-chat

**Description**: Get detailed information about a specific Telegram chat by ID. Returns chat type, title, member count, permissions, and other metadata.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID to get info for (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-chat",
  "parameters": {
    "chat_id": "example_chat_id"
  }
}
```

---

#### create-private-chat

**Description**: Open or create a direct message conversation with a Telegram user by their user ID.

**Parameters**:
- **user_id** (string) **(required)**: The user ID to start a DM with (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-private-chat",
  "parameters": {
    "user_id": "example_user_id"
  }
}
```

---

#### create-group

**Description**: Create a new Telegram group. Provide a title and at least one user ID to add as a member.

**Parameters**:
- **title** (string) **(required)**: The group title (required)
- **user_ids** (string) **(required)**: Comma-separated list of user IDs to add to the group (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-group",
  "parameters": {
    "title": "example_title",
    "user_ids": "example_user_ids"
  }
}
```

---

#### create-channel

**Description**: Create a new Telegram channel. Channels are broadcast-only chats where only admins can post.

**Parameters**:
- **description** (string): Channel description (optional)
- **title** (string) **(required)**: The channel title (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-channel",
  "parameters": {
    "description": "example_description",
    "title": "example_title"
  }
}
```

---

#### join-chat

**Description**: Join a Telegram chat by invite link (e.g., https://t.me/+abc123 or https://t.me/joinchat/abc123).

**Parameters**:
- **invite_link** (string) **(required)**: The chat invite link (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "join-chat",
  "parameters": {
    "invite_link": "example_invite_link"
  }
}
```

---

#### leave-chat

**Description**: Leave a Telegram group or channel by chat ID.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID to leave (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "leave-chat",
  "parameters": {
    "chat_id": "example_chat_id"
  }
}
```

---

#### set-chat-title

**Description**: Change a Telegram group or channel's title. Requires admin privileges.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **title** (string) **(required)**: The new title (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "set-chat-title",
  "parameters": {
    "chat_id": "example_chat_id",
    "title": "example_title"
  }
}
```

---

#### get-chat-invite-link

**Description**: Get or create an invite link for a Telegram group or channel. Requires admin privileges.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **member_limit** (string): Maximum number of members that can join via this link (0 = unlimited)
- **name** (string): Optional name for the invite link

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-chat-invite-link",
  "parameters": {
    "chat_id": "example_chat_id",
    "member_limit": "example_member_limit",
    "name": "example_name"
  }
}
```

---

#### get-user

**Description**: Get basic information about a Telegram user by their user ID.

**Parameters**:
- **user_id** (string) **(required)**: The user ID to look up (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-user",
  "parameters": {
    "user_id": "example_user_id"
  }
}
```

---

#### get-user-profile

**Description**: Get the full profile for a Telegram user, including bio, common group count, and other extended info.

**Parameters**:
- **user_id** (string) **(required)**: The user ID to look up (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-user-profile",
  "parameters": {
    "user_id": "example_user_id"
  }
}
```

---

#### search-public-chat

**Description**: Find a Telegram user, channel, or group by their @username. Returns the matching chat/user info.

**Parameters**:
- **username** (string) **(required)**: The username to search for, without the @ prefix (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-public-chat",
  "parameters": {
    "username": "example_username"
  }
}
```

---

#### add-contact

**Description**: Add a Telegram user as a saved contact. Requires the user ID and a first name.

**Parameters**:
- **first_name** (string) **(required)**: Contact's first name (required)
- **last_name** (string): Contact's last name (optional)
- **phone_number** (string): Contact's phone number (optional)
- **user_id** (string) **(required)**: The user ID to add as contact (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "add-contact",
  "parameters": {
    "first_name": "example_first_name",
    "last_name": "example_last_name",
    "phone_number": "example_phone_number",
    "user_id": "example_user_id"
  }
}
```

---

#### remove-contact

**Description**: Remove a Telegram user from your saved contacts.

**Parameters**:
- **user_id** (string) **(required)**: The user ID to remove from contacts (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "remove-contact",
  "parameters": {
    "user_id": "example_user_id"
  }
}
```

---

#### block-user

**Description**: Block or unblock a Telegram user. Blocked users cannot send you messages.

**Parameters**:
- **unblock** (string): Set to "true" to unblock instead of block. Default: false
- **user_id** (string) **(required)**: The user ID to block/unblock (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "block-user",
  "parameters": {
    "unblock": "example_unblock",
    "user_id": "example_user_id"
  }
}
```

---

#### add-reaction

**Description**: Add a reaction emoji to a Telegram message. Common reactions: 👍 ❤️ 🔥 🎉 😮 😢 💯 👎

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **emoji** (string) **(required)**: The reaction emoji (required)
- **message_id** (string) **(required)**: The message ID to react to (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "add-reaction",
  "parameters": {
    "chat_id": "example_chat_id",
    "emoji": "example_emoji",
    "message_id": "example_message_id"
  }
}
```

---

#### remove-reaction

**Description**: Remove a reaction emoji from a Telegram message.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **emoji** (string) **(required)**: The reaction emoji to remove (required)
- **message_id** (string) **(required)**: The message ID (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "remove-reaction",
  "parameters": {
    "chat_id": "example_chat_id",
    "emoji": "example_emoji",
    "message_id": "example_message_id"
  }
}
```

---

#### send-sticker

**Description**: Send a sticker to a Telegram chat by its file ID. Use search-stickers to find sticker IDs.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID to send the sticker to (required)
- **reply_to_message_id** (string): Message ID to reply to (optional)
- **sticker_id** (string) **(required)**: The sticker file ID (required). Get from search-stickers.

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "send-sticker",
  "parameters": {
    "chat_id": "example_chat_id",
    "reply_to_message_id": "example_reply_to_message_id",
    "sticker_id": "example_sticker_id"
  }
}
```

---

#### send-gif

**Description**: Send a GIF/animation to a Telegram chat by URL or file ID.

**Parameters**:
- **caption** (string): Optional caption for the GIF
- **chat_id** (string) **(required)**: The chat ID to send the GIF to (required)
- **reply_to_message_id** (string): Message ID to reply to (optional)
- **url** (string) **(required)**: The GIF URL or file ID (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "send-gif",
  "parameters": {
    "caption": "example_caption",
    "chat_id": "example_chat_id",
    "reply_to_message_id": "example_reply_to_message_id",
    "url": "example_url"
  }
}
```

---

#### search-stickers

**Description**: Search for Telegram stickers by emoji or keyword. Returns sticker file IDs that can be used with send-sticker.

**Parameters**:
- **limit** (string): Maximum number of stickers to return (default: 10, max: 20)
- **query** (string) **(required)**: Emoji or keyword to search for stickers (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-stickers",
  "parameters": {
    "limit": "example_limit",
    "query": "example_query"
  }
}
```

---

#### get-chat-folders

**Description**: List all Telegram chat folders (filters) configured for the account.

**Parameters**: *None*

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-chat-folders",
  "parameters": {}
}
```

---

#### create-chat-folder

**Description**: Create a new Telegram chat folder. You can specify which chats to include/exclude and filter settings.

**Parameters**:
- **excluded_chat_ids** (string): Comma-separated list of chat IDs to exclude
- **include_bots** (string): Include bots (true/false). Default: false
- **include_channels** (string): Include channels (true/false). Default: false
- **include_contacts** (string): Include contacts (true/false). Default: false
- **include_groups** (string): Include groups (true/false). Default: false
- **include_non_contacts** (string): Include non-contacts (true/false). Default: false
- **included_chat_ids** (string): Comma-separated list of chat IDs to include
- **title** (string) **(required)**: The folder title (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-chat-folder",
  "parameters": {
    "excluded_chat_ids": "example_excluded_chat_ids",
    "include_bots": "example_include_bots",
    "include_channels": "example_include_channels",
    "include_contacts": "example_include_contacts",
    "include_groups": "example_include_groups",
    "include_non_contacts": "example_include_non_contacts",
    "included_chat_ids": "example_included_chat_ids",
    "title": "example_title"
  }
}
```

---

#### edit-chat-folder

**Description**: Edit an existing Telegram chat folder. Change title, included/excluded chats, or filter settings.

**Parameters**:
- **excluded_chat_ids** (string): Comma-separated list of chat IDs to exclude (replaces existing)
- **folder_id** (string) **(required)**: The folder ID to edit (required)
- **include_bots** (string): Include bots (true/false)
- **include_channels** (string): Include channels (true/false)
- **include_contacts** (string): Include contacts (true/false)
- **include_groups** (string): Include groups (true/false)
- **include_non_contacts** (string): Include non-contacts (true/false)
- **included_chat_ids** (string): Comma-separated list of chat IDs to include (replaces existing)
- **title** (string): New folder title (optional)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "edit-chat-folder",
  "parameters": {
    "excluded_chat_ids": "example_excluded_chat_ids",
    "folder_id": "example_folder_id",
    "include_bots": "example_include_bots",
    "include_channels": "example_include_channels",
    "include_contacts": "example_include_contacts",
    "include_groups": "example_include_groups",
    "include_non_contacts": "example_include_non_contacts",
    "included_chat_ids": "example_included_chat_ids",
    "title": "example_title"
  }
}
```

---

#### delete-chat-folder

**Description**: Delete a Telegram chat folder. Chats in the folder are not affected.

**Parameters**:
- **folder_id** (string) **(required)**: The folder ID to delete (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "delete-chat-folder",
  "parameters": {
    "folder_id": "example_folder_id"
  }
}
```

---

#### get-chat-members

**Description**: Get the members of a Telegram group or channel. Can filter by type (recent, administrators, banned, bots).

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID to get members from (required)
- **filter** (string): Filter members by type
- **limit** (string): Maximum number of members to return (default: 50, max: 200)
- **offset** (string): Number of members to skip for pagination
- **query** (string): Search query to filter members by name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-chat-members",
  "parameters": {
    "chat_id": "example_chat_id",
    "filter": "example_filter",
    "limit": "example_limit",
    "offset": "example_offset",
    "query": "example_query"
  }
}
```

---

#### add-chat-member

**Description**: Add one or more users to a Telegram group. Requires admin privileges in the group.

**Parameters**:
- **chat_id** (string) **(required)**: The group chat ID (required)
- **user_ids** (string) **(required)**: Comma-separated list of user IDs to add (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "add-chat-member",
  "parameters": {
    "chat_id": "example_chat_id",
    "user_ids": "example_user_ids"
  }
}
```

---

#### ban-chat-member

**Description**: Ban (kick) a member from a Telegram group or channel. Requires admin privileges.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **user_id** (string) **(required)**: The user ID to ban (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "ban-chat-member",
  "parameters": {
    "chat_id": "example_chat_id",
    "user_id": "example_user_id"
  }
}
```

---

#### promote-chat-member

**Description**: Promote a member to admin or demote an admin in a Telegram group/channel. Specify individual admin rights to grant.

**Parameters**:
- **can_change_info** (string): Can change chat info (true/false)
- **can_delete_messages** (string): Can delete messages (true/false)
- **can_edit_messages** (string): Can edit messages in channels (true/false)
- **can_invite_users** (string): Can invite users (true/false)
- **can_manage_chat** (string): Can manage chat settings (true/false)
- **can_pin_messages** (string): Can pin messages (true/false)
- **can_post_messages** (string): Can post in channels (true/false)
- **can_promote_members** (string): Can promote other members (true/false)
- **can_restrict_members** (string): Can restrict members (true/false)
- **chat_id** (string) **(required)**: The chat ID (required)
- **custom_title** (string): Custom admin title (optional)
- **demote** (string): Set to "true" to demote to regular member
- **user_id** (string) **(required)**: The user ID to promote/demote (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "promote-chat-member",
  "parameters": {
    "can_change_info": "example_can_change_info",
    "can_delete_messages": "example_can_delete_messages",
    "can_edit_messages": "example_can_edit_messages",
    "can_invite_users": "example_can_invite_users",
    "can_manage_chat": "example_can_manage_chat",
    "can_pin_messages": "example_can_pin_messages",
    "can_post_messages": "example_can_post_messages",
    "can_promote_members": "example_can_promote_members",
    "can_restrict_members": "example_can_restrict_members",
    "chat_id": "example_chat_id",
    "custom_title": "example_custom_title",
    "demote": "example_demote",
    "user_id": "example_user_id"
  }
}
```

---

#### get-chat-admins

**Description**: Get the list of administrators for a Telegram group or channel.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-chat-admins",
  "parameters": {
    "chat_id": "example_chat_id"
  }
}
```

---

#### set-chat-permissions

**Description**: Set the default permissions for all members in a Telegram group. Requires admin privileges. Controls what regular (non-admin) members can do.

**Parameters**:
- **can_add_link_previews** (string): Can add link previews (true/false)
- **can_change_info** (string): Can change chat info (true/false)
- **can_invite_users** (string): Can invite users (true/false)
- **can_pin_messages** (string): Can pin messages (true/false)
- **can_send_audios** (string): Can send audio (true/false)
- **can_send_basic_messages** (string): Can send text messages (true/false)
- **can_send_documents** (string): Can send documents (true/false)
- **can_send_photos** (string): Can send photos (true/false)
- **can_send_polls** (string): Can send polls (true/false)
- **can_send_videos** (string): Can send videos (true/false)
- **chat_id** (string) **(required)**: The chat ID (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "set-chat-permissions",
  "parameters": {
    "can_add_link_previews": "example_can_add_link_previews",
    "can_change_info": "example_can_change_info",
    "can_invite_users": "example_can_invite_users",
    "can_pin_messages": "example_can_pin_messages",
    "can_send_audios": "example_can_send_audios",
    "can_send_basic_messages": "example_can_send_basic_messages",
    "can_send_documents": "example_can_send_documents",
    "can_send_photos": "example_can_send_photos",
    "can_send_polls": "example_can_send_polls",
    "can_send_videos": "example_can_send_videos",
    "chat_id": "example_chat_id"
  }
}
```

---

#### send-photo

**Description**: Send a photo to a Telegram chat by URL. Supports optional caption and reply.

**Parameters**:
- **caption** (string): Optional photo caption
- **chat_id** (string) **(required)**: The chat ID to send the photo to (required)
- **reply_to_message_id** (string): Message ID to reply to (optional)
- **url** (string) **(required)**: The photo URL (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "send-photo",
  "parameters": {
    "caption": "example_caption",
    "chat_id": "example_chat_id",
    "reply_to_message_id": "example_reply_to_message_id",
    "url": "example_url"
  }
}
```

---

#### send-document

**Description**: Send a document or file to a Telegram chat by URL. Supports optional caption and reply.

**Parameters**:
- **caption** (string): Optional document caption
- **chat_id** (string) **(required)**: The chat ID to send the document to (required)
- **reply_to_message_id** (string): Message ID to reply to (optional)
- **url** (string) **(required)**: The document URL (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "send-document",
  "parameters": {
    "caption": "example_caption",
    "chat_id": "example_chat_id",
    "reply_to_message_id": "example_reply_to_message_id",
    "url": "example_url"
  }
}
```

---

#### get-message

**Description**: Get a single Telegram message by its chat ID and message ID. Returns the full message content and metadata.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **message_id** (string) **(required)**: The message ID (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-message",
  "parameters": {
    "chat_id": "example_chat_id",
    "message_id": "example_message_id"
  }
}
```

---

#### get-message-link

**Description**: Get a shareable link to a specific Telegram message. Works for messages in public groups and channels.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **message_id** (string) **(required)**: The message ID (required)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-message-link",
  "parameters": {
    "chat_id": "example_chat_id",
    "message_id": "example_message_id"
  }
}
```

---

#### mute-chat

**Description**: Mute or unmute a Telegram chat's notifications. Muting prevents notifications but messages still appear in the chat list.

**Parameters**:
- **chat_id** (string) **(required)**: The chat ID (required)
- **duration** (string): Mute duration in seconds. Use 0 to unmute, 2147483647 for forever. Default: forever.
- **mute** (string): Set to "true" to mute, "false" to unmute. Default: true

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "mute-chat",
  "parameters": {
    "chat_id": "example_chat_id",
    "duration": "example_duration",
    "mute": "example_mute"
  }
}
```

---


### Notion Tools

This skill provides 25 tools for notion integration.

#### append-blocks

**Description**: Append child blocks to a page or block. Supports various block types.

**Parameters**:
- **block_id** (string) **(required)**: The parent page or block ID
- **blocks** (string) **(required)**: JSON string of blocks array. Example: [{"type":"paragraph","paragraph":{"rich_text":[{"text":{"content":"Hello"}}]}}]

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "append-blocks",
  "parameters": {
    "block_id": "example_block_id",
    "blocks": "example_blocks"
  }
}
```

---

#### append-text

**Description**: Append text content to a page or block. Creates paragraph blocks with the given text.

**Parameters**:
- **block_id** (string) **(required)**: The page or block ID to append to
- **text** (string) **(required)**: Text content to append

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "append-text",
  "parameters": {
    "block_id": "example_block_id",
    "text": "example_text"
  }
}
```

---

#### create-comment

**Description**: Create a comment on a page or in a discussion thread. Must specify either page_id (for new discussion) or discussion_id (to reply).

**Parameters**:
- **discussion_id** (string): Discussion ID to reply to an existing thread
- **page_id** (string): Page ID to start a new discussion on
- **text** (string) **(required)**: Comment text content

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-comment",
  "parameters": {
    "discussion_id": "example_discussion_id",
    "page_id": "example_page_id",
    "text": "example_text"
  }
}
```

---

#### create-database

**Description**: Create a new database in Notion. Must specify parent page and property schema.

**Parameters**:
- **parent_page_id** (string) **(required)**: Parent page ID where the database will be created
- **properties** (string): JSON string of properties schema. Example: {"Name":{"title":{}},"Status":{"select":{"options":[{"name":"Todo"},{"name":"Done"}]}}}
- **title** (string) **(required)**: Database title

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-database",
  "parameters": {
    "parent_page_id": "example_parent_page_id",
    "properties": "example_properties",
    "title": "example_title"
  }
}
```

---

#### create-page

**Description**: Create a new page in Notion. Parent can be another page or a database. For database parents, properties must match the database schema.

**Parameters**:
- **content** (string): Initial text content (creates a paragraph block)
- **parent_id** (string) **(required)**: Parent page ID or database ID
- **parent_type** (string): Type of parent (default: page_id)
- **properties** (string): JSON string of additional properties (for database pages)
- **title** (string) **(required)**: Page title

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-page",
  "parameters": {
    "content": "example_content",
    "parent_id": "example_parent_id",
    "parent_type": "example_parent_type",
    "properties": "example_properties",
    "title": "example_title"
  }
}
```

---

#### delete-block

**Description**: Delete a block. This permanently removes the block from Notion.

**Parameters**:
- **block_id** (string) **(required)**: The block ID to delete

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "delete-block",
  "parameters": {
    "block_id": "example_block_id"
  }
}
```

---

#### delete-page

**Description**: Delete (archive) a page. Archived pages can be restored from Notion's trash.

**Parameters**:
- **page_id** (string) **(required)**: The page ID to delete/archive

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "delete-page",
  "parameters": {
    "page_id": "example_page_id"
  }
}
```

---

#### get-block

**Description**: Get a block by its ID. Returns the block's type and content.

**Parameters**:
- **block_id** (string) **(required)**: The block ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-block",
  "parameters": {
    "block_id": "example_block_id"
  }
}
```

---

#### get-block-children

**Description**: Get the children blocks of a block or page.

**Parameters**:
- **block_id** (string) **(required)**: The parent block or page ID
- **page_size** (number): Number of blocks (default 50, max 100)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-block-children",
  "parameters": {
    "block_id": "example_block_id",
    "page_size": 10
  }
}
```

---

#### get-database

**Description**: Get a database's schema and metadata. Shows all properties and their types.

**Parameters**:
- **database_id** (string) **(required)**: The database ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-database",
  "parameters": {
    "database_id": "example_database_id"
  }
}
```

---

#### get-page

**Description**: Get a page's metadata and properties by its ID. Use notion-get-page-content to get the actual content/blocks.

**Parameters**:
- **page_id** (string) **(required)**: The page ID (UUID format, with or without dashes)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-page",
  "parameters": {
    "page_id": "example_page_id"
  }
}
```

---

#### get-page-content

**Description**: Get the content blocks of a page. Returns the text and structure of the page. Use recursive=true to also get nested blocks.

**Parameters**:
- **page_id** (string) **(required)**: The page ID to get content from
- **page_size** (number): Number of blocks to return (default 50, max 100)
- **recursive** (string): Whether to fetch nested blocks (default: false)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-page-content",
  "parameters": {
    "page_id": "example_page_id",
    "page_size": 10,
    "recursive": "example_recursive"
  }
}
```

---

#### get-user

**Description**: Get a user by their ID.

**Parameters**:
- **user_id** (string) **(required)**: The user ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-user",
  "parameters": {
    "user_id": "example_user_id"
  }
}
```

---

#### list-all-databases

**Description**: List all databases in the workspace that the integration has access to.

**Parameters**:
- **page_size** (number): Number of results (default 20, max 100)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-all-databases",
  "parameters": {
    "page_size": 10
  }
}
```

---

#### list-all-pages

**Description**: List all pages in the workspace that the integration has access to.

**Parameters**:
- **page_size** (number): Number of results to return (default 20, max 100)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-all-pages",
  "parameters": {
    "page_size": 10
  }
}
```

---

#### list-comments

**Description**: List comments on a block or page.

**Parameters**:
- **block_id** (string) **(required)**: Block or page ID to get comments for
- **page_size** (number): Number of results (default 20, max 100)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-comments",
  "parameters": {
    "block_id": "example_block_id",
    "page_size": 10
  }
}
```

---

#### list-users

**Description**: List all users in the workspace that the integration can see.

**Parameters**:
- **page_size** (number): Number of results (default 20, max 100)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-users",
  "parameters": {
    "page_size": 10
  }
}
```

---

#### query-database

**Description**: Query a database with optional filters and sorts. Returns database rows/pages.

**Parameters**:
- **database_id** (string) **(required)**: The database ID to query
- **filter** (string): JSON string of filter object (Notion filter syntax)
- **page_size** (number): Number of results (default 20, max 100)
- **sorts** (string): JSON string of sorts array (Notion sort syntax)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "query-database",
  "parameters": {
    "database_id": "example_database_id",
    "filter": "example_filter",
    "page_size": 10,
    "sorts": "example_sorts"
  }
}
```

---

#### search

**Description**: Search for pages and databases in your Notion workspace. Can filter by type (page or database) and returns matching results.

**Parameters**:
- **filter** (string): Filter results by type
- **page_size** (number): Number of results to return (default 20, max 100)
- **query** (string): Search query (optional, returns recent if empty)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search",
  "parameters": {
    "filter": "example_filter",
    "page_size": 10,
    "query": "example_query"
  }
}
```

---

#### summarize-pages

**Description**: AI summarization of Notion pages is now handled by the backend server. Synced page content is submitted to the server which runs summarization.

**Parameters**: *None*

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "summarize-pages",
  "parameters": {}
}
```

---

#### sync-now

**Description**: Trigger an immediate Notion sync to refresh local data. Returns sync results including counts of synced pages and databases.

**Parameters**: *None*

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "sync-now",
  "parameters": {}
}
```

---

#### sync-status

**Description**: Get the current Notion sync status including last sync time, total synced pages/databases, sync progress, and any errors.

**Parameters**: *None*

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "sync-status",
  "parameters": {}
}
```

---

#### update-block

**Description**: Update a block's content. The structure depends on the block type.

**Parameters**:
- **archived** (string): Set to true to archive the block
- **block_id** (string) **(required)**: The block ID to update
- **content** (string): JSON string of the block type content. Example for paragraph: {"paragraph":{"rich_text":[{"text":{"content":"Updated text"}}]}}

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "update-block",
  "parameters": {
    "archived": "example_archived",
    "block_id": "example_block_id",
    "content": "example_content"
  }
}
```

---

#### update-database

**Description**: Update a database's title or properties schema.

**Parameters**:
- **database_id** (string) **(required)**: The database ID to update
- **properties** (string): JSON string of properties to add or update
- **title** (string): New title (optional)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "update-database",
  "parameters": {
    "database_id": "example_database_id",
    "properties": "example_properties",
    "title": "example_title"
  }
}
```

---

#### update-page

**Description**: Update a page's properties. Can update title and other properties. Use notion-append-text to add content blocks.

**Parameters**:
- **archived** (string): Set to true to archive the page
- **page_id** (string) **(required)**: The page ID to update
- **properties** (string): JSON string of properties to update
- **title** (string): New title (optional)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "update-page",
  "parameters": {
    "archived": "example_archived",
    "page_id": "example_page_id",
    "properties": "example_properties",
    "title": "example_title"
  }
}
```

---


### Github Tools

This skill provides 72 tools for github integration.

#### list-repos

**Description**: List repositories for the authenticated user or a specific owner

**Parameters**:
- **limit** (number): Maximum number of repositories to return
- **owner** (string): Repository owner (user or org). Defaults to the authenticated user
- **sort** (string): Sort field
- **visibility** (string): Filter by visibility

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-repos",
  "parameters": {
    "limit": 10,
    "owner": "example_owner",
    "sort": "example_sort",
    "visibility": "example_visibility"
  }
}
```

---

#### get-repo

**Description**: Get detailed information about a specific repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-repo",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### create-repo

**Description**: Create a new repository for the authenticated user

**Parameters**:
- **auto_init** (boolean): Initialize with a README
- **description** (string): Repository description
- **name** (string) **(required)**: Repository name
- **visibility** (string): Repository visibility

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-repo",
  "parameters": {
    "auto_init": true,
    "description": "example_description",
    "name": "example_name",
    "visibility": "example_visibility"
  }
}
```

---

#### fork-repo

**Description**: Fork a repository to the authenticated user's account

**Parameters**:
- **fork_name** (string): Custom name for the forked repository
- **owner** (string) **(required)**: Owner of the repository to fork
- **repo** (string) **(required)**: Repository name to fork

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "fork-repo",
  "parameters": {
    "fork_name": "example_fork_name",
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### delete-repo

**Description**: Permanently delete a repository. This action cannot be undone

**Parameters**:
- **confirm** (boolean) **(required)**: Must be true to confirm deletion
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "delete-repo",
  "parameters": {
    "confirm": true,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### clone-repo

**Description**: Get clone URLs for a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "clone-repo",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-collaborators

**Description**: List collaborators on a repository

**Parameters**:
- **limit** (number): Maximum number of collaborators to return
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-collaborators",
  "parameters": {
    "limit": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### add-collaborator

**Description**: Add a collaborator to a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **permission** (string): Permission level to grant
- **repo** (string) **(required)**: Repository name
- **username** (string) **(required)**: GitHub username of the collaborator to add

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "add-collaborator",
  "parameters": {
    "owner": "example_owner",
    "permission": "example_permission",
    "repo": "example_repo",
    "username": "example_username"
  }
}
```

---

#### remove-collaborator

**Description**: Remove a collaborator from a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **username** (string) **(required)**: GitHub username of the collaborator to remove

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "remove-collaborator",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "username": "example_username"
  }
}
```

---

#### list-topics

**Description**: List topics (tags) on a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-topics",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### set-topics

**Description**: Replace all topics on a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **topics** (array) **(required)**: List of topic names to set

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "set-topics",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "topics": []
  }
}
```

---

#### list-languages

**Description**: List programming languages detected in a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-languages",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-issues

**Description**: List issues in a repository with optional filters

**Parameters**:
- **assignee** (string): Filter by assignee username
- **label** (string): Filter by label name
- **limit** (number): Maximum number of issues to return
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **state** (string): Filter by state

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-issues",
  "parameters": {
    "assignee": "example_assignee",
    "label": "example_label",
    "limit": 10,
    "owner": "example_owner",
    "repo": "example_repo",
    "state": "example_state"
  }
}
```

---

#### get-issue

**Description**: Get detailed information about a specific issue

**Parameters**:
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-issue",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### create-issue

**Description**: Create a new issue in a repository

**Parameters**:
- **assignees** (array): Usernames to assign
- **body** (string): Issue body (Markdown supported)
- **labels** (array): Labels to apply
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **title** (string) **(required)**: Issue title

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-issue",
  "parameters": {
    "assignees": [],
    "body": "example_body",
    "labels": [],
    "owner": "example_owner",
    "repo": "example_repo",
    "title": "example_title"
  }
}
```

---

#### close-issue

**Description**: Close an issue

**Parameters**:
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **reason** (string): Reason for closing
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "close-issue",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "reason": "example_reason",
    "repo": "example_repo"
  }
}
```

---

#### reopen-issue

**Description**: Reopen a closed issue

**Parameters**:
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "reopen-issue",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### edit-issue

**Description**: Edit an existing issue's title or body

**Parameters**:
- **body** (string): New issue body
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **title** (string): New issue title

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "edit-issue",
  "parameters": {
    "body": "example_body",
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo",
    "title": "example_title"
  }
}
```

---

#### comment-on-issue

**Description**: Add a comment to an issue

**Parameters**:
- **body** (string) **(required)**: Comment body (Markdown supported)
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "comment-on-issue",
  "parameters": {
    "body": "example_body",
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-issue-comments

**Description**: List comments on an issue

**Parameters**:
- **limit** (number): Maximum number of comments to return
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-issue-comments",
  "parameters": {
    "limit": 10,
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### add-issue-labels

**Description**: Add labels to an issue

**Parameters**:
- **labels** (array) **(required)**: Labels to add
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "add-issue-labels",
  "parameters": {
    "labels": [],
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### remove-issue-labels

**Description**: Remove labels from an issue

**Parameters**:
- **labels** (array) **(required)**: Labels to remove
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "remove-issue-labels",
  "parameters": {
    "labels": [],
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### add-issue-assignees

**Description**: Add assignees to an issue

**Parameters**:
- **assignees** (array) **(required)**: Usernames to assign
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "add-issue-assignees",
  "parameters": {
    "assignees": [],
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### remove-issue-assignees

**Description**: Remove assignees from an issue

**Parameters**:
- **assignees** (array) **(required)**: Usernames to remove
- **number** (number) **(required)**: Issue number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "remove-issue-assignees",
  "parameters": {
    "assignees": [],
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-prs

**Description**: List pull requests in a repository with optional filters

**Parameters**:
- **base** (string): Filter by base branch name
- **limit** (number): Maximum number of pull requests to return
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **state** (string): Filter by state

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-prs",
  "parameters": {
    "base": "example_base",
    "limit": 10,
    "owner": "example_owner",
    "repo": "example_repo",
    "state": "example_state"
  }
}
```

---

#### get-pr

**Description**: Get detailed information about a specific pull request

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-pr",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### create-pr

**Description**: Create a new pull request

**Parameters**:
- **base** (string): The branch to merge into (defaults to repo default branch)
- **body** (string): Pull request body (Markdown supported)
- **draft** (boolean): Create as a draft pull request
- **head** (string) **(required)**: The branch containing the changes (e.g. 'feature-branch' or 'user:feature-branch')
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **title** (string) **(required)**: Pull request title

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-pr",
  "parameters": {
    "base": "example_base",
    "body": "example_body",
    "draft": true,
    "head": "example_head",
    "owner": "example_owner",
    "repo": "example_repo",
    "title": "example_title"
  }
}
```

---

#### close-pr

**Description**: Close a pull request without merging

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "close-pr",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### reopen-pr

**Description**: Reopen a closed pull request

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "reopen-pr",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### merge-pr

**Description**: Merge a pull request

**Parameters**:
- **commit_message** (string): Custom merge commit message
- **delete_branch** (boolean): Delete the head branch after merging
- **method** (string): Merge method to use
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "merge-pr",
  "parameters": {
    "commit_message": "example_commit_message",
    "delete_branch": true,
    "method": "example_method",
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### edit-pr

**Description**: Edit a pull request's title, body, or base branch

**Parameters**:
- **base** (string): New base branch
- **body** (string): New pull request body
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **title** (string): New pull request title

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "edit-pr",
  "parameters": {
    "base": "example_base",
    "body": "example_body",
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo",
    "title": "example_title"
  }
}
```

---

#### comment-on-pr

**Description**: Add a comment to a pull request

**Parameters**:
- **body** (string) **(required)**: Comment body (Markdown supported)
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "comment-on-pr",
  "parameters": {
    "body": "example_body",
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-pr-comments

**Description**: List comments on a pull request

**Parameters**:
- **limit** (number): Maximum number of comments to return
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-pr-comments",
  "parameters": {
    "limit": 10,
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-pr-reviews

**Description**: List reviews on a pull request

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-pr-reviews",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### create-pr-review

**Description**: Submit a review on a pull request

**Parameters**:
- **body** (string): Review comment body
- **event** (string) **(required)**: Review action to perform
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-pr-review",
  "parameters": {
    "body": "example_body",
    "event": "example_event",
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-pr-files

**Description**: List files changed in a pull request

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-pr-files",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### get-pr-diff

**Description**: Get the unified diff for a pull request

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-pr-diff",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### get-pr-checks

**Description**: Get CI/CD check runs and status for a pull request

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-pr-checks",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### request-pr-reviewers

**Description**: Request reviews from specific users on a pull request

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **reviewers** (array) **(required)**: Usernames to request review from

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "request-pr-reviewers",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo",
    "reviewers": []
  }
}
```

---

#### mark-pr-ready

**Description**: Mark a draft pull request as ready for review

**Parameters**:
- **number** (number) **(required)**: Pull request number
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "mark-pr-ready",
  "parameters": {
    "number": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### search-repos

**Description**: Search GitHub repositories by query

**Parameters**:
- **limit** (number): Maximum number of results to return
- **order** (string): Sort order
- **query** (string) **(required)**: Search query (supports GitHub search syntax)
- **sort** (string): Sort field

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-repos",
  "parameters": {
    "limit": 10,
    "order": "example_order",
    "query": "example_query",
    "sort": "example_sort"
  }
}
```

---

#### search-issues

**Description**: Search issues and pull requests across GitHub

**Parameters**:
- **limit** (number): Maximum number of results to return
- **query** (string) **(required)**: Search query (supports GitHub search syntax, e.g. 'is:issue is:open label:bug')
- **sort** (string): Sort field

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-issues",
  "parameters": {
    "limit": 10,
    "query": "example_query",
    "sort": "example_sort"
  }
}
```

---

#### search-code

**Description**: Search code across GitHub repositories

**Parameters**:
- **language** (string): Filter by programming language
- **limit** (number): Maximum number of results to return
- **query** (string) **(required)**: Search query (supports GitHub code search syntax)
- **repo** (string): Restrict search to a specific repo (owner/name format)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-code",
  "parameters": {
    "language": "example_language",
    "limit": 10,
    "query": "example_query",
    "repo": "example_repo"
  }
}
```

---

#### search-commits

**Description**: Search commits across GitHub repositories

**Parameters**:
- **limit** (number): Maximum number of results to return
- **query** (string) **(required)**: Search query (supports GitHub commit search syntax)
- **repo** (string): Restrict search to a specific repo (owner/name format)

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "search-commits",
  "parameters": {
    "limit": 10,
    "query": "example_query",
    "repo": "example_repo"
  }
}
```

---

#### view-file

**Description**: View the contents of a file in a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **path** (string) **(required)**: File path within the repository
- **ref** (string): Git ref (branch, tag, or commit SHA). Defaults to the default branch
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "view-file",
  "parameters": {
    "owner": "example_owner",
    "path": "example_path",
    "ref": "example_ref",
    "repo": "example_repo"
  }
}
```

---

#### list-directory

**Description**: List the contents of a directory in a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **path** (string): Directory path within the repository. Defaults to the root
- **ref** (string): Git ref (branch, tag, or commit SHA). Defaults to the default branch
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-directory",
  "parameters": {
    "owner": "example_owner",
    "path": "example_path",
    "ref": "example_ref",
    "repo": "example_repo"
  }
}
```

---

#### get-readme

**Description**: Get the README file for a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-readme",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-releases

**Description**: List releases for a repository

**Parameters**:
- **limit** (number): Maximum number of releases to return
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-releases",
  "parameters": {
    "limit": 10,
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### get-release

**Description**: Get a specific release by tag name

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **tag** (string) **(required)**: Release tag name (e.g. 'v1.0.0')

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-release",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "tag": "example_tag"
  }
}
```

---

#### create-release

**Description**: Create a new release for a repository

**Parameters**:
- **draft** (boolean): Create as a draft release
- **generate_notes** (boolean): Auto-generate release notes from commits
- **notes** (string): Release notes body (Markdown supported)
- **owner** (string) **(required)**: Repository owner
- **prerelease** (boolean): Mark as a pre-release
- **repo** (string) **(required)**: Repository name
- **tag** (string) **(required)**: Tag name for the release (e.g. 'v1.0.0')
- **target** (string): Target commitish (branch or commit SHA) for the tag
- **title** (string): Release title

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-release",
  "parameters": {
    "draft": true,
    "generate_notes": true,
    "notes": "example_notes",
    "owner": "example_owner",
    "prerelease": true,
    "repo": "example_repo",
    "tag": "example_tag",
    "target": "example_target",
    "title": "example_title"
  }
}
```

---

#### delete-release

**Description**: Delete a release by tag name

**Parameters**:
- **cleanup_tag** (boolean): Also delete the associated git tag
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **tag** (string) **(required)**: Release tag name to delete

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "delete-release",
  "parameters": {
    "cleanup_tag": true,
    "owner": "example_owner",
    "repo": "example_repo",
    "tag": "example_tag"
  }
}
```

---

#### list-release-assets

**Description**: List assets (downloadable files) attached to a release

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **tag** (string) **(required)**: Release tag name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-release-assets",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "tag": "example_tag"
  }
}
```

---

#### get-latest-release

**Description**: Get the latest published release for a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-latest-release",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-gists

**Description**: List gists for the authenticated user or a specific user

**Parameters**:
- **limit** (number): Maximum number of gists to return
- **username** (string): GitHub username. Defaults to the authenticated user

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-gists",
  "parameters": {
    "limit": 10,
    "username": "example_username"
  }
}
```

---

#### get-gist

**Description**: Get a specific gist by ID, including its files and content

**Parameters**:
- **gist_id** (string) **(required)**: The gist ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-gist",
  "parameters": {
    "gist_id": "example_gist_id"
  }
}
```

---

#### create-gist

**Description**: Create a new gist with one or more files

**Parameters**:
- **description** (string): Gist description
- **files** (object) **(required)**: Map of filename to file content, e.g. {"hello.py": {"content": "print('hello')"}}
- **public** (boolean): Whether the gist is public

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "create-gist",
  "parameters": {
    "description": "example_description",
    "files": {},
    "public": true
  }
}
```

---

#### edit-gist

**Description**: Edit an existing gist's description or files

**Parameters**:
- **description** (string): New gist description
- **files** (object): Map of filename to new content. Set content to null to delete a file
- **gist_id** (string) **(required)**: The gist ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "edit-gist",
  "parameters": {
    "description": "example_description",
    "files": {},
    "gist_id": "example_gist_id"
  }
}
```

---

#### delete-gist

**Description**: Permanently delete a gist

**Parameters**:
- **gist_id** (string) **(required)**: The gist ID to delete

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "delete-gist",
  "parameters": {
    "gist_id": "example_gist_id"
  }
}
```

---

#### clone-gist

**Description**: Get the clone URL for a gist

**Parameters**:
- **gist_id** (string) **(required)**: The gist ID to clone

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "clone-gist",
  "parameters": {
    "gist_id": "example_gist_id"
  }
}
```

---

#### list-workflows

**Description**: List GitHub Actions workflows defined in a repository

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-workflows",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo"
  }
}
```

---

#### list-workflow-runs

**Description**: List recent workflow runs for a repository

**Parameters**:
- **branch** (string): Filter by branch name
- **limit** (number): Maximum number of runs to return
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **status** (string): Filter by status
- **workflow_id** (string): Filter by workflow ID or filename (e.g. 'ci.yml')

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-workflow-runs",
  "parameters": {
    "branch": "example_branch",
    "limit": 10,
    "owner": "example_owner",
    "repo": "example_repo",
    "status": "example_status",
    "workflow_id": "example_workflow_id"
  }
}
```

---

#### get-workflow-run

**Description**: Get detailed information about a specific workflow run

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **run_id** (number) **(required)**: Workflow run ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-workflow-run",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "run_id": 10
  }
}
```

---

#### list-run-jobs

**Description**: List jobs for a specific workflow run

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **run_id** (number) **(required)**: Workflow run ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-run-jobs",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "run_id": 10
  }
}
```

---

#### get-run-logs

**Description**: Get the logs URL for a specific workflow run

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **run_id** (number) **(required)**: Workflow run ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "get-run-logs",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "run_id": 10
  }
}
```

---

#### rerun-workflow

**Description**: Re-run an entire workflow run

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **run_id** (number) **(required)**: Workflow run ID to re-run

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "rerun-workflow",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "run_id": 10
  }
}
```

---

#### cancel-workflow-run

**Description**: Cancel a workflow run that is in progress

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **run_id** (number) **(required)**: Workflow run ID to cancel

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "cancel-workflow-run",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "run_id": 10
  }
}
```

---

#### trigger-workflow

**Description**: Manually trigger a workflow dispatch event

**Parameters**:
- **inputs** (object): Input key-value pairs for the workflow_dispatch event
- **owner** (string) **(required)**: Repository owner
- **ref** (string): Git ref (branch or tag) to run the workflow on
- **repo** (string) **(required)**: Repository name
- **workflow_id** (string) **(required)**: Workflow ID or filename (e.g. 'deploy.yml')

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "trigger-workflow",
  "parameters": {
    "inputs": {},
    "owner": "example_owner",
    "ref": "example_ref",
    "repo": "example_repo",
    "workflow_id": "example_workflow_id"
  }
}
```

---

#### view-workflow-yaml

**Description**: View the YAML source of a workflow definition

**Parameters**:
- **owner** (string) **(required)**: Repository owner
- **repo** (string) **(required)**: Repository name
- **workflow_id** (string) **(required)**: Workflow ID or filename (e.g. 'ci.yml')

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "view-workflow-yaml",
  "parameters": {
    "owner": "example_owner",
    "repo": "example_repo",
    "workflow_id": "example_workflow_id"
  }
}
```

---

#### list-notifications

**Description**: List GitHub notifications for the authenticated user

**Parameters**:
- **all** (boolean): Include read notifications
- **limit** (number): Maximum number of notifications to return

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "list-notifications",
  "parameters": {
    "all": true,
    "limit": 10
  }
}
```

---

#### mark-notification-read

**Description**: Mark a specific notification thread as read

**Parameters**:
- **thread_id** (string) **(required)**: Notification thread ID

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "mark-notification-read",
  "parameters": {
    "thread_id": "example_thread_id"
  }
}
```

---

#### mark-all-notifications-read

**Description**: Mark all notifications as read

**Parameters**: *None*

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "mark-all-notifications-read",
  "parameters": {}
}
```

---

#### gh-api

**Description**: Make a raw GitHub REST API request. Use this for any endpoint not covered by the other tools

**Parameters**:
- **body** (object): Request body (for POST/PUT/PATCH)
- **endpoint** (string) **(required)**: API endpoint path (e.g. '/repos/owner/repo/branches')
- **method** (string): HTTP method

**Usage Context**: Available in all environments

**Example**:
```json
{
  "tool": "gh-api",
  "parameters": {
    "body": {},
    "endpoint": "example_endpoint",
    "method": "example_method"
  }
}
```

---


## Tool Usage Guidelines

### Authentication
- All tools require proper authentication setup through the Skills system
- OAuth credentials are managed securely and refreshed automatically
- API keys are stored encrypted in the application keychain

### Rate Limiting
- Tools automatically respect API rate limits of external services
- Intelligent retry logic handles temporary failures with exponential backoff

### Error Handling
- All tools return structured error responses with detailed information
- Network failures trigger automatic retry with configurable attempts

---

**Tool Statistics**
- Total Tools: 152
- Active Skills: 4
- Last Updated: 2026-03-05T15:29:00.373Z

*This file was automatically generated when the app loaded.*
*Tools are discovered from the running V8 skills runtime.*