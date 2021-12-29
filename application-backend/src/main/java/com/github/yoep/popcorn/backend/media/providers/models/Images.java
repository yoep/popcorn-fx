package com.github.yoep.popcorn.backend.media.providers.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.io.Serializable;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class Images implements Serializable {
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
