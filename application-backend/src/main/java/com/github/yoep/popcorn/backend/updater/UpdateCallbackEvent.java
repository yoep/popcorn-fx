package com.github.yoep.popcorn.backend.updater;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"tag", "union"})
public class UpdateCallbackEvent extends Structure implements Closeable {
    public static class ByValue extends UpdateCallbackEvent implements Structure.ByValue {
    }

    public UpdateCallbackEvent.Tag tag;
    public UpdateCallbackEvent.UpdateEventCUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        switch (tag) {
            case StateChanged -> union.setType(UpdateCallbackEvent.StateChangedBody.class);
            case UpdateAvailable -> union.setType(UpdateCallbackEvent.UpdateAvailableBody.class);
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
    @FieldOrder({"newState"})
    public static class StateChangedBody extends Structure implements Closeable {
        public static class ByReference extends StateChangedBody implements Structure.ByReference {
        }

        public UpdateState newState;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"newVersion"})
    public static class UpdateAvailableBody extends Structure implements Closeable {
        public static class ByReference extends UpdateAvailableBody implements Structure.ByReference {
        }

        public VersionInfo newVersion;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    public static class UpdateEventCUnion extends Union implements Closeable {
        public static class ByValue extends UpdateEventCUnion implements Union.ByValue {

        }
        public StateChangedBody state_changed;
        public UpdateAvailableBody update_available;

        @Override
        public void close() {
            Optional.ofNullable(state_changed)
                    .ifPresent(StateChangedBody::close);
            Optional.ofNullable(update_available)
                    .ifPresent(UpdateAvailableBody::close);
        }
    }

    public enum Tag implements NativeMapped {
        StateChanged,
        UpdateAvailable;

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
