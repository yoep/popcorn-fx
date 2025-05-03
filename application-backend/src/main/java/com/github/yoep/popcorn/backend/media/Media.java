package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Images;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Rating;
import com.github.yoep.popcorn.backend.media.providers.MediaType;

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
    String id();

    /**
     * Get the unescaped title of the media.
     *
     * @return Returns the media title.
     */
    String title();

    /**
     * Get the unescaped description of the media.
     *
     * @return Returns the media description.
     */
    String synopsis();

    /**
     * Get the year of the media.
     *
     * @return Returns the year of the media.
     */
    String year();

    /**
     * Get the duration of the media.
     *
     * @return Returns the duration of the media.
     */
    Integer runtime();

    /**
     * Get a list of genres for this media.
     *
     * @return Returns a list of genres (non-null).
     */
    List<String> genres();

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
    Images images();

    /**
     * Get the type of the media item.
     *
     * @return Returns the media type.
     */
    MediaType type();
}
