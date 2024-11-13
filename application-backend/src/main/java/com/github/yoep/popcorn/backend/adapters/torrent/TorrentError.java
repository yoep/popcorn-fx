package com.github.yoep.popcorn.backend.adapters.torrent;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.*;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Optional;

@Getter
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"tag", "union"})
public class TorrentError extends Structure implements Closeable {
    public static class ByValue extends TorrentError implements Structure.ByValue {
    }

    public Tag tag;
    public TorrentErrorUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        updateUnionType();
        union.read();
    }

    @Override
    public void write() {
        updateUnionType();
        super.write();
    }

    @Override
    public void close() {
        setAutoSynch(false);
        getUnion().close();
    }

    @Override
    public String toString() {
        switch (tag) {
            case INVALID_URL -> {
                return getUnion().getInvalidUrl_body().getText();
            }
            case TORRENT -> {
                return getUnion().getTorrent_body().getText();
            }
            default -> {
                return getTag().toString();
            }
        }
    }

    void updateUnionType() {
        switch (tag) {
            case INVALID_URL -> union.setType(InvalidUrl_Body.class);
            case TORRENT_RESOLVING_FAILED -> union.setType(TorrentResolvingFailed_Body.class);
            case TORRENT -> union.setType(Torrent_Body.class);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"text"})
    public static class InvalidUrl_Body extends Structure implements Closeable {
        public String text;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"text"})
    @NoArgsConstructor
    @AllArgsConstructor
    public static class TorrentResolvingFailed_Body extends Structure implements Closeable {
        public String text;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"text"})
    public static class Torrent_Body extends Structure implements Closeable {
        public String text;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class TorrentErrorUnion extends Union implements Closeable {
        public static class ByValue extends TorrentErrorUnion implements Structure.ByValue {

        }

        public InvalidUrl_Body invalidUrl_body;
        public TorrentResolvingFailed_Body torrentResolvingFailed_body;
        public Torrent_Body torrent_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(invalidUrl_body)
                    .ifPresent(InvalidUrl_Body::close);
            Optional.ofNullable(torrentResolvingFailed_body)
                    .ifPresent(TorrentResolvingFailed_Body::close);
            Optional.ofNullable(torrent_body)
                    .ifPresent(Torrent_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        INVALID_URL,
        FILE_NOT_FOUND,
        FILE_ERROR,
        INVALID_STREAM_STATE,
        INVALID_MANAGER_STATE,
        INVALID_HANDLE,
        TORRENT_RESOLVING_FAILED,
        TORRENT_COLLECTION_LOADING_FAILED,
        TORRENT;

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
