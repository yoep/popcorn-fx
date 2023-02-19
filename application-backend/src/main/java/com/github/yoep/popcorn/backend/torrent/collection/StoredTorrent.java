package com.github.yoep.popcorn.backend.torrent.collection;

import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"name", "magnetUri"})
public class StoredTorrent extends Structure implements Closeable {
    public static class ByReference extends StoredTorrent implements Structure.ByReference {
    }

    public String name;
    public String magnetUri;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
