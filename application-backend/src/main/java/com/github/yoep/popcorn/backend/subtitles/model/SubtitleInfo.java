package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;

/**
 * The subtitle info contains information about available subtitles for a certain IMDB ID.
 * This info includes a specific language for the media ID as well as multiple available files which can be used for smart subtitle detection.
 */
@Slf4j
public record SubtitleInfo(Subtitle.Info proto) {
    public String imdbId() {
        return proto.getImdbId();
    }
    
    public Subtitle.Language language() {
        return proto.getLanguage();
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
        return proto.getLanguage() == Subtitle.Language.NONE;
    }

    /**
     * Check if this subtitle is the special "custom" subtitle.
     *
     * @return Returns true if this subtitle is the "custom" subtitle, else false.
     */
    public boolean isCustom() {
        return proto.getLanguage() == Subtitle.Language.CUSTOM;
    }

    /**
     * Get the flag resource for this subtitle.
     * The flag resource should exist as the "unknown"/"not supported" languages are already filtered by the {@link com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Language}.
     *
     * @return Returns the flag class path resource.
     */
    public String getFlagResource() {
        return "/images/flags/" + SubtitleHelper.getCode(proto.getLanguage()) + ".png";
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof SubtitleInfo that)) return false;

        return Objects.equals(imdbId(), that.imdbId()) && language() == that.language();
    }

    @Override
    public int hashCode() {
        int result = Objects.hashCode(imdbId());
        result = 31 * result + language().hashCode();
        return result;
    }
}
