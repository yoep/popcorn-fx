package com.github.yoep.popcorn.backend.torrent.collection;

import com.github.yoep.popcorn.backend.FxLib;
import com.sun.jna.Structure;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.io.IOException;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"magnets", "len"})
public class StoredTorrentSet extends Structure implements Closeable {
    public StoredTorrent.ByReference magnets;
    public int len;

    private List<StoredTorrent> cache;

    public List<StoredTorrent> getMagnets() {
        return cache;
    }

    @Override
    public void read() {
        super.read();
        cache = Optional.ofNullable(magnets)
                .map(e -> (StoredTorrent[]) e.toArray(len))
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    public void close() throws IOException {
        setAutoSynch(false);
        for (StoredTorrent storedTorrent : cache) {
            storedTorrent.close();
        }
        FxLib.INSTANCE.dispose_torrent_collection(this);
    }
}