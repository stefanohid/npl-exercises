# Network Programming Laboratory

## Exercise: UDP Command Chatroom

Implement a simple centralized chatroom in which multiple clients communicate
through a UDP server. The application must use an arbitrary UDP port,
conventionally set to `8080`.

Communication uses a text-based command protocol. Fields are separated by the
`|` character.

## Message Protocol

Client-to-server messages:

```text
JOIN|<username>
QUIT|<username>
SEND|<username>|<message>
PRIV|<target_username>|<message>
```

Server-to-client messages:

```text
SEND|<message>
ERROR|<error_description>
```

## Chatroom Server

The server binds a UDP socket to `0.0.0.0:8080` and maintains a table
associating each username with the corresponding client socket address.

For each received message, the server must:

- `JOIN`: register the client if the username is valid and available.
- `QUIT`: remove the client and notify the remaining users.
- `SEND`: broadcast the message to every registered client except the sender.
- `PRIV`: forward the message only to the specified recipient.
- Send an error when a username is already taken or a private-message recipient
  does not exist.
- Announce users joining and leaving the chatroom.
- Ignore or reject malformed and unknown commands without terminating.

## Chatroom Client

The client binds a UDP socket to an arbitrary local port and asks the user for a
username. Empty usernames and usernames containing `|` must be rejected.

After joining, the client must use two execution threads:

- A background thread waits for UDP messages from the server and immediately
  prints them.
- The main thread reads commands and messages from standard input.

The user interface must support:

```text
/quit
/msg <username> <message>
<message>
```

Plain text is sent to every user, while `/msg` sends a private message.

## Tasks

1. Write a Rust program implementing the UDP chatroom server.
2. Write a Rust program implementing the multithreaded UDP client.
3. Test the application using multiple simultaneously running clients.
4. Ensure that invalid input, duplicate usernames, unknown users, and network
   errors are handled without crashing the application.
