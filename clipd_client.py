#!/usr/bin/env python3

import argparse
import logging
import socket
import string
import sys

from defs import *

HOST = 'localhost'
PORT = 3399 

class ClipdException(Exception):
    pass

def _sock_send_recv(msg):
    '''Sends a message to the server and returns the response'''
    resp = ""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((HOST, PORT))
        s.sendall(msg)
        while True:
            data = str(s.recv(1024), 'ascii')
            if not data:
                break
            resp += data
    return resp

def _parse_resp(resp):
    '''Returns a (hdr, payload) tuple from a raw response'''
    for idx,c in enumerate(resp):
        if c in string.ascii_letters:
            msg_len = int(resp[:idx])
            resp = resp[idx:]
            break

    h = None
    if resp.startswith(HDR_OK):
        h = HDR_OK
    elif resp.startswith(HDR_ERR):
        h = HDR_ERR
    else:
        raise ClipdException('Unknown response code')

    resp = resp[len(h):]
    msg_len -= len(h)
    if msg_len != len(resp):
        raise ClipdException('Message payload corrupted -- lengths do not match')
    return (h, resp)

def pull():
    '''Returns clipboard contents from server'''
    req = str(len(HDR_PULL)) + HDR_PULL
    req = bytes(req, 'ascii')
    resp = _sock_send_recv(req)

    hdr, payload = _parse_resp(resp)
    if (hdr != HDR_OK):
        raise ClipdException(payload)
    return payload

def push(txt):
    '''Pushes new clipboard contents to server'''
    req = str(len(HDR_PUSH) + len(txt)) + HDR_PUSH + txt
    req = bytes(req, 'ascii')
    resp = _sock_send_recv(req)

    hdr, payload = _parse_resp(resp)
    if (hdr != HDR_OK):
        raise ClipdException(payload)

def main(args):
    try:
        if args.pull:
            sys.stdout.write(pull())
            sys.stdout.flush()
        else:
            push(sys.stdin.read())
    except ClipdException as e:
        sys.stderr.write(str(e) + '\n')
        sys.exit(1)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('-p', '--pull', action='store_true',
            help='pull clipboard contents from server')
    main(parser.parse_args())
