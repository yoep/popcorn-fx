package com.github.yoep.popcorn.backend.events;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Objects;
import java.util.Optional;

@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"tag", "union"})
public class EventC extends Structure implements Closeable {
    public static class ByValue extends EventC implements Structure.ByValue {
    }

    public EventC.Tag tag;
    public EventCUnion.ByValue union;

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

    private void updateUnionType() {
        if (Objects.requireNonNull(tag) == Tag.PlayerStopped) {
            union.setType(PlayerStoppedEventCBody.class);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"stoppedEvent"})
    public static class PlayerStoppedEventCBody extends Structure implements Closeable {
        public PlayerStoppedEventC.ByValue stoppedEvent;

        @Override
        public void close() {
            setAutoSynch(false);
            stoppedEvent.close();
        }
    }

    @Getter
    @ToString
    public static class EventCUnion extends Union implements Closeable {
        public static class ByValue extends EventCUnion implements Union.ByValue {
        }

        public PlayerStoppedEventCBody playerStoppedEventCBody;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(playerStoppedEventCBody)
                    .ifPresent(PlayerStoppedEventCBody::close);
        }
    }

    public enum Tag implements NativeMapped {
        PlayerStopped,
        PlayVideo;

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
