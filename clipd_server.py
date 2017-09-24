#!/usr/bin/env python3

import datetime
import socket
import socketserver
import string
from threading import Lock

from defs import *

HOST = '0.0.0.0'
PORT = 3399

clipboard = None
rwlock = Lock()

def _get_len(buf):
    '''Returns the LEN header, if present, on a partially filled buffer'''
    for idx,c in enumerate(buf):
        if c in string.ascii_letters:
            return int(buf[:idx])
    return None

def _prep_resp(status, payload):
    '''Returns a composed response in bytes form'''
    resp = str(status) + str(payload)
    resp = str(len(resp)) + resp
    resp = bytes(resp, 'ascii')
    return resp

def _error_out_and_close(sock, exc):
    response = _prep_resp(HDR_ERR, str(exc))
    sock.sendall(response)
    sock.close()

class ThreadedTCPServer(socketserver.ThreadingMixIn, socketserver.TCPServer):
    pass

class ThreadedTCPRequestHandler(socketserver.BaseRequestHandler):
    def handle(self):
        global clipboard
        global rwlock
        found_len = False
        len_hdr = None
        buf = ""

        # read the client message
        while True:
            data = str(self.request.recv(1024), 'ascii')
            if not data:
                break
            buf += data

            # find and store the LEN header if we haven't already
            if not found_len:
                try:
                    len_hdr = _get_len(buf)
                except ValueError as e:
                    _error_out_and_close(self.request, e)
                    return
                if not len_hdr:
                    continue

                # remove the LEN header from our buffer
                buf = buf[len(str(len_hdr)):]

                found_len = True

            # read until we have the whole message
            if len(buf) < len_hdr:
                continue
            else:
                break

        # handle request
        reqtype = None
        if buf.startswith(HDR_PUSH):
            reqtype = HDR_PUSH
            buf = buf[len(HDR_PUSH):]
            rwlock.acquire()
            clipboard = buf
            rwlock.release()
            response = _prep_resp(HDR_OK, '')
        elif buf.startswith(HDR_PULL):
            reqtype = HDR_PULL
            rwlock.acquire()
            if clipboard:
                response = _prep_resp(HDR_OK, clipboard)
            else:
                response = _prep_resp(HDR_OK, '')
            rwlock.release()
        else:
            msg = 'Unsupported request type'
            response = _prep_resp(HDR_ERR, msg)

        # fire off response
        self.request.sendall(response)
        self.request.close()

        print("{} -- Handled {} request from {}:{}".format(
            datetime.datetime.now(), reqtype, self.client_address[0],
            self.client_address[1]))


def main():
    print("Starting clipd server on {}:{}".format(HOST, PORT))
    server = ThreadedTCPServer((HOST, PORT), ThreadedTCPRequestHandler)
    server.allow_reuse_address = True

    try:
        server.serve_forever()
    except KeyboardInterrupt as e:
        server.shutdown()

if __name__ == '__main__':
    main()
