package com.github.yoep.popcorn.models.settings;

import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.media.providers.models.Media;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class Settings {
    private static final String DEFAULT_TORRENT_DIRECTORY = "torrents";

    /**
     * The directory to save the torrents to.
     */
    @Builder.Default
    private File torrentDirectory = new File(PopcornTimeApplication.APP_DIR + DEFAULT_TORRENT_DIRECTORY);
    /**
     * The indication if the torrent directory should be cleaned if the application is closed.
     */
    @Builder.Default
    private boolean torrentDirectoryCleaningEnabled = true;
    /**
     * The favorites of the user.
     */
    @Builder.Default
    private List<Media> favorites = new ArrayList<>();
}
