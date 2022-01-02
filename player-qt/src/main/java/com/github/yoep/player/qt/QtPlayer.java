package com.github.yoep.player.qt;

import com.github.yoep.player.qt.player.PopcornPlayer;
import com.github.yoep.player.qt.player.PopcornPlayerEventListener;
import com.github.yoep.player.qt.player.PopcornPlayerState;
import com.github.yoep.player.qt.player.PopcornQtPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerException;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;
import org.springframework.lang.Nullable;

import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

@Slf4j
public class QtPlayer implements Player {
    static final String ID = "QT_NATIVE_PLAYER";
    static final String NAME = "QT player";
    static final Resource GRAPHIC_RESOURCE = new ClassPathResource("/external-qt-icon.png");

    private final Queue<PlayerListener> listeners = new ConcurrentLinkedQueue<>();

    private PopcornPlayer popcornPlayer;
    private PlayerState lastKnownState = PlayerState.UNKNOWN;
    private boolean initialized;

    public QtPlayer() {
        init(null);
    }

    QtPlayer(PopcornPlayer popcornPlayer) {
        init(popcornPlayer);
    }

    //region Player

    @Override
    public String getId() {
        return ID;
    }

    @Override
    public String getName() {
        return NAME;
    }

    @Override
    public Optional<Resource> getGraphicResource() {
        return Optional.of(GRAPHIC_RESOURCE);
    }

    @Override
    public PlayerState getState() {
        return lastKnownState;
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return false;
    }

    @Override
    public void dispose() {
        popcornPlayer.release();
        popcornPlayer = null;
    }

    @Override
    public void addListener(PlayerListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(PlayerListener listener) {
        listeners.remove(listener);
    }

    @Override
    public void play(PlayRequest request) {
        checkInitialized();

        popcornPlayer.show();
        popcornPlayer.play(request.getUrl());
    }

    @Override
    public void resume() {
        popcornPlayer.resume();
    }

    @Override
    public void pause() {
        popcornPlayer.pause();
    }

    @Override
    public void stop() {
        popcornPlayer.stop();
    }

    @Override
    public void seek(long time) {
        popcornPlayer.seek(time);
    }

    @Override
    public void volume(int volume) {
        //TODO: implement
    }

    //endregion

    //region PostConstruct

    private void init(@Nullable PopcornPlayer instance) {
        log.trace("Initializing QT player");
        try {
            var level = getPlayerLogLevel();
            var args = new String[]{"PopcornPlayer", "-l", level};

            this.popcornPlayer = Objects.requireNonNullElseGet(instance, () -> new PopcornQtPlayer(args));

            initializeListener();
            initialized = true;
            log.debug("Popcorn player has been initialized");
        } catch (Exception ex) {
            log.error("Failed to load the popcorn player library, " + ex.getMessage(), ex);
        }
    }

    private void initializeListener() {
        popcornPlayer.addListener(new PopcornPlayerEventListener() {
            @Override
            public void onStateChanged(PopcornPlayerState newState) {
                switch (newState) {
                    case PLAYING:
                        updateState(PlayerState.PLAYING);
                        break;
                    case PAUSED:
                        updateState(PlayerState.PAUSED);
                        break;
                    case BUFFERING:
                        updateState(PlayerState.BUFFERING);
                        break;
                    case STOPPED:
                        updateState(PlayerState.STOPPED);
                        break;
                    case UNKNOWN:
                        updateState(PlayerState.UNKNOWN);
                        break;
                    default:
                        log.error("Received unknown popcorn player state " + newState);
                        break;
                }
            }

            @Override
            public void onTimeChanged(long newValue) {
                if (newValue >= 0) {
                    QtPlayer.this.onTimeChanged(newValue);
                } else {
                    log.warn("Received invalid time value {}", newValue);
                }
            }

            @Override
            public void onDurationChanged(long newValue) {
                if (newValue >= 0) {
                    log.trace("Popcorn player duration changed to {}", newValue);
                    QtPlayer.this.onDurationChanged(newValue);
                } else {
                    log.warn("Received invalid duration value {}", newValue);
                }
            }
        });
    }

    //endregion

    //region Functions

    private void checkInitialized() {
        if (!initialized) {
            throw new VideoPlayerException("Popcorn player has not yet been initialized");
        }
    }

    private void onDurationChanged(long newValue) {
        invokeListeners(e -> e.onDurationChanged(newValue));
    }

    private void onTimeChanged(long newValue) {
        invokeListeners(e -> e.onTimeChanged(newValue));
    }

    private void updateState(PlayerState newState) {
        log.debug("Popcorn player state changed to " + newState);
        this.lastKnownState = newState;
        invokeListeners(e -> e.onStateChanged(newState));
    }

    private void invokeListeners(Consumer<PlayerListener> action) {
        listeners.forEach(e -> {
            try {
                action.accept(e);
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    private String getPlayerLogLevel() {
        if (log.isTraceEnabled()) {
            return "trace";
        } else if (log.isDebugEnabled()) {
            return "debug";
        }

        return "info";
    }

    //endregion
}
