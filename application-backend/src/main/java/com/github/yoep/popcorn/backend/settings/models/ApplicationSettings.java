package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;

@Data
@EqualsAndHashCode(callSuper = false)
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"subtitleSettings", "torrentSettings", "uiSettings", "serverSettings"})
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
    public ServerSettings serverSettings;
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

    @Override
    public void close() {
        setAutoSynch(false);
        subtitleSettings.close();
        torrentSettings.close();
        uiSettings.close();
        serverSettings.close();
    }

    //endregion
}
