package com.github.yoep.popcorn.torrent;

public class StreamStatus {
    public final float progress;
    public final int bufferProgress;
    public final int seeds;
    public final float downloadSpeed;

    protected StreamStatus(float progress, int bufferProgress, int seeds, int downloadSpeed) {
        this.progress = progress;
        this.bufferProgress = bufferProgress;
        this.seeds = seeds;
        this.downloadSpeed = downloadSpeed;
    }
}
