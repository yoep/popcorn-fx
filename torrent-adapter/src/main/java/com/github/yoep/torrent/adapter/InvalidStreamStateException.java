package com.github.yoep.torrent.adapter;

import com.github.yoep.torrent.adapter.state.TorrentStreamState;
import lombok.Getter;

import java.text.MessageFormat;

@Getter
public class InvalidStreamStateException extends StreamException {
    private final TorrentStreamState state;

    public InvalidStreamStateException(TorrentStreamState state) {
        super(MessageFormat.format("Torrent stream is in an invalid state \"{0}\"", state));
        this.state = state;
    }
}
