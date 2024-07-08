package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import lombok.Data;
import lombok.EqualsAndHashCode;

import java.io.Closeable;
import java.util.Objects;

@Data
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"name", "quality"})
public class SubtitleMatcher extends Structure implements Closeable {
    public static class ByReference extends SubtitleMatcher implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(String name, String quality) {
            super(name, quality);
        }
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
     * @param quality The quality of the media (optional).
     * @return Returns the subtitle matcher.
     */
    public static ByReference from(String name, String quality) {
        Objects.requireNonNull(name, "name cannot be null");
        return new ByReference(name, quality);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //endregion
}
