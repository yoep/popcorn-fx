package com.github.yoep.torrent.adapter;

import java.text.MessageFormat;

/**
 * Exception indicating that the given torrent url is invalid.
 */
public class InvalidTorrentUrlException extends TorrentException {
    private final String url;

    public InvalidTorrentUrlException(String url) {
        super(MessageFormat.format("Torrent url \"{0}\" is invalid", url));
        this.url = url;
    }

    /**
     * Get the invalid url.
     *
     * @return Returns the invalid torrent url.
     */
    public String getUrl() {
        return url;
    }
}
