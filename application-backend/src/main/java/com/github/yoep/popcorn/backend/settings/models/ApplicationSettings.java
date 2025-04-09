package com.github.yoep.popcorn.backend.settings.models;

import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;

@Data
@EqualsAndHashCode(callSuper = false)
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class ApplicationSettings {
    public SubtitleSettings subtitleSettings;
    public TorrentSettings torrentSettings;
    public UISettings uiSettings;
    public ServerSettings serverSettings;
    public PlaybackSettings playbackSettings;
    public TrackingSettings trackingSettings;
}
