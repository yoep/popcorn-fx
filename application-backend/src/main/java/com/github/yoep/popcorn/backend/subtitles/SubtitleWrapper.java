package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Cue;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Info;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Language;

import java.util.List;
import java.util.Objects;
import java.util.Optional;

/**
 * The subtitle contains the parsed information of a subtitle file.
 * This is effectively a wrapper around the {@link SubtitleCue} objects which contain the actual parsed data and a reference to the original
 * {@link ISubtitleInfo} from which this {@link SubtitleWrapper} was generated.
 */
public record SubtitleWrapper(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle proto) implements ISubtitle {
    @Override
    public String getFilePath() {
        return proto.getFilePath();
    }

    @Override
    public List<Cue> cues() {
        return proto.getCuesList();
    }

    @Override
    public boolean isNone() {
        return getSubtitleInfo()
                .map(e -> e.getLanguage() == Language.NONE)
                .orElse(false);
    }

    @Override
    public Optional<Info> getSubtitleInfo() {
        return Optional.ofNullable(proto.getInfo());
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof ISubtitle that)) return false;

        return Objects.equals(getSubtitleInfo(), that.getSubtitleInfo());
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(getSubtitleInfo());
    }
}
