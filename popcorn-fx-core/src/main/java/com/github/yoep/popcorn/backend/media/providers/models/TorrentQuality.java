package com.github.yoep.popcorn.backend.media.providers.models;

import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;

@Getter
@ToString
@Structure.FieldOrder({"quality", "info"})
public class TorrentQuality extends Structure implements Closeable {
    public static class ByReference extends TorrentQuality implements Structure.ByReference {
    }

    public String quality;
    public MediaTorrentInfo info;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
