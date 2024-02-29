package com.github.yoep.popcorn.backend.media.tracking;

import com.github.yoep.popcorn.backend.FxLib;
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
public class TrackingEventC extends Structure implements Closeable {
    public static class ByValue extends TrackingEventC implements Structure.ByValue {
        @Override
        public void close() {
            super.close();
            FxLib.INSTANCE.get().dispose_tracking_event_value(this);
        }
    }

    public Tag tag;
    public TrackingEventCUnion.ByValue union;

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
            case AUTHORIZATION_STATE_CHANGED -> union.setType(AuthorizationStateChanged_Body.class);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"state"})
    public static class AuthorizationStateChanged_Body extends Structure implements Closeable {
        public byte state;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class TrackingEventCUnion extends Union implements Closeable {
        public static class ByValue extends TrackingEventCUnion implements Union.ByValue {
        }

        public AuthorizationStateChanged_Body authorizationStateChanged_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(authorizationStateChanged_body)
                    .ifPresent(AuthorizationStateChanged_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        AUTHORIZATION_STATE_CHANGED;

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
