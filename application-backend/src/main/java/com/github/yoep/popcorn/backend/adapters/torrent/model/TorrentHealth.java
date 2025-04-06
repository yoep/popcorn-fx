package com.github.yoep.popcorn.backend.adapters.torrent.model;

import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentHealthState;
import com.sun.jna.Structure;
import lombok.Getter;

import java.io.Closeable;

@Getter
@Structure.FieldOrder({"state", "ratio", "seeds", "leechers"})
public class TorrentHealth extends Structure implements Closeable {
    public static class ByValue extends TorrentHealth implements Structure.ByValue {
    }
    public static class ByReference extends TorrentHealth implements Structure.ByReference {
    }

    public TorrentHealthState state;
    public float ratio;
    public int seeds;
    public int leechers;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
