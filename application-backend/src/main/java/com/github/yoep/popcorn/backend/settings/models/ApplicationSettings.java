package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.Objects;

@Data
@EqualsAndHashCode(callSuper = false)
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"subtitleSettings", "torrentSettings", "uiSettings"})
public class ApplicationSettings extends Structure implements Closeable {
    public static final String TORRENT_PROPERTY = "torrentSettings";
    public static final String UI_PROPERTY = "uiSettings";
    public static final String TRAKT_PROPERTY = "traktSettings";
    public static final String LOGGING_PROPERTY = "loggingSettings";
    public static final String PLAYBACK_PROPERTY = "playbackSettings";
    public static final String SERVER_PROPERTY = "serverSettings";

    public SubtitleSettings subtitleSettings;
    public TorrentSettings torrentSettings;
    public UISettings uiSettings;
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
    /**
     * The server settings of the application.
     */
    @Builder.Default
    private ServerSettings serverSettings = ServerSettings.builder().build();

    //region Getters & Setters

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

    public ServerSettings getServerSettings() {
        if (serverSettings == null)
            serverSettings = ServerSettings.builder().build();

        return serverSettings;
    }

    public void setTraktSettings(TraktSettings traktSettings) {
        if (Objects.equals(this.traktSettings, traktSettings))
            return;

        var oldValue = this.traktSettings;
        this.traktSettings = traktSettings;
    }

    public void setPlaybackSettings(PlaybackSettings playbackSettings) {
        if (Objects.equals(this.playbackSettings, playbackSettings))
            return;

        var oldValue = this.playbackSettings;
        this.playbackSettings = playbackSettings;
    }

    public void setServerSettings(ServerSettings serverSettings) {
        if (Objects.equals(this.serverSettings, serverSettings))
            return;

        var oldValue = this.serverSettings;
        this.serverSettings = serverSettings;
    }

    @Override
    public void close() {
        setAutoSynch(false);
        subtitleSettings.close();
        torrentSettings.close();
        uiSettings.close();
    }

    //endregion
}
