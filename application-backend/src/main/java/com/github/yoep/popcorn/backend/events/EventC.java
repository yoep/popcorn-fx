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

    public ApplicationEvent toEvent() {
        switch (tag) {
            case PlayerChanged -> {
                var event = union.playerChanged_body.playerChangedEvent;

                return PlayerChangedEvent.builder()
                        .source(this)
                        .oldPlayerId(event.getOldPlayerId().orElse(null))
                        .newPlayerId(event.getNewPlayerId())
                        .newPlayerName(event.getNewPlayerName())
                        .build();
            }
            case PlayerStarted -> {
                var event = union.playerStarted_body.startedEvent;

                return PlayerStartedEvent.builder()
                        .source(this)
                        .url(event.getUrl())
                        .title(event.getTitle())
                        .thumbnail(event.getThumbnail().orElse(null))
                        .quality(event.getQuality().orElse(null))
                        .autoResumeTimestamp(event.getAutoResumeTimestamp())
                        .subtitleEnabled(event.isSubtitlesEnabled())
                        .build();
            }
            case PlayerStopped -> {
                var event = union.playerStopped_body.stoppedEvent;

                return PlayerStoppedEvent.builder()
                        .source(this)
                        .url(event.url)
                        .time(event.getTime())
                        .duration(event.getDuration())
                        .media(event.media.getMedia())
                        .build();
            }
            case LoadingStarted -> {
                return new LoadingStartedEvent(this);
            }
            case LoadingCompleted -> {
                return new LoadingCompletedEvent(this);
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
            case PlayerStarted -> union.setType(PlayerStarted_Body.class);
            case PlayerStopped -> union.setType(PlayerStopped_Body.class);
            case PlaybackStateChanged -> union.setType(PlaybackState_Body.class);
            case WatchStateChanged -> union.setType(WatchStateChanged_Body.class);
            case LoadingStarted -> union.setType(LoadingStarted_Body.class);
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
    @FieldOrder({"startedEvent"})
    public static class PlayerStarted_Body extends Structure implements Closeable {
        public PlayerStartedEventC.ByValue startedEvent;

        @Override
        public void close() {
            setAutoSynch(false);
            startedEvent.close();
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
    @FieldOrder({"imdbId", "watched"})
    public static class WatchStateChanged_Body extends Structure implements Closeable {
        public String imdbId;
        public byte watched;

        public boolean isWatched() {
            return watched == 1;
        }

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"url", "title"})
    public static class LoadingStarted_Body extends Structure implements Closeable {
        public String url;
        public String title;

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
        public PlayerStarted_Body playerStarted_body;
        public PlayerStopped_Body playerStopped_body;
        public PlaybackState_Body playbackState_body;
        public WatchStateChanged_Body watchStateChanged_body;
        public LoadingStarted_Body loadingStarted_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(playerChanged_body)
                    .ifPresent(PlayerChanged_Body::close);
            Optional.ofNullable(playerStarted_body)
                    .ifPresent(PlayerStarted_Body::close);
            Optional.ofNullable(playerStopped_body)
                    .ifPresent(PlayerStopped_Body::close);
            Optional.ofNullable(playbackState_body)
                    .ifPresent(PlaybackState_Body::close);
            Optional.ofNullable(watchStateChanged_body)
                    .ifPresent(WatchStateChanged_Body::close);
            Optional.ofNullable(loadingStarted_body)
                    .ifPresent(LoadingStarted_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        PlayerChanged,
        PlayerStarted,
        PlayerStopped,
        PlaybackStateChanged,
        WatchStateChanged,
        LoadingStarted,
        LoadingCompleted;

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
