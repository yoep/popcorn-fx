package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.Data;
import lombok.EqualsAndHashCode;

import java.io.Closeable;
import java.util.Objects;

@Data
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"directory", "cleaningMode", "connectionsLimit", "downloadRateLimit", "uploadRateLimit"})
public class TorrentSettings extends Structure implements Closeable {
    public static class ByValue extends TorrentSettings implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(TorrentSettings settings) {
            Objects.requireNonNull(settings, "settings cannot be null");
            this.directory = settings.directory;
            this.cleaningMode = settings.cleaningMode;
            this.connectionsLimit = settings.connectionsLimit;
            this.downloadRateLimit = settings.downloadRateLimit;
            this.uploadRateLimit = settings.uploadRateLimit;
        }
    }

    public String directory;
    public CleaningMode cleaningMode;
    public int connectionsLimit;
    public int downloadRateLimit;
    public int uploadRateLimit;

    //region Methods

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //endregion
}
