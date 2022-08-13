package com.github.yoep.popcorn.backend.subtitles.model;

import lombok.Getter;

import java.util.Arrays;

/**
 * The subtitle format type from which it was parsed.
 */
@Getter
public enum SubtitleType {
    SRT("srt"),
    VTT("vtt");

    private final String extension;

    SubtitleType(String extension) {
        this.extension = extension;
    }

    /**
     * Get the parser type based on the given file extension.
     *
     * @param extension The file extension.
     * @return Returns the parser type for the given file extension.
     * @throws EnumConstantNotPresentException Is thrown when know suitable parser type could be found for the given extension.
     */
    public static SubtitleType fromExtension(String extension) {
        return Arrays.stream(SubtitleType.values())
                .filter(e -> e.getExtension().equals(extension))
                .findFirst()
                .orElseThrow(() -> new EnumConstantNotPresentException(SubtitleType.class, extension));
    }
}
