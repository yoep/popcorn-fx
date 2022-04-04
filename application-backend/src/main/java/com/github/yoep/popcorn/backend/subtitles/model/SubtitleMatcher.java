package com.github.yoep.popcorn.backend.subtitles.model;

import lombok.Data;

import java.util.regex.Pattern;

@Data
public class SubtitleMatcher {
    private static final Pattern QUALITY_PATTERN = Pattern.compile("([0-9]{3,4})p");

    private final String name;
    private final Integer quality;

    //region Constructors

    private SubtitleMatcher(String name, Integer quality) {
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
    public static SubtitleMatcher from(String name, String quality) {
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
