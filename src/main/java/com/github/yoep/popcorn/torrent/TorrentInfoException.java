package com.github.yoep.popcorn.torrent;

public class TorrentInfoException extends TorrentException {
    public TorrentInfoException(Throwable cause) {
        super("No torrent info could be found or read", cause);
    }
}
