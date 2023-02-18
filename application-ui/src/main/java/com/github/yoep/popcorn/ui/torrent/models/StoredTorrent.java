package com.github.yoep.popcorn.ui.torrent.models;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.io.IOException;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"name", "magnetUri"})
public class StoredTorrent extends Structure implements Closeable {
    public String name;
    public String magnetUri;

    @Override
    public void close() throws IOException {
        setAutoSynch(false);
    }
}
