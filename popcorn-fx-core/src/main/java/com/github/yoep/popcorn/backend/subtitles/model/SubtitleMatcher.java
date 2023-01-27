package com.github.yoep.popcorn.backend.subtitles.model;

import com.sun.jna.Structure;
import lombok.Data;
import lombok.EqualsAndHashCode;
import org.springframework.lang.Nullable;

import java.io.Closeable;
import java.util.Optional;
import java.util.regex.Pattern;

@Data
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"name", "quality"})
public class SubtitleMatcher extends Structure implements Closeable {
    private static final Pattern QUALITY_PATTERN = Pattern.compile("([0-9]{3,4})p");

    public static class ByReference extends SubtitleMatcher implements Structure.ByReference {
    }

    public String name;
    public int quality;

    //region Constructors

    public SubtitleMatcher() {
    }

    private SubtitleMatcher(String name, Integer quality) {
        this.name = name;
        this.quality = Optional.ofNullable(quality).orElse(-1);
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
        Integer actualQuality = null;

        if (quality != null) {
            actualQuality = parseQuality(quality);
        }

        return new SubtitleMatcher(name, actualQuality);
    }

    /**
     * Get the subtitle matcher for the given name and quality.
     *
     * @param name    The name of the media file.
     * @param quality The quality of the media file.
     * @return Returns the subtitle matcher.
     */
    public static SubtitleMatcher from(String name, Integer quality) {
        return new SubtitleMatcher(name, quality);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //endregion

    //region Functions

    private static Integer parseQuality(String quality) {
        var matcher = QUALITY_PATTERN.matcher(quality);

        if (matcher.find()) {
            return Integer.parseInt(matcher.group(1));
        }

        return null;
    }

    //endregion
}
