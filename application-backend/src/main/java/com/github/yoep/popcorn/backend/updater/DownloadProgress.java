package com.github.yoep.popcorn.backend.updater;

import com.sun.jna.Structure;
import lombok.Getter;

import java.io.Closeable;

@Getter
@Structure.FieldOrder({"totalSize", "downloaded"})
public class DownloadProgress extends Structure implements Closeable {
    public long totalSize;
    public long downloaded;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
