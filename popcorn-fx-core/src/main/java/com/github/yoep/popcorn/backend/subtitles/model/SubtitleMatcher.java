package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import lombok.Data;
import lombok.EqualsAndHashCode;
import org.springframework.lang.Nullable;

import java.io.Closeable;

@Data
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"name", "quality"})
public class SubtitleMatcher extends Structure implements Closeable {
    public static class ByReference extends SubtitleMatcher implements Structure.ByReference {
    }

    public String name;
    public String quality;

    //region Constructors

    public SubtitleMatcher() {
    }

    private SubtitleMatcher(String name, String quality) {
        this.name = name;
        this.quality = quality;
    }

    //endregion

    //region Methods

    /**
     * Get the subtitle matcher for the given name and quality.
     *
     * @param name    The name of the media file.
     * @param quality The quality of the media.
     * @return Returns the subtitle matcher.
     */
    public static SubtitleMatcher from(String name, @Nullable String quality) {
        return new SubtitleMatcher(name, quality);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //endregion
}
