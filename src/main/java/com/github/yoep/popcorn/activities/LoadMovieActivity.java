package com.github.yoep.popcorn.activities;

public interface LoadMovieActivity extends PlayMediaActivity {
    /**
     * Get the torrent quality that should be played.
     *
     * @return Returns the torrent quality.
     */
    String getQuality();
}
