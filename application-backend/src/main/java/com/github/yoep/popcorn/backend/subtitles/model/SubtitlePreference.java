package com.github.yoep.popcorn.backend.subtitles.model;

import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Objects;
import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"tag", "union"})
public class SubtitlePreference extends Structure implements Closeable {
    public static class ByValue extends SubtitlePreference implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(SubtitleLanguage language) {
            tag = Tag.LANGUAGE;
            union = new SubtitlePreferenceUnion.ByValue();
            union.language_body = new SubtitleLanguagePreference_Body();
            union.language_body.language = language;
        }

        public static SubtitlePreference.ByValue disabled() {
            var instance = new SubtitlePreference.ByValue();
            instance.tag = Tag.DISABLED;
            instance.union = new SubtitlePreferenceUnion.ByValue();
            return instance;
        }
    }

    public Tag tag;
    public SubtitlePreferenceUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        if (Objects.requireNonNull(tag) == Tag.LANGUAGE) {
            union.setType(SubtitleLanguagePreference_Body.class);
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

    public enum Tag implements NativeMapped {
        LANGUAGE,
        DISABLED;

        @Override
        public Object fromNative(Object nativeValue, FromNativeContext context) {
            return Arrays.stream(values())
                    .filter(e -> e.ordinal() == (int) nativeValue)
                    .findFirst()
                    .orElse(null);
        }

        @Override
        public Object toNative() {
            return ordinal();
        }

        @Override
        public Class<?> nativeType() {
            return Integer.class;
        }
    }
}
