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
        switch (tag) {
            case PlayerStopped -> union.setType(PlayerStopped_Body.class);
            case PlayVideo -> union.setType(PlayVideo_Body.class);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"stoppedEvent"})
    public static class PlayerStopped_Body extends Structure implements Closeable {
        public PlayerStoppedEventC.ByValue stoppedEvent;

        @Override
        public void close() {
            setAutoSynch(false);
            stoppedEvent.close();
        }
    }

    @Getter
    @ToString
    @FieldOrder({"playVideoEvent"})
    public static class PlayVideo_Body extends Structure implements Closeable {
        public PlayVideoEventC.ByValue playVideoEvent;

        @Override
        public void close() {
            setAutoSynch(false);
            playVideoEvent.close();
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class EventCUnion extends Union implements Closeable {
        public static class ByValue extends EventCUnion implements Union.ByValue {
        }

        public PlayerStopped_Body playerStopped_body;
        public PlayVideo_Body playVideo_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(playerStopped_body)
                    .ifPresent(PlayerStopped_Body::close);
            Optional.ofNullable(playVideo_body)
                    .ifPresent(PlayVideo_Body::close);
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
