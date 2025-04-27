package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;

@Slf4j
public record SubtitleInfoWrapper(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Info proto) implements ISubtitleInfo {
    @Override
    public String getImdbId() {
        return proto.getImdbId();
    }

    @Override
    public Subtitle.Language getLanguage() {
        return proto.getLanguage();
    }

    @Override
    public boolean isNone() {
        return Objects.equals(getLanguage(), Subtitle.Language.NONE);
    }

    @Override
    public boolean isCustom() {
        return Objects.equals(getLanguage(), Subtitle.Language.CUSTOM);
    }

    @Override
    public String getFlagResource() {
        return "/images/flags/" + SubtitleHelper.getCode(proto.getLanguage()) + ".png";
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof ISubtitleInfo that)) return false;

        return Objects.equals(getImdbId(), that.getImdbId()) &&
                Objects.equals(getLanguage(), that.getLanguage());
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(getImdbId()) ^ Objects.hashCode(getLanguage());
    }
}
