package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentInfoWrapper;
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
public class EventC extends Structure implements Closeable {
    public static class ByValue extends EventC implements Structure.ByValue {
        @Override
        public void close() {
            super.close();
        }
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
            case PLAYER_CHANGED -> {
                var event = union.getPlayerChanged_body().playerChangedEvent;

                return PlayerChangedEvent.builder()
                        .source(this)
                        .oldPlayerId(event.getOldPlayerId().orElse(null))
                        .newPlayerId(event.getNewPlayerId())
                        .newPlayerName(event.getNewPlayerName())
                        .build();
            }
            case PLAYER_STARTED -> {
                return PlayerStartedEvent.builder()
                        .source(this)
                        .build();
            }
            case PLAYER_STOPPED -> {
               return new PlayerStoppedEvent(this);
            }
            case LOADING_STARTED -> {
                return new LoadingStartedEvent(this);
            }
            case LOADING_COMPLETED -> {
                return new LoadingCompletedEvent(this);
            }
            case TORRENT_DETAILS_LOADED -> {
                var body = union.getTorrentDetailsLoaded_body();
                return new ShowTorrentDetailsEvent(this, "", body.getTorrentInfo());
            }
            case CLOSE_PLAYER -> {
                return new ClosePlayerEvent(this, ClosePlayerEvent.Reason.END_OF_VIDEO);
            }
            default -> {
                log.error("Failed to create ApplicationEvent from {}", this);
                return null;
            }
        }
    }

    private void updateUnionType() {
        switch (tag) {
            case PLAYER_CHANGED -> union.setType(PlayerChanged_Body.class);
            case PLAYBACK_STATE_CHANGED -> union.setType(PlaybackState_Body.class);
            case WATCH_STATE_CHANGED -> union.setType(WatchStateChanged_Body.class);
            case TORRENT_DETAILS_LOADED -> union.setType(TorrentDetailsLoaded_Body.class);
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
            playerChangedEvent.close();
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    @FieldOrder({"newState"})
    public static class PlaybackState_Body extends Structure implements Closeable {
        public Player.State newState;

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
    @FieldOrder({"torrentInfo"})
    public static class TorrentDetailsLoaded_Body extends Structure implements Closeable {
        public TorrentInfoWrapper.ByValue torrentInfo;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(torrentInfo)
                    .ifPresent(TorrentInfoWrapper::close);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class EventCUnion extends Union implements Closeable {
        public static class ByValue extends EventCUnion implements Union.ByValue {
        }

        public PlayerChanged_Body playerChanged_body;
        public PlaybackState_Body playbackState_body;
        public WatchStateChanged_Body watchStateChanged_body;
        public TorrentDetailsLoaded_Body torrentDetailsLoaded_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(playerChanged_body)
                    .ifPresent(PlayerChanged_Body::close);
            Optional.ofNullable(playbackState_body)
                    .ifPresent(PlaybackState_Body::close);
            Optional.ofNullable(watchStateChanged_body)
                    .ifPresent(WatchStateChanged_Body::close);
            Optional.ofNullable(torrentDetailsLoaded_body)
                    .ifPresent(TorrentDetailsLoaded_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        PLAYER_CHANGED,
        PLAYER_STARTED,
        PLAYER_STOPPED,
        PLAYBACK_STATE_CHANGED,
        WATCH_STATE_CHANGED,
        LOADING_STARTED,
        LOADING_COMPLETED,
        TORRENT_DETAILS_LOADED,
        CLOSE_PLAYER;

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
