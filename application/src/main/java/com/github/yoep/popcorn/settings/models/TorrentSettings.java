package com.github.yoep.popcorn.settings.models;

import com.github.yoep.popcorn.PopcornTimeApplication;
import lombok.*;

import java.io.File;
import java.util.Objects;

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

    private static final String DEFAULT_TORRENT_DIRECTORY = "torrents";

    /**
     * The directory to save the torrents to.
     */
    @Builder.Default
    private File directory = new File(PopcornTimeApplication.APP_DIR + DEFAULT_TORRENT_DIRECTORY);
    /**
     * The indication if the torrent directory should be cleaned if the application is closed.
     */
    @Builder.Default
    private boolean autoCleaningEnabled = true;
    /**
     * Maximum number of connections.
     */
    @Builder.Default
    private int connectionsLimit = 200;
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
        if (Objects.equals(this.directory, directory))
            return;

        var oldValue = this.directory;
        this.directory = directory;
        changes.firePropertyChange(DIRECTORY_PROPERTY, oldValue, directory);
    }

    public void setAutoCleaningEnabled(boolean autoCleaningEnabled) {
        if (Objects.equals(this.autoCleaningEnabled, autoCleaningEnabled))
            return;

        var oldValue = this.autoCleaningEnabled;
        this.autoCleaningEnabled = autoCleaningEnabled;
        changes.firePropertyChange(AUTO_CLEANING_PROPERTY, oldValue, autoCleaningEnabled);
    }

    public void setConnectionsLimit(int connectionsLimit) {
        if (Objects.equals(this.connectionsLimit, connectionsLimit))
            return;

        var oldValue = this.connectionsLimit;
        this.connectionsLimit = connectionsLimit;
        changes.firePropertyChange(CONNECTIONS_LIMIT_PROPERTY, oldValue, connectionsLimit);
    }

    public void setDownloadRateLimit(int downloadRateLimit) {
        if (Objects.equals(this.downloadRateLimit, downloadRateLimit))
            return;

        var oldValue = this.downloadRateLimit;
        this.downloadRateLimit = downloadRateLimit;
        changes.firePropertyChange(DOWNLOAD_RATE_PROPERTY, oldValue, downloadRateLimit);
    }

    public void setUploadRateLimit(int uploadRateLimit) {
        if (Objects.equals(this.uploadRateLimit, uploadRateLimit))
            return;

        var oldValue = this.uploadRateLimit;
        this.uploadRateLimit = uploadRateLimit;
        changes.firePropertyChange(UPLOAD_RATE_PROPERTY, oldValue, uploadRateLimit);
    }

    //endregion
}
