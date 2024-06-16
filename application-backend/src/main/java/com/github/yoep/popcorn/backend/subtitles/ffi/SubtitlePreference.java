package com.github.yoep.popcorn.backend.subtitles.ffi;

import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitlePreferenceTag;
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
public class SubtitlePreference extends Structure implements Closeable {
    public static class ByReference extends SubtitlePreference implements Structure.ByReference {
        public ByReference() {
        }

        public ByReference(SubtitleLanguage language) {
            tag = SubtitlePreferenceTag.LANGUAGE;
            union = new SubtitlePreferenceUnion.ByValue();
            union.language_body = new SubtitleLanguagePreference_Body();
            union.language_body.language = language;
            updateUnionType();
        }

        public static SubtitlePreference.ByReference disabled() {
            var instance = new SubtitlePreference.ByReference();
            instance.tag = SubtitlePreferenceTag.DISABLED;
            instance.union = new SubtitlePreferenceUnion.ByValue();
            return instance;
        }
    }

    public SubtitlePreferenceTag tag;
    public SubtitlePreferenceUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        updateUnionType();
        union.read();
    }

    @Override
    public void close() {
        setAutoSynch(false);
        getUnion().close();
    }

    void updateUnionType() {
        if (Objects.requireNonNull(tag) == SubtitlePreferenceTag.LANGUAGE) {
            union.setType(SubtitleLanguagePreference_Body.class);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    @FieldOrder({"language"})
    public static class SubtitleLanguagePreference_Body extends Structure implements Closeable {
        public SubtitleLanguage language;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class SubtitlePreferenceUnion extends Union implements Closeable {
        public static class ByValue extends SubtitlePreferenceUnion implements Union.ByValue {

        }

        public SubtitleLanguagePreference_Body language_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(language_body)
                    .ifPresent(SubtitleLanguagePreference_Body::close);
        }
    }
}
