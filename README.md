# clipd

This is a simple distributed clipboard.

Imagine this scenario: you use a a couple of different machines to do your
work. You need a way to share clipboard contents between all these machines.
`clipd` solves this problem for you. Imagine `xclip` but with a server and
clients.

## Installation

1. Stand up `clipd_server.py` on a remote (or local) host somewhere
2. Copy `config.ini` into `~/.config/clipd/config.ini` on all your client machines. Make sure you update the config.
3. Symlink `clipd` into `/usr/local/bin`

## Protocol

`clipd` communicates over TCP using ASCII encoded data (please don't hate me,
utf-8 people). The wire format is as follows:

    +---+------+-------+
    |LEN|HEADER|PAYLOAD|
    +---+------+-------+

where `LEN` is the total number of characters in the message (`LEN` excluded).

### Client -> Server

There are two types of requests:

1. PUSH

    A push is a request to add something to the clipboard. The header is `PUSH`
    and the payload is the future contents of the clipboard.

2. PULL

    A pull is a request for the current contents of the clipboard. The header is
    `PULL` and there is no payload.

### Server -> Client

There are two types of responses:

1. OK

    If the request succeeded, then an OK is returned to the client. The payload
    is either empty or the contents of the clipboard, depending on the request.

2. ERR

    If the request failed, then ERR is returned. The payload is the error
    message, if any.

## TODO

[ ] Set up private key authentication for security
