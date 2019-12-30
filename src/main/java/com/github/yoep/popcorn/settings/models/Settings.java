package com.github.yoep.popcorn.settings.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class Settings {
    /**
     * The torrent settings of the application.
     */
    @Builder.Default
    private TorrentSettings torrentSettings = TorrentSettings.builder().build();
    /**
     * The subtitle settings of the application.
     */
    @Builder.Default
    private SubtitleSettings subtitleSettings = SubtitleSettings.builder().build();
    /**
     * The ui settings of the application.
     */
    @Builder.Default
    private UISettings uiSettings = UISettings.builder().build();
    /**
     * The logging settings of the application.
     */
    @Builder.Default
    private LoggingSettings loggingSettings = LoggingSettings.builder().build();

    //region Getters

    public TorrentSettings getTorrentSettings() {
        if (torrentSettings == null)
            torrentSettings = TorrentSettings.builder().build();

        return torrentSettings;
    }

    public SubtitleSettings getSubtitleSettings() {
        if (subtitleSettings == null)
            subtitleSettings = SubtitleSettings.builder().build();

        return subtitleSettings;
    }

    public UISettings getUiSettings() {
        if (uiSettings == null)
            uiSettings = UISettings.builder().build();

        return uiSettings;
    }

    public LoggingSettings getLoggingSettings() {
        if (loggingSettings == null)
            loggingSettings = LoggingSettings.builder().build();

        return loggingSettings;
    }

    //endregion
}
