package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Optional;

@Slf4j
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"tag", "union"})
public class PlayerEventC extends Structure implements Closeable {
    public static class ByValue extends PlayerEventC implements Structure.ByValue {
        public static PlayerEventC.ByValue durationChanged(long duration) {
            var instance = new PlayerEventC.ByValue();
            instance.tag = Tag.DurationChanged;
            instance.union = new PlayerEventCUnion.ByValue();
            instance.union.durationChanged_body = new DurationChanged_Body(duration);
            instance.updateUnionType();
            instance.setAutoRead(false);
            return instance;
        }

        public static PlayerEventC.ByValue timeChanged(long duration) {
            var instance = new PlayerEventC.ByValue();
            instance.tag = Tag.TimeChanged;
            instance.union = new PlayerEventCUnion.ByValue();
            instance.union.timeChanged_body = new TimeChanged_Body(duration);
            instance.updateUnionType();
            instance.setAutoRead(false);
            return instance;
        }

        public static PlayerEventC.ByValue stateChanged(Player.State state) {
            var instance = new PlayerEventC.ByValue();
            instance.tag = Tag.StateChanged;
            instance.union = new PlayerEventCUnion.ByValue();
            instance.union.stateChanged_body = new StateChanged_Body(state);
            instance.updateUnionType();
            instance.setAutoRead(false);
            return instance;
        }

        @Override
        public void close() {
            super.close();
        }
    }

    public PlayerEventC.Tag tag;
    public PlayerEventCUnion union;

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
            case DurationChanged -> union.setType(DurationChanged_Body.class);
            case TimeChanged -> union.setType(TimeChanged_Body.class);
            case StateChanged -> union.setType(StateChanged_Body.class);
            case VolumeChanged -> {
            }
        }
    }

    @Getter
    @ToString
    @FieldOrder({"duration"})
    public static class DurationChanged_Body extends Structure implements Closeable {
        public Long duration;

        public DurationChanged_Body() {
        }

        public DurationChanged_Body(Long duration) {
            this.duration = duration;
        }

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"time"})
    public static class TimeChanged_Body extends Structure implements Closeable {
        public Long time;

        public TimeChanged_Body() {
        }

        public TimeChanged_Body(Long time) {
            this.time = time;
        }

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"state"})
    public static class StateChanged_Body extends Structure implements Closeable {
        public Player.State state;

        public StateChanged_Body() {
        }

        public StateChanged_Body(Player.State state) {
            this.state = state;
        }

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class PlayerEventCUnion extends Union implements Closeable {
        public static class ByValue extends PlayerEventCUnion implements Union.ByValue {
        }

        public DurationChanged_Body durationChanged_body;
        public TimeChanged_Body timeChanged_body;
        public StateChanged_Body stateChanged_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(durationChanged_body)
                    .ifPresent(DurationChanged_Body::close);
            Optional.ofNullable(timeChanged_body)
                    .ifPresent(TimeChanged_Body::close);
            Optional.ofNullable(stateChanged_body)
                    .ifPresent(StateChanged_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        DurationChanged,
        TimeChanged,
        StateChanged,
        VolumeChanged;

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
