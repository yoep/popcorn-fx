package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
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
public class TorrentStreamEvent extends Structure implements Closeable {
    public static class ByValue extends TorrentStreamEvent implements Structure.ByValue {
    }

    public TorrentStreamEvent.Tag tag;
    public TorrentStreamEvent.TorrentStreamEventCUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        if (Objects.requireNonNull(tag) == TorrentStreamEvent.Tag.StateChanged) {
            union.setType(TorrentStreamEvent.StateChangedBody.class);
        }
        union.read();
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    @Getter
    @ToString
    @FieldOrder({"newState"})
    public static class StateChangedBody extends Structure implements Closeable {
        public static class ByReference extends TorrentStreamEvent.StateChangedBody implements Structure.ByReference {
        }

        public TorrentStreamState newState;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    public static class TorrentStreamEventCUnion extends Union {
        public static class ByValue extends TorrentStreamEvent.TorrentStreamEventCUnion implements Union.ByValue {
        }

        public TorrentStreamEvent.StateChangedBody state_changed;
    }

    public enum Tag implements NativeMapped {
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
