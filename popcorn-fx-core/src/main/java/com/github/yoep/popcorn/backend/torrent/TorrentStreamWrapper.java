package com.github.yoep.popcorn.backend.torrent;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"url", "ptr"})
public class TorrentStreamWrapper extends Structure implements Closeable {
    public String url;
    public Pointer ptr;
    
    @Override
    public void close() {
        setAutoSynch(false);
    }
}
