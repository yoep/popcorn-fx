package com.github.yoep.popcorn.activities;

public interface PlayMediaMovieActivity extends PlayMediaActivity {
    /**
     * Get the torrent quality that should be played.
     *
     * @return Returns the torrent quality.
     */
    String getQuality();
}
