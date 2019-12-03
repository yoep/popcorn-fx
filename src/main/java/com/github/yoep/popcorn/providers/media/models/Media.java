package com.github.yoep.popcorn.providers.media.models;

import java.util.List;
import java.util.Map;

public interface Media {
    /**
     * Get the video ID of the media.
     *
     * @return Returns the video ID of the media.
     */
    String getVideoId();

    String getImdbId();

    String getTitle();

    String getYear();

    List<String> getGenres();

    Rating getRating();

    boolean isMovie();

    Images getImages();

    Map<String, Map<String, Torrent>> getTorrents();

    Map<String, String> getSubtitles();
}
