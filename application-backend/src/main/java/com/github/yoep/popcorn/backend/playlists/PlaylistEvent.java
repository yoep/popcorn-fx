package com.github.yoep.popcorn.backend.playlists;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.util.Arrays;

@Slf4j
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"tag", "union"})
public class PlaylistEvent extends Structure implements Closeable {
    public static class ByValue extends PlaylistEvent implements Structure.ByValue {
    }

    public Tag tag;
    public PlaylistEventUnion union;

    @Override
    public void read() {
        super.read();
        updateUnionType();
        union.read();
    }

    void updateUnionType() {
        switch (tag) {
            case PLAYING_NEXT -> {}
            case STATE_CHANGED -> union.setType(PlaylistState.class);
        }
    }

    @Override
    public void close() {
        setAutoSynch(false);
        getUnion().close();
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class PlaylistEventUnion extends Union implements Closeable {
        
        public PlaylistState state;
        
        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    public enum Tag implements NativeMapped {
        PLAYLIST_CHANGED,
        PLAYING_NEXT,
        STATE_CHANGED;

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
