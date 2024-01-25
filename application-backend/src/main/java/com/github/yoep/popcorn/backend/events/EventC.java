package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEvent;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Optional;

@Slf4j
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

    public <T extends ApplicationEvent> T toEvent() {
        switch (tag) {
            case PlayerChanged -> {
                var event = union.playerChanged_body.playerChangedEvent;

                return (T) PlayerChangedEvent.builder()
                        .source(this)
                        .oldPlayerId(event.getOldPlayerId().orElse(null))
                        .newPlayerId(event.getNewPlayerId())
                        .newPlayerName(event.getNewPlayerName())
                        .build();
            }
            case PlayerStopped -> {
                var event = union.playerStopped_body.stoppedEvent;

                return (T) PlayerStoppedEvent.builder()
                        .source(this)
                        .url(event.url)
                        .time(event.getTime())
                        .duration(event.getDuration())
                        .media(event.media.getMedia())
                        .build();
            }
            default -> {
                log.error("Failed to create ApplicationEvent from {}", this);
                return null;
            }
        }
    }

    private void updateUnionType() {
        switch (tag) {
            case PlayerChanged -> union.setType(PlayerChanged_Body.class);
            case PlayerStopped -> union.setType(PlayerStopped_Body.class);
            case PlayVideo -> union.setType(PlayVideo_Body.class);
            case PlaybackStateChanged -> union.setType(PlaybackState_Body.class);
        }
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
    @FieldOrder({"newState"})
    public static class PlaybackState_Body extends Structure implements Closeable {
        public PlayerState newState;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class EventCUnion extends Union implements Closeable {
        public static class ByValue extends EventCUnion implements Union.ByValue {
        }

        public PlayerChanged_Body playerChanged_body;
        public PlayerStopped_Body playerStopped_body;
        public PlayVideo_Body playVideo_body;
        public PlaybackState_Body playbackState_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(playerChanged_body)
                    .ifPresent(PlayerChanged_Body::close);
            Optional.ofNullable(playerStopped_body)
                    .ifPresent(PlayerStopped_Body::close);
            Optional.ofNullable(playVideo_body)
                    .ifPresent(PlayVideo_Body::close);
            Optional.ofNullable(playbackState_body)
                    .ifPresent(PlaybackState_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        PlayerChanged,
        PlayerStopped,
        PlayVideo,
        PlaybackStateChanged;

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
