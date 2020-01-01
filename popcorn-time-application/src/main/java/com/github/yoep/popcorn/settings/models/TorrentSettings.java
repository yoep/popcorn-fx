package com.github.yoep.popcorn.settings.models;

import com.github.yoep.popcorn.PopcornTimeApplication;
import lombok.*;

import java.io.File;
import java.util.Objects;

@EqualsAndHashCode(callSuper = true)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class TorrentSettings extends AbstractSettings {
    public static final String DIRECTORY_PROPERTY = "directory";
    public static final String AUTO_CLEANING_PROPERTY = "autoCleaningEnabled";

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
}
