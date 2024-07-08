package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import lombok.Builder;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;

import static java.util.Arrays.asList;

/**
 * The subtitle info contains information about available subtitles for a certain IMDB ID.
 * This info includes a specific language for the media ID as well as multiple available files which can be used for smart subtitle detection.
 */
@Slf4j
public record SubtitleInfo(String imdbId, SubtitleLanguage language, List<SubtitleFile> files) {
    @Builder
    public SubtitleInfo(String imdbId, SubtitleLanguage language, SubtitleFile... files) {
        this(imdbId, language, asList(files));
    }

    /**
     * Check if this subtitle is a special subtitle.
     *
     * @return Returns true if this subtitle is a special one, else false.
     */
    public boolean isSpecial() {
        return isNone() || isCustom();
    }

    /**
     * Check if this subtitle is the special "none" subtitle.
     *
     * @return Returns true if this subtitle is the "none" subtitle, else false.
     */
    public boolean isNone() {
        return language() == SubtitleLanguage.NONE;
    }

    /**
     * Check if this subtitle is the special "custom" subtitle.
     *
     * @return Returns true if this subtitle is the "custom" subtitle, else false.
     */
    public boolean isCustom() {
        return language() == SubtitleLanguage.CUSTOM;
    }

    /**
     * Get the flag resource for this subtitle.
     * The flag resource should exist as the "unknown"/"not supported" languages are already filtered by the {@link SubtitleLanguage}.
     *
     * @return Returns the flag class path resource.
     */
    public String getFlagResource() {
        return "/images/flags/" + language.getCode() + ".png";
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof SubtitleInfo that)) return false;

        return Objects.equals(imdbId, that.imdbId) && language == that.language;
    }

    @Override
    public int hashCode() {
        int result = Objects.hashCode(imdbId);
        result = 31 * result + language.hashCode();
        return result;
    }

    public static SubtitleInfo from(com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo info) {
        Objects.requireNonNull(info, "info cannot be null");
        return SubtitleInfo.builder()
                .imdbId(info.imdbId)
                .language(info.language)
                .files(info.getFiles().stream()
                        .map(SubtitleFile::from)
                        .toArray(SubtitleFile[]::new))
                .build();
    }
}
