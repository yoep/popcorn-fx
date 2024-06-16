package com.github.yoep.popcorn.backend.media.providers;

import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import lombok.Builder;
import lombok.NoArgsConstructor;

import java.io.Closeable;
import java.io.Serializable;
import java.util.Optional;

@NoArgsConstructor
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
    public Pointer poster;
    /**
     * The fanart image of the media.
     */
    public Pointer fanart;
    /**
     * The banner of the media.
     */
    public Pointer banner;

    private String cachedPoster;
    private String cachedFanart;
    private String cachedBanner;

    @Builder
    public Images(Pointer poster, Pointer fanart, Pointer banner) {
        this.poster = poster;
        this.fanart = fanart;
        this.banner = banner;
        write();
        read();
    }

    public String getPoster() {
        return cachedPoster;
    }

    public String getFanart() {
        return cachedFanart;
    }

    public String getBanner() {
        return cachedBanner;
    }

    @Override
    public void read() {
        super.read();
        this.cachedPoster = Optional.ofNullable(poster)
                .map(e -> e.getString(0))
                .orElse(null);
        this.cachedFanart = Optional.ofNullable(fanart)
                .map(e -> e.getString(0))
                .orElse(null);
        this.cachedBanner = Optional.ofNullable(banner)
                .map(e -> e.getString(0))
                .orElse(null);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
