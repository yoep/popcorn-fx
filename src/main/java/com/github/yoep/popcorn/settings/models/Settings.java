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
}
