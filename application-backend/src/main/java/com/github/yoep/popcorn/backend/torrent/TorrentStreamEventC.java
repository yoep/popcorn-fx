package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
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
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"tag", "union"})
public class TorrentStreamEventC extends Structure implements Closeable {
    public static class ByValue extends TorrentStreamEventC implements Structure.ByValue {
        @Override
        public void close() {
            super.close();
            FxLib.INSTANCE.get().dispose_torrent_stream_event_value(this);
        }
    }

    public Tag tag;
    public TorrentStreamEventCUnion union;

    @Override
    public void read() {
        super.read();
        updateUnionType();
        union.read();
    }

    void updateUnionType() {
        switch (tag) {
            case STATE_CHANGED -> union.setType(StateChanged_Body.class);
            case DOWNLOAD_STATUS -> union.setType(DownloadStatus_Body.class);
        }
    }

    @Override
    public void close() {
        setAutoSynch(false);
        getUnion().close();
    }

    @Getter
    @ToString
    @FieldOrder({"state"})
    public static class StateChanged_Body extends Structure implements Closeable {
        public TorrentStreamState state;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"status"})
    public static class DownloadStatus_Body extends Structure implements Closeable {
        public DownloadStatusC.ByValue status;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class TorrentStreamEventCUnion extends Union implements Closeable {

        public StateChanged_Body stateChanged_body;
        public DownloadStatus_Body downloadStatus_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(stateChanged_body)
                    .ifPresent(StateChanged_Body::close);
            Optional.ofNullable(downloadStatus_body)
                    .ifPresent(DownloadStatus_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        STATE_CHANGED,
        DOWNLOAD_STATUS;

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
