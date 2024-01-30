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
public class LoaderEventC extends Structure implements Closeable {
    public static class ByValue extends LoaderEventC implements Structure.ByValue {
    }

    public Tag tag;
    public LoaderEventCUnion.ByValue union;

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
            case StateChanged -> union.setType(StateChanged_Body.class);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"state"})
    public static class StateChanged_Body extends Structure implements Closeable {
        public LoaderState state;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class LoaderEventCUnion extends Union implements Closeable {
        public static class ByValue extends LoaderEventCUnion implements Union.ByValue {
        }

        public StateChanged_Body stateChanged_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(stateChanged_body)
                    .ifPresent(StateChanged_Body::close);
        }
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
