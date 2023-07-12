package com.github.yoep.popcorn.backend.player.model;

import com.github.yoep.popcorn.backend.adapters.player.PlayStreamRequest;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Objects;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
public class StreamPlayRequest extends SimplePlayRequest implements PlayStreamRequest {
    private final TorrentStream torrentStream;

    @Builder(builderMethodName = "streamBuilder")
    public StreamPlayRequest(String url, String title, String thumb, Long autoResumeTimestamp, TorrentStream torrentStream, boolean isSubtitlesEnabled) {
        super(url, title, thumb, autoResumeTimestamp, isSubtitlesEnabled);
        Objects.requireNonNull(torrentStream, "torrentStream cannot be null");
        this.torrentStream = torrentStream;
    }
}
