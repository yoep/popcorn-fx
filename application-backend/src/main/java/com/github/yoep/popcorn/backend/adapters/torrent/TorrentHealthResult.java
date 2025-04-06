package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentHealth;
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
public class TorrentHealthResult extends Structure implements Closeable {
    public static class ByValue extends TorrentHealthResult implements Structure.ByValue {
    }

    public Tag tag;
    public ResultUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        switch (tag) {
            case Ok -> union.setType(TorrentHealthResult.OkBody.class);
            case Err -> union.setType(TorrentHealthResult.ErrBody.class);
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
    @FieldOrder({"value"})
    public static class OkBody extends Structure implements Closeable {
        public TorrentHealth.ByValue value;

        @Override
        public void close() {
            setAutoSynch(false);
            value.close();
        }
    }

    @Getter
    @ToString
    @FieldOrder({"error"})
    public static class ErrBody extends Structure implements Closeable {
        public TorrentError.ByValue error;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    public static class ResultUnion extends Union implements Closeable {
        public static class ByValue extends ResultUnion implements Union.ByValue {
        }

        public OkBody ok;
        public ErrBody err;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(ok)
                    .ifPresent(OkBody::close);
            Optional.ofNullable(err)
                    .ifPresent(ErrBody::close);
        }
    }

    public enum Tag implements NativeMapped {
        Ok,
        Err;

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
