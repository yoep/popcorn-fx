package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.sun.jna.Structure;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

import java.io.Closeable;
import java.io.Serializable;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
@Structure.FieldOrder({"poster", "fanart", "banner"})
public class Images extends Structure implements Serializable, Closeable {
    /**
     * The poster image of the media.
     */
    public String poster;
    /**
     * The fanart image of the media.
     */
    public String fanart;
    /**
     * The banner of the media.
     */
    public String banner;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
