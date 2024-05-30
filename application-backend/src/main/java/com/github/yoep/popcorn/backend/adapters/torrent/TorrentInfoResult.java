package com.github.yoep.popcorn.backend.adapters.torrent;

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
public class TorrentInfoResult extends Structure implements Closeable {
    public static class ByValue extends TorrentInfoResult implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(TorrentInfoWrapper.ByValue info) {
            super(info);
        }

        public ByValue(TorrentError.ByValue error) {
            super(error);
        }
    }

    public Tag tag;
    public TorrentInfoResultUnion.ByValue union;

    public TorrentInfoResult() {
    }

    public TorrentInfoResult(TorrentInfoWrapper.ByValue info) {
        this.tag = Tag.OK;
        this.union = new TorrentInfoResultUnion.ByValue();
        this.union.ok_body = new Ok_Body();
        this.union.ok_body.info = info;
        write();
    }

    public TorrentInfoResult(TorrentError.ByValue error) {
        this.tag = Tag.ERROR;
        this.union = new TorrentInfoResultUnion.ByValue();
        this.union.error_body = new Error_Body();
        this.union.error_body.error = error;
        write();
    }

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

    void updateUnionType() {
        switch (tag) {
            case OK -> union.setType(Ok_Body.class);
            case ERROR -> union.setType(Error_Body.class);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"info"})
    public static class Ok_Body extends Structure implements Closeable {
        public TorrentInfoWrapper.ByValue info;

        @Override
        public void close() {
            setAutoSynch(false);
            info.close();
        }
    }

    @Getter
    @ToString
    @FieldOrder({"error"})
    public static class Error_Body extends Structure implements Closeable {
        public TorrentError.ByValue error;

        @Override
        public void close() {
            setAutoSynch(false);
            error.close();
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class TorrentInfoResultUnion extends Union implements Closeable {
        public static class ByValue extends TorrentInfoResultUnion implements Structure.ByValue {
        }

        public Ok_Body ok_body;
        public Error_Body error_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(ok_body)
                    .ifPresent(Ok_Body::close);
            Optional.ofNullable(error_body)
                    .ifPresent(Error_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        OK,
        ERROR;

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
