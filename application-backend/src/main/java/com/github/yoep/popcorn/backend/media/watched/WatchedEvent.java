package com.github.yoep.popcorn.backend.media.watched;

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
public class WatchedEvent extends Structure implements Closeable {
    public static class ByValue extends WatchedEvent implements Structure.ByValue {
    }

    public Tag tag;
    public WatchedEventCUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        if (Objects.requireNonNull(tag) == Tag.WatchedStateChanged) {
            union.setType(WatchedStateChangedBody.class);
        }
        union.read();
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    @Getter
    @ToString
    @FieldOrder({"imdbId", "newState"})
    public static class WatchedStateChangedBody extends Structure implements Closeable {
        public static class ByReference extends WatchedStateChangedBody implements Structure.ByReference {
        }

        public String imdbId;
        public byte newState;

        public boolean getNewState() {
            return newState == 1;
        }

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    public static class WatchedEventCUnion extends Union {
        public static class ByValue extends WatchedEventCUnion implements Union.ByValue {}

        public WatchedStateChangedBody watched_state_changed;
    }

    public enum Tag implements NativeMapped {
        WatchedStateChanged;

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
