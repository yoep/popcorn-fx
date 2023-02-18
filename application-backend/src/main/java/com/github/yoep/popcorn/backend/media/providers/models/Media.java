package com.github.yoep.popcorn.backend.media.providers.models;

import java.util.List;
import java.util.Optional;

/**
 * Interface definition of media items of the Popcorn Time API.
 */
public interface Media {
    /**
     * Get the unique ID of the media.
     * This can be a IMDB ID or TVDB ID value that is returned.
     *
     * @return Returns the unique ID of the media (non-null).
     */
    String getId();

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

    /**
     * The rating information of the media item.
     *
     * @return Returns the rating of the media if known, else {@link Optional#empty()}.
     */
    Optional<Rating> getRating();

    /**
     * Get the images of the media item.
     *
     * @return Returns the images of the media.
     */
    Images getImages();

    MediaType getType();
}
