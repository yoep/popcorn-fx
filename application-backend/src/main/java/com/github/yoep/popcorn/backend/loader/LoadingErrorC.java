package com.github.yoep.popcorn.backend.loader;

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
public class LoadingErrorC extends Structure implements Closeable {
    public Tag tag;
    public LoadingErrorCUnion.ByValue union;

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

    private void updateUnionType() {
        switch (tag) {
            case ParseError -> union.setType(ParseError_Body.class);
            case TorrentError -> {
            }
            case MediaError -> union.setType(MediaError_Body.class);
            case TimeoutError -> union.setType(TimeoutError_Body.class);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"message"})
    public static class ParseError_Body extends Structure implements Closeable {
        public String message;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"message"})
    public static class MediaError_Body extends Structure implements Closeable {
        public String message;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"message"})
    public static class TorrentError_Body extends Structure implements Closeable {
        public String message;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"message"})
    public static class TimeoutError_Body extends Structure implements Closeable {
        public String message;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class LoadingErrorCUnion extends Union implements Closeable {
        public static class ByValue extends LoadingErrorCUnion implements Structure.ByValue {
        }

        public ParseError_Body parseError_body;
        public MediaError_Body mediaError_body;
        public TorrentError_Body torrentError_body;
        public TimeoutError_Body timeoutError_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(parseError_body)
                    .ifPresent(ParseError_Body::close);
            Optional.ofNullable(mediaError_body)
                    .ifPresent(MediaError_Body::close);
            Optional.ofNullable(torrentError_body)
                    .ifPresent(TorrentError_Body::close);
            Optional.ofNullable(timeoutError_body)
                    .ifPresent(TimeoutError_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        ParseError,
        TorrentError,
        MediaError,
        TimeoutError;

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
