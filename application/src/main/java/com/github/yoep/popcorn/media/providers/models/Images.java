package com.github.yoep.popcorn.media.providers.models;

import lombok.Data;

@Data
public class Images {
    /**
     * The poster image of the media.
     */
    private String poster;
    /**
     * The fanart image of the media.
     */
    private String fanart;
    /**
     * The banner of the media.
     */
    private String banner;
}
