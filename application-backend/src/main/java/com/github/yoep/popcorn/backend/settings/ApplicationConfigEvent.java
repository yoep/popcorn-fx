package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Objects;

@Getter
@ToString
@Structure.FieldOrder({"tag", "union"})
public class ApplicationConfigEvent extends Structure implements Closeable {
    public static class ByValue extends ApplicationConfigEvent implements Structure.ByValue {
    }

    public Tag tag;
    public ApplicationConfigEventUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        if (Objects.requireNonNull(tag) == Tag.SubtitleSettingsChanged) {
            union.setType(ApplicationConfigEvent.SubtitleSettingsChanged_Body.class);
        }
        union.read();
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class SubtitleSettingsChanged_Body extends Structure {
        public SubtitleSettings settings;
    }

    @Getter
    @ToString
    public static class ApplicationConfigEventUnion extends Union {
        public static class ByValue extends ApplicationConfigEventUnion implements Union.ByValue {
        }

        public ApplicationConfigEvent.SubtitleSettingsChanged_Body subtitleSettings;
    }

    public enum Tag implements NativeMapped {
        SettingsLoaded,
        SubtitleSettingsChanged;

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
