# GhostWire - Multi-Channel Usage Guide

## 🎯 What's New: Channel System

GhostWire now supports **multiple channels** including:

- **Global Channel** (`# global`) - Everyone sees messages
- **Direct Messages** (`@ username`) - Private 1-on-1 conversations

---

## 🎮 Keyboard Controls

### Channel Navigation

| Key        | Action                     |
| ---------- | -------------------------- |
| `h` or `←` | Select previous channel    |
| `l` or `→` | Select next channel        |
| `Tab`      | Switch to selected channel |
| `#`        | Jump to global channel     |

### Direct Messages

| Key           | Action                       |
| ------------- | ---------------------------- |
| `d`           | Create DM with selected user |
| `J` (Shift+j) | Select next user             |
| `K` (Shift+k) | Select previous user         |

### Chat Controls (Unchanged)

| Key            | Action                |
| -------------- | --------------------- |
| `i` or `Enter` | Enter edit mode       |
| `Esc`          | Exit edit mode / Quit |
| `q`            | Quit application      |
| `j` or `↓`     | Scroll down           |
| `k` or `↑`     | Scroll up             |
| `G`            | Scroll to bottom      |

---

## 📖 How to Use

### Starting a Direct Message

1. **Select a user:**

   - Press `J` or `K` to navigate the user list
   - The selected user will be highlighted

2. **Create DM:**

   - Press `d` to create a DM channel
   - A new channel will appear: `@ username`
   - You'll automatically switch to that channel

3. **Send messages:**
   - Press `i` to enter edit mode
   - Type your message
   - Press `Enter` to send
   - Only you and the other user will see these messages!

### Switching Between Channels

**Method 1: Direct Selection**

- Press `h` or `l` to highlight different channels
- Press `Tab` to switch to the highlighted channel

**Method 2: Quick Jump**

- Press `#` to instantly jump to global channel

### Understanding the UI

```
┌─────────────────┐ ┌──────────────────────────┐ ┌─────────────┐
│ Channels (2)    │ │ # global ● CONNECTED     │ │  Telemetry  │
│                 │ │                          │ │             │
│ # global        │ │ [12:34:56] alice: hi     │ │ ┌─────────┐ │
│ @ bob (3)       │ │ [12:35:01] bob: hello    │ │ │ Uptime  │ │
│                 │ │                          │ │ └─────────┘ │
└─────────────────┘ └──────────────────────────┘ └─────────────┘
     ▲                        ▲                         ▲
  Channels              Active Channel              Stats
```

**Channel List Features:**

- **Highlighted** = Active channel (black on green)
- **Yellow + Bold** = Unread messages
- **Number in ()** = Unread count
- **# prefix** = Global channel
- **@ prefix** = Direct message

---

## 🧪 Testing the Channel System

### Scenario 1: Global Chat

1. Start server: `cargo run --bin ghostwire-local`
2. Start Alice: `cargo run -p ghostwire-client alice ws://localhost:8080/ws`
3. Start Bob: `cargo run -p ghostwire-client bob ws://localhost:8080/ws`
4. Both users send messages in `# global`
5. ✅ Both see all messages

### Scenario 2: Direct Messages

1. Alice presses `J` to select Bob
2. Alice presses `d` to create DM
3. Alice sees new channel: `@ bob`
4. Alice types message and sends
5. ✅ Only Bob sees the message
6. ✅ Charlie (if connected) does NOT see it

### Scenario 3: Channel Switching

1. Alice is in `@ bob` channel
2. Alice presses `#` to jump to global
3. Alice presses `h` to select `@ bob`
4. Alice presses `Tab` to switch back
5. ✅ Seamless channel switching

### Scenario 4: Unread Counts

1. Alice is in `# global`
2. Bob sends DM to Alice
3. Alice sees `@ bob (1)` in yellow
4. Alice presses `h` then `Tab` to switch
5. ✅ Unread count clears

---

## 🔧 Technical Details

### Channel ID Format

- **Global:** `"global"`
- **DM:** `"dm:alice:bob"` (alphabetically sorted)
- **Group:** `"group:name"` (not yet implemented)

### Message Routing

Messages are routed based on the `channel` field in the protocol:

```json
{
  "type": "MSG",
  "payload": "Hello!",
  "channel": "dm:alice:bob",
  "meta": {
    "sender": "alice",
    "timestamp": 1733234567
  }
}
```

### Server Behavior

The server currently broadcasts all messages to all clients. **Channel filtering happens client-side.**

> [!WARNING] > **Privacy Note:** In this minimal version, the server sees all messages. For true privacy, the server needs to implement channel-based routing (coming in future updates).

---

## 🚀 Next Steps

### Planned Features

- [ ] **Server-side channel routing** - Server only sends messages to channel members
- [ ] **Group channels** - Multi-user private channels
- [ ] **Channel persistence** - Channels survive disconnects
- [ ] **Channel invitations** - Invite users to channels
- [ ] **Channel discovery** - List available channels

### Current Limitations

1. **No server-side filtering** - All messages broadcast to all clients
2. **No group channels** - Only global + DMs
3. **Session-based** - Channels reset on disconnect
4. **No channel history** - Messages only in memory

---

## 🐛 Troubleshooting

### "No users to DM"

- Wait for other users to connect
- Users appear in the user list when they join

### "DM not working"

- Make sure you selected a user first (`J`/`K`)
- Press `d` to create the DM
- Check that you're in the DM channel (should show `@ username`)

### "Can't switch channels"

- Use `h`/`l` to select, then `Tab` to switch
- Or press `#` to jump to global

### "Messages in wrong channel"

- Check the channel name in the title bar
- Make sure you switched to the correct channel

---

## 📝 Quick Reference Card

```
┌─────────────────────────────────────────┐
│         GhostWire Channels              │
├─────────────────────────────────────────┤
│ CHANNEL NAVIGATION                      │
│  h/l  - Select channel                  │
│  Tab  - Switch to selected              │
│  #    - Jump to global                  │
│                                         │
│ DIRECT MESSAGES                         │
│  d    - Create DM with selected user    │
│  J/K  - Select user                     │
│                                         │
│ CHAT                                    │
│  i    - Start typing                    │
│  Esc  - Stop typing                     │
│  q    - Quit                            │
└─────────────────────────────────────────┘
```

---

**Enjoy your multi-channel chat! 👻**
