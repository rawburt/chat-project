# Internet Relay Chat Protocol


# 1. Introduction

This document describes a protocol in which connected clients may communicate with each other by passing messages through a centralized server.

Users can join rooms and send messages to the room which acts as a message stream to all users who have joined the room. Users can send messages directly to other users.


# 2. Conventions used in this document

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119 [RFC2119].  

In this document, these words will appear with that interpretation only when in ALL CAPS. Lower case uses of these words are not to be interpreted as carrying significance described in RFC 2119.


# 3. Basic Information

Communication in this protocol takes place over TCP/IP. Servers listen to connections on port 5456. Clients can connect to this port and send messages to the server at any time. The server MAY asynchronously send messages back to the client. The client or server MAY terminate connection at any time.


# 4. Message Infrastructure


## 4.1 Generic Message Format

The following message format definition loosely follows a mix of Backus–Naur form and regular expressions. An expanded definition for each element follows in section 4.1.1.

&lt;message> = [ &lt;prefix> “ “] &lt;command> [“ “ &lt;params> ] [“ “ &lt;payload> ] &lt;newline>

&lt;command> = [A-Z]+

&lt;prefix> = &lt;user> | &lt;room> “ “ &lt;user>

&lt;params> = &lt;param> | &lt;params> “ “ &lt;param>

&lt;param> = &lt;user> | &lt;room>

&lt;user> = @ &lt;ident>

&lt;room> = # &lt;ident>

&lt;ident> = [a-zA-Z0-9\_\-]{2,19}

&lt;payload> = .+

&lt;newline> = \n


### 4.1.1 Field Semantics



* &lt;message>
    * A message MUST have a &lt;command> and it MAY have a &lt;prefix>, it MAY have &lt;params>, and it MAY have a &lt;payload>. A message MUST end with a &lt;newline>. A &lt;message> MUST NOT exceed 1024 bytes.
* &lt;command>
    * A command MUST be any upper-case character sequence of length 1 or more. The allowed characters are A through Z (or, the ASCII character codes 65 through 90).
* &lt;prefix>
    * A prefix MUST have a &lt;user> or it MUST have a &lt;room>, followed by a space (ASCII code 32), and then followed by a &lt;user>.
* &lt;params>
    * Params MUST be a space (ASCII code 32) delimited list of &lt;param>.
* &lt;param>
    * A param MUST be a &lt;user> or a &lt;room>.
* &lt;user>
    * A user MUST begin with @ (ASCII code 64) followed by an &lt;ident>.
* &lt;room>
    * A room MUST begin with # (ASCII code 35) followed by an &lt;ident>.
* &lt;ident>
    * An ident MUST be a sequence of characters of minimum length 2 and maximum length 19. The valid characters of an ident are the upper-case and lower-case characters A through Z (ASCII codes 65 through 90 and 97 through 122), the numbers 0 through 9 (ASCII codes 48 through 57), and the characters “_” (ASCII code 95) or “-” (ASCII code 45).
* &lt;payload>
    * A payload MUST be any ASCII character other than “\n” (LF, or ASCII code 10).
* &lt;newline>
    * A newline is the character “\n” (LF, or ASCII code 10).


# 5. Client Messages


## 5.1 Registration


### 5.1.1 NAME

Usage: NAME &lt;user>

The NAME message MUST be used to register a user name to the newly connected client. After registration, the client MUST receive the REGISTERED message from the server. The NAME message MAY be used after registration to change the user name of the connected client. If there is an error with registration, such as duplicate user name or bad formatting of a user name, the server MUST reply with an ERROR message.

Example: NAME @robert


## 5.2 Room Operations


### 5.2.1 ROOMS

Usage: ROOMS

The ROOMS message MAY be used to request a list of created rooms from the server. If rooms exist, the server MUST reply with a list of rooms using the ROOM message. If rooms do not exist, the server MAY reply with an ERROR message.


### 5.2.2 JOIN

Usage: JOIN &lt;room>

The JOIN message MAY be used to join a room. If the room does not exist, the server MUST create it. If the room already exists, the client is added to the room and a JOINED message MUST be sent to the existing subscribers of the room. If there is an error with joining the room, such as bad formatting of the room name, the server MUST reply with an ERROR message.

Example: JOIN #sports


### 5.2.3 LEAVE

Usage: LEAVE &lt;room>

