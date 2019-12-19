package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Torrent;

import java.util.Optional;

public interface PlayMediaActivity extends Activity {
    /**
     * Get the media that needs to be played.
     *
     * @return Returns the media that needs to be played.
     */
    Media getMedia();

    /**
     * Get the selected torrent to play.
     *
     * @return Returns the torrent that needs to be played, else {@link Optional#empty()}
     */
    Optional<Torrent> getTorrent();
}
