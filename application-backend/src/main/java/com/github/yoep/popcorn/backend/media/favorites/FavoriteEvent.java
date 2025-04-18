package com.github.yoep.popcorn.backend.media.favorites;

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
public class FavoriteEvent extends Structure implements Closeable {
    public static class ByValue extends FavoriteEvent implements Structure.ByValue {
    }

    public Tag tag;
    public FavoriteEventCUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        if (Objects.requireNonNull(tag) == Tag.LikedStateChanged) {
            union.setType(LikedStateChangedBody.class);
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
    @FieldOrder({"imdbId", "newState"})
    public static class LikedStateChangedBody extends Structure implements Closeable {
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
    public static class FavoriteEventCUnion extends Union implements Closeable {
        public static class ByValue extends FavoriteEventCUnion implements Union.ByValue {}

        public LikedStateChangedBody liked_state_changed;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(liked_state_changed)
                    .ifPresent(LikedStateChangedBody::close);
        }
    }

    public enum Tag implements NativeMapped {
        LikedStateChanged;

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
