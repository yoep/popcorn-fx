package com.github.yoep.popcorn.backend.media.providers;

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
@Structure.FieldOrder({"poster", "fanart", "banner"})
public class Images extends Structure implements Serializable, Closeable {
    public static class ByValue extends Images implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(Images images) {
            this.poster = images.poster;
            this.banner = images.banner;
            this.fanart = images.fanart;
        }
    }

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
