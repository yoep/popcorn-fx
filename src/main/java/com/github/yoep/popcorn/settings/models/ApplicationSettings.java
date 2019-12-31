package com.github.yoep.popcorn.settings.models;

import lombok.*;

import java.util.Objects;

@EqualsAndHashCode(callSuper = true)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class ApplicationSettings extends AbstractSettings {
    public static final String TORRENT_PROPERTY = "torrentSettings";
    public static final String SUBTITLE_PROPERTY = "subtitleSettings";
    public static final String UI_PROPERTY = "uiSettings";
    public static final String LOGGING_PROPERTY = "loggingSettings";

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

    //region Setters

    public void setTorrentSettings(TorrentSettings torrentSettings) {
        if (Objects.equals(this.torrentSettings, torrentSettings))
            return;

        var oldValue = this.torrentSettings;
        this.torrentSettings = torrentSettings;
        changes.firePropertyChange(TORRENT_PROPERTY, oldValue, torrentSettings);
    }

    public void setSubtitleSettings(SubtitleSettings subtitleSettings) {
        if (Objects.equals(this.subtitleSettings, subtitleSettings))
            return;

        var oldValue = this.subtitleSettings;
        this.subtitleSettings = subtitleSettings;
        changes.firePropertyChange(SUBTITLE_PROPERTY, oldValue, this.subtitleSettings);
    }

    public void setUiSettings(UISettings uiSettings) {
        if (Objects.equals(this.uiSettings, uiSettings))
            return;

        var oldValue = this.uiSettings;
        this.uiSettings = uiSettings;
        changes.firePropertyChange(UI_PROPERTY, oldValue, this.uiSettings);
    }

    public void setLoggingSettings(LoggingSettings loggingSettings) {
        if (Objects.equals(this.loggingSettings, loggingSettings))
            return;

        var oldValue = this.loggingSettings;
        this.loggingSettings = loggingSettings;
        changes.firePropertyChange(LOGGING_PROPERTY, oldValue, this.loggingSettings);
    }

    //endregion
}
