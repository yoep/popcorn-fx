package com.github.yoep.popcorn.media.providers.models;

import com.github.yoep.popcorn.subtitle.models.Subtitle;
import javafx.collections.ObservableList;

import java.util.List;

public interface Media {
    /**
     * Get the video ID of the media.
     *
     * @return Returns the video ID of the media.
     */
    String getVideoId();

    /**
     * Get the IMDB ID of the media.
     * Use this ID to retrieve the show details.
     *
     * @return Returns the IMDB ID.
     */
    String getImdbId();

    /**
     * Get the unescaped title of the media.
     *
     * @return Returns the media title.
     */
    String getTitle();

    /**
     * Get the unescaped description of the media.
     *
     * @return Returns the media description.
     */
    String getSynopsis();

    /**
     * Get the year of the media.
     *
     * @return Returns the year of the media.
     */
    String getYear();

    /**
     * Get the duration of the media.
     *
     * @return Returns the duration of the media.
     */
    Integer getRuntime();

    /**
     * Get a list of genres for this media.
     *
     * @return Returns a list of genres (non-null).
     */
    List<String> getGenres();

    Rating getRating();

    boolean isMovie();

    Images getImages();

    /**
     * Get the subtitles for this media.
     * This map is an observable as they it's async loaded from the media information and can be empty when first requested.
     *
     * @return Returns the observable map of subtitles.
     */
    ObservableList<Subtitle> getSubtitles();
}
