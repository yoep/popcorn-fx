package com.github.yoep.popcorn.backend.loader;

import com.sun.jna.Structure;
import lombok.Builder;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;

@Getter
@ToString
@Structure.FieldOrder({"progress", "seeds", "peers", "downloadSpeed", "uploadSpeed", "downloaded", "total_size"})
public class LoadingProgress extends Structure implements Closeable {
    public static class ByValue extends LoadingProgress implements Structure.ByValue {
        public ByValue() {
            super();
        }

        @Builder
        public ByValue(Float progress, int seeds, int peers, int downloadSpeed, int uploadSpeed, long downloaded, long total_size) {
            super(progress, seeds, peers, downloadSpeed, uploadSpeed, downloaded, total_size);
        }
    }

    public Float progress;
    public int seeds;
    public int peers;
    public int downloadSpeed;
    public int uploadSpeed;
    public long downloaded;
    public long total_size;

    public LoadingProgress() {
    }

    public LoadingProgress(Float progress, int seeds, int peers, int downloadSpeed, int uploadSpeed, long downloaded, long total_size) {
        this.progress = progress;
        this.seeds = seeds;
        this.peers = peers;
        this.downloadSpeed = downloadSpeed;
        this.uploadSpeed = uploadSpeed;
        this.downloaded = downloaded;
        this.total_size = total_size;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
