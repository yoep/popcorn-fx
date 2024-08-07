package com.github.yoep.popcorn.backend.subtitles.ffi;

import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleEventTag;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Objects;
import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"tag", "union"})
public class SubtitleEvent extends Structure implements Closeable {
    public static class ByValue extends SubtitleEvent implements Structure.ByValue {
    }

    public SubtitleEventTag tag;
    public SubtitleEvent.SubtitleEventCUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        if (Objects.requireNonNull(tag) == SubtitleEventTag.SubtitleInfoChanged) {
            union.setType(SubtitleEvent.SubtitleInfoChanged_Body.class);
        }
        if (Objects.requireNonNull(tag) == SubtitleEventTag.PreferredLanguageChanged) {
            union.setType(SubtitleEvent.PreferredLanguageChanged_Body.class);
        }
        union.read();
    }

    @Override
    public void close() {
        setAutoSynch(false);
        getUnion().close();
    }

    @Getter
    @ToString
    @FieldOrder({"subtitleInfo"})
    public static class SubtitleInfoChanged_Body extends Structure implements Closeable {
        public SubtitleInfo.ByReference subtitleInfo;

        public SubtitleInfo getSubtitleInfo() {
            return subtitleInfo;
        }

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(subtitleInfo)
                    .ifPresent(SubtitleInfo::close);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"subtitleLanguage"})
    public static class PreferredLanguageChanged_Body extends Structure implements Closeable {
        public SubtitleLanguage subtitleLanguage;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    public static class SubtitleEventCUnion extends Union implements Closeable {
        public static class ByValue extends SubtitleEventCUnion implements Union.ByValue {
        }

        public SubtitleInfoChanged_Body subtitle_info_changed;
        public PreferredLanguageChanged_Body preferred_language_changed;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(subtitle_info_changed)
                    .ifPresent(SubtitleEvent.SubtitleInfoChanged_Body::close);
            Optional.ofNullable(preferred_language_changed)
                    .ifPresent(SubtitleEvent.PreferredLanguageChanged_Body::close);
        }
    }
}
