package com.github.yoep.popcorn.backend.loader;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;

@Getter
@ToString
@Structure.FieldOrder({"progress", "seeds", "peers", "downloadSpeed", "uploadSpeed", "downloaded", "total_size"})
public class LoadingProgress extends Structure implements Closeable, DownloadStatus {
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
    public float progress() {
        return progress;
    }

    @Override
    public int seeds() {
        return seeds;
    }

    @Override
    public int peers() {
        return peers;
    }

    @Override
    public int downloadSpeed() {
        return downloadSpeed;
    }

    @Override
    public int uploadSpeed() {
        return uploadSpeed;
    }

    @Override
    public long downloaded() {
        return downloaded;
    }

    @Override
    public long totalSize() {
        return total_size;
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
