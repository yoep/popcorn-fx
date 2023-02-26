package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.Objects;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"directory", "autoCleaningEnabled", "connectionsLimit", "downloadRateLimit", "uploadRateLimit"})
public class TorrentSettings extends Structure implements Closeable {
    public static class ByValue extends TorrentSettings implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(TorrentSettings settings) {
            Objects.requireNonNull(settings, "settings cannot be null");
            this.directory = settings.directory;
            this.autoCleaningEnabled = settings.autoCleaningEnabled;
            this.connectionsLimit = settings.connectionsLimit;
            this.downloadRateLimit = settings.downloadRateLimit;
            this.uploadRateLimit = settings.uploadRateLimit;
        }
    }

    public String directory;
    public byte autoCleaningEnabled;
    public int connectionsLimit;
    public int downloadRateLimit;
    public int uploadRateLimit;

    //region Methods

    public boolean isAutoCleaningEnabled() {
        return autoCleaningEnabled == 1;
    }

    public void setAutoCleaningEnabled(boolean autoCleaningEnabled) {
        this.autoCleaningEnabled = (byte) (autoCleaningEnabled ? 1 : 0);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //endregion
}
