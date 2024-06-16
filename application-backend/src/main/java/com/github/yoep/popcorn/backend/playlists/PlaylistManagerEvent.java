package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.playlists.ffi.PlaylistItem;
import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"tag", "union"})
public class PlaylistManagerEvent extends Structure implements Closeable {
    public static class ByValue extends PlaylistManagerEvent implements Structure.ByValue {
    }

    public Tag tag;
    public PlaylistManagerEventUnion union;

    @Override
    public void read() {
        super.read();
        updateUnionType();
        union.read();
    }

    void updateUnionType() {
        switch (tag) {
            case PlaylistChanged -> {
            }
            case PlayingNext -> union.setType(PlayingNext_Body.class);
            case StateChanged -> union.setType(StateChanged_Body.class);
        }
    }

    @Override
    public void close() {
        setAutoSynch(false);
        getUnion().close();
    }

    @Getter
    @ToString
    @FieldOrder({"playingIn", "item"})
    public static class PlayingNext_Body extends Structure implements Closeable {
        public Long playingIn;
        public PlaylistItem.ByReference item;

        public Optional<Long> getPlayingIn() {
            return Optional.ofNullable(playingIn)
                    .filter(e -> e != 0);
        }

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"state"})
    public static class StateChanged_Body extends Structure implements Closeable {
        public PlaylistState state;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class PlaylistManagerEventUnion extends Union implements Closeable {
        public PlayingNext_Body playingNext_body;
        public StateChanged_Body stateChanged_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(playingNext_body)
                    .ifPresent(PlayingNext_Body::close);
            Optional.ofNullable(stateChanged_body)
                    .ifPresent(StateChanged_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        PlaylistChanged,
        PlayingNext,
        StateChanged;

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