The LEAVE message MAY be used to join a previously joined room. If leaving the room is successful, the server MUST send a LEFT message to the remaining subscribers of the room. If the user leaving causes the room to become empty (no subscribers), the server MUST delete the room. If the room does not exist, or the room was not previously joined, the server MUST reply with an ERROR message. If there is an error leaving the room, such as bad formatting of the room name, the server MUST reply with an ERROR message.


### 5.2.4 USERS

Usage: USERS &lt;room>

The USERS message MAY be used to request a list of users who have joined the room. If the room exists, the server MUST reply with a list of users using the USER message. If the room does not exist, the server MUST reply with an ERROR message. If there is an error listing the users of the room, such as bad formatting of the room name, the server MUST reply with an ERROR message.

Example: USERS #sports


## 5.3 Private Messaging


### 5.3.1 SAY

Usage: SAY &lt;room> &lt;payload>

Usage: SAY &lt;user> &lt;payload>

The SAY message MAY be used to send a message to a room or a user. If the room or user does not exist, the server MUST reply with an ERROR message. If there is an error sending the message to the users or room, such as bad formatting of the name, the server MUST reply with an ERROR message.

Example: SAY #sports hello everybody! \
Example: SAY @robert I hear you like sports. Is that true?


## 5.4 Connection


### 5.4.1 PONG

Usage: PONG

The client MUST send a PONG message to the server if the server sends the client a PING message. The server MAY disconnect the client if a PONG message is not received within a server specified time threshold after sending a PING message to the client.


### 5.4.3 QUIT

Usage: QUIT

The QUIT message MAY be used to request that the server disconnect the client. The server MUST disconnect the client. The QUIT message MAY have extra &lt;params> or &lt;payload> sent with it and the server MUST ignore them. The server MAY send a message to the client before the connection is severed.


# 6. Server Messages


## 6.1 Registration


### 6.1.1 CONNECTED

Usage: CONNECTED

After a client successfully connects to a server, the server MUST send a CONNECTED message. When a client receives a CONNECTED message, the client has entered the registration phase of their interaction with the server. A client MUST send a NAME message to complete the registration process before any other message is allowed (other than QUIT).


### 6.1.1 REGISTERED

Usage: REGISTERED

After successful client registration, the server MUST send the REGISTERED command to the client. After the registration phase is completed, a client MAY send any acceptable message to the server.


## 6.2 Room Operations


### 6.2.1 ROOM

Usage: ROOM &lt;room>

In response to a ROOMS message from the client, the server MUST respond with a ROOM message for each room that exists.

Example: ROOM #general


### 6.2.2 JOINED

Usage: &lt;room> &lt;user> JOINED

After a client successfully joins a room, the server MUST send a JOINED message to each user that is subscribed to the room. 

Example: #sports @robert JOINED


### 6.2.3 LEFT

Usage: &lt;room> &lt;user> LEFT

After a client successfully leaves a room, the server MUST send a LEFT message to each user subscribed to the room.

Example: #general @kelsey LEFT


### 6.2.4 USER

Usage: USER &lt;user>

In response to a USERS message from the client, the server MUST respond with a USER message for each user that is subscribed to the room listed in the USERS message.

Example: USER @lilly


## 6.3 Private Messaging


### 6.3.1 SAID

Usage: &lt;room> &lt;user> SAID &lt;payload> 

Usage: &lt;user> SAID &lt;payload> 

After a successful SAY message, the recipient of the SAY message MUST receive a corresponding SAID message. If the client sends a private message to a room, each user subscribed to the room MUST receive a corresponding SAID command. If the client sends a private message to a user, the user MUST receive a corresponding SAID command.

Example: #sports @robert SAID good game

Example: @kelsey SAID are you home?


## 6.4 Connection


### 6.4.1 PING

Usage: PING

A server MAY send a PING to a client at any time. If the client does not respond with a PONG in some server defined timeline, the server MAY disconnect the client due to inactivity.


## 6.5 Errors


### 6.5.1 ERROR

Usage: ERROR &lt;payload>

Any error detected on the server side MAY trigger the server to send an ERROR message to the client. If no information is known about the error that occurred, the server MAY omit the &lt;payload>.

Example: ERROR bad format of room name

Example: ERROR bad format of user name

Example: ERROR user already exists @steve

Example: ERROR room unknown #karate


# 7. Security Considerations

All interactions between the server and client are visible if intercepted while transmitting over TCP/IP. This protocol does not specify any encryption or security features. Any such implementations are left to the implementor.


# 8. Conclusion

This protocol defines a framework for passing messages between multiple clients through a centralized server.


# 9. Normative References

[RFC2119] 	Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997.


# 10. Acknowledgements

This document was prepared using Google Doc.
