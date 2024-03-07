package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.PlayerChangedEventC;
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
public class PlayerManagerEvent extends Structure implements Closeable {
    public static class ByValue extends PlayerManagerEvent implements Structure.ByValue {
        @Override
        public void close() {
            super.close();
            FxLib.INSTANCE.get().dispose_player_manager_event(this);
        }
    }

    public PlayerManagerEvent.Tag tag;
    public PlayerManagerEvent.PlayerManagerEventCUnion union;

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

    @Getter
    @ToString
    @FieldOrder({"playerChangedEvent"})
    public static class PlayerChanged_Body extends Structure implements Closeable {
        public PlayerChangedEventC.ByValue playerChangedEvent;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"duration"})
    public static class PlayerDurationChanged_Body extends Structure implements Closeable {
        public Long duration;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"time"})
    public static class PlayerTimeChanged_Body extends Structure implements Closeable {
        public Long time;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"state"})
    public static class PlayerStateChanged_Body extends Structure implements Closeable {
        public PlayerState state;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"request"})
    public static class PlayerPlaybackChanged_Body extends Structure implements Closeable {
        public PlayRequestWrapper.ByValue request;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(request)
                    .ifPresent(PlayRequestWrapper::close);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class PlayerManagerEventCUnion extends Union implements Closeable {
        public static class ByValue extends PlayerManagerEvent.PlayerManagerEventCUnion implements Union.ByValue {
        }

        public PlayerChanged_Body playerChanged_body;
        public PlayerDurationChanged_Body playerDurationChanged_body;
        public PlayerTimeChanged_Body playerTimeChanged_body;
        public PlayerStateChanged_Body playerStateChanged_body;
        public PlayerPlaybackChanged_Body playerPlaybackChanged_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(playerChanged_body)
                    .ifPresent(PlayerChanged_Body::close);
            Optional.ofNullable(playerDurationChanged_body)
                    .ifPresent(PlayerDurationChanged_Body::close);
            Optional.ofNullable(playerTimeChanged_body)
                    .ifPresent(PlayerTimeChanged_Body::close);
            Optional.ofNullable(playerStateChanged_body)
                    .ifPresent(PlayerStateChanged_Body::close);
            Optional.ofNullable(playerPlaybackChanged_body)
                    .ifPresent(PlayerPlaybackChanged_Body::close);
        }
    }

    private void updateUnionType() {
        switch (getTag()) {
            case ACTIVE_PLAYER_CHANGED -> union.setType(PlayerManagerEvent.PlayerChanged_Body.class);
            case PLAYER_PLAYBACK_CHANGED -> union.setType(PlayerPlaybackChanged_Body.class);
            case PLAYER_DURATION_CHANGED -> union.setType(PlayerDurationChanged_Body.class);
            case PLAYER_TIME_CHANGED -> union.setType(PlayerTimeChanged_Body.class);
            case PLAYER_STATE_CHANGED -> union.setType(PlayerStateChanged_Body.class);
            default -> {
            }
        }
    }

    public enum Tag implements NativeMapped {
        ACTIVE_PLAYER_CHANGED,
        PLAYERS_CHANGED,
        PLAYER_PLAYBACK_CHANGED,
        PLAYER_DURATION_CHANGED,
        PLAYER_TIME_CHANGED,
        PLAYER_STATE_CHANGED;

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
