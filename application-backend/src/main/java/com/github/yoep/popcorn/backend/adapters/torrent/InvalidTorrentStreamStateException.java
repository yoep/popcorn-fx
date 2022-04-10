package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import lombok.Getter;

import java.text.MessageFormat;

@Getter
public class InvalidTorrentStreamStateException extends TorrentStreamException {
    private final TorrentStreamState state;

    public InvalidTorrentStreamStateException(TorrentStreamState state) {
        super(MessageFormat.format("Torrent stream is in an invalid state \"{0}\"", state));
        this.state = state;
    }
}
