package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;

@Data
@EqualsAndHashCode(callSuper = false)
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"subtitleSettings", "torrentSettings", "uiSettings", "serverSettings", "playbackSettings", "trackingSettings"})
public class ApplicationSettings extends Structure implements Closeable {
    public SubtitleSettings subtitleSettings;
    public TorrentSettings torrentSettings;
    public UISettings uiSettings;
    public ServerSettings serverSettings;
    public PlaybackSettings playbackSettings;
    public TrackingSettings trackingSettings;


    //region Getters & Setters

    @Override
    public void close() {
        setAutoSynch(false);
        subtitleSettings.close();
        torrentSettings.close();
        uiSettings.close();
        serverSettings.close();
        playbackSettings.close();
        trackingSettings.close();
    }

    //endregion
}
