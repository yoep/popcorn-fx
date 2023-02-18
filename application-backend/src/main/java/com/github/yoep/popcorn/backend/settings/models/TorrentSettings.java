package com.github.yoep.popcorn.backend.settings.models;

import lombok.*;

import java.io.File;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class TorrentSettings extends AbstractSettings {
    public static final String DIRECTORY_PROPERTY = "directory";
    public static final String AUTO_CLEANING_PROPERTY = "autoCleaningEnabled";
    public static final String CONNECTIONS_LIMIT_PROPERTY = "connectionsLimit";
    public static final String DOWNLOAD_RATE_PROPERTY = "downloadRateLimit";
    public static final String UPLOAD_RATE_PROPERTY = "uploadRateLimit";
    public static final String DEFAULT_TORRENT_DIRECTORY = "torrents";

    /**
     * The directory to save the torrents to.
     */
    private File directory;
    /**
     * The indication if the torrent directory should be cleaned if the application is closed.
     */
    @Builder.Default
    private boolean autoCleaningEnabled = true;
    /**
     * Maximum number of connections.
     */
    @Builder.Default
    private int connectionsLimit = 300;
    /**
     * The download rate limit.
     */
    @Builder.Default
    private int downloadRateLimit = 0;
    /**
     * The upload rate limit.
     */
    @Builder.Default
    private int uploadRateLimit = 0;

    //region Setters

    public void setDirectory(File directory) {
        this.directory = updateProperty(this.directory, directory, DIRECTORY_PROPERTY);
    }

    public void setAutoCleaningEnabled(boolean autoCleaningEnabled) {
        this.autoCleaningEnabled = updateProperty(this.autoCleaningEnabled, autoCleaningEnabled, AUTO_CLEANING_PROPERTY);
    }

    public void setConnectionsLimit(int connectionsLimit) {
        this.connectionsLimit = updateProperty(this.connectionsLimit, connectionsLimit, CONNECTIONS_LIMIT_PROPERTY);
    }

    public void setDownloadRateLimit(int downloadRateLimit) {
        this.downloadRateLimit = updateProperty(this.downloadRateLimit, downloadRateLimit, DOWNLOAD_RATE_PROPERTY);
    }

    public void setUploadRateLimit(int uploadRateLimit) {
        this.uploadRateLimit = updateProperty(this.uploadRateLimit, uploadRateLimit, UPLOAD_RATE_PROPERTY);
    }

    //endregion
}
