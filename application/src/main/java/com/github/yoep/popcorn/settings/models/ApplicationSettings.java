package com.github.yoep.popcorn.settings.models;

import lombok.*;

import java.util.Objects;

@EqualsAndHashCode(callSuper = false)
@ToString
@Builder
@NoArgsConstructor
@AllArgsConstructor
@SuppressWarnings("unused")
public class ApplicationSettings extends AbstractSettings {
    public static final String TORRENT_PROPERTY = "torrentSettings";
    public static final String SUBTITLE_PROPERTY = "subtitleSettings";
    public static final String UI_PROPERTY = "uiSettings";
    public static final String TRAKT_PROPERTY = "traktSettings";
    public static final String LOGGING_PROPERTY = "loggingSettings";
    public static final String PLAYBACK_PROPERTY = "playbackSettings";

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
     * The trakt.tv settings of the application.
     */
    @Builder.Default
    private TraktSettings traktSettings = TraktSettings.builder().build();
    /**
     * The video playback settings of the application.
     */
    @Builder.Default
    private PlaybackSettings playbackSettings = PlaybackSettings.builder().build();

    //region Getters & Setters

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

    public TraktSettings getTraktSettings() {
        if (traktSettings == null)
            traktSettings = TraktSettings.builder().build();

        return traktSettings;
    }

    public PlaybackSettings getPlaybackSettings() {
        if (playbackSettings == null)
            playbackSettings = PlaybackSettings.builder().build();

        return playbackSettings;
    }

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

    public void setTraktSettings(TraktSettings traktSettings) {
        if (Objects.equals(this.traktSettings, traktSettings))
            return;

        var oldValue = this.traktSettings;
        this.traktSettings = traktSettings;
        changes.firePropertyChange(TRAKT_PROPERTY, oldValue, this.traktSettings);
    }

    public void setPlaybackSettings(PlaybackSettings playbackSettings) {
        if (Objects.equals(this.playbackSettings, playbackSettings))
            return;

        var oldValue = this.playbackSettings;
        this.playbackSettings = playbackSettings;
        changes.firePropertyChange(PLAYBACK_PROPERTY, oldValue, this.playbackSettings);
    }

    //endregion
}
