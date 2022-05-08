package com.github.yoep.player.vlc;

import com.github.yoep.player.vlc.mappers.StateMapper;
import com.github.yoep.player.vlc.model.VlcState;
import com.github.yoep.player.vlc.services.VlcPlayerService;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;

import java.util.Objects;
import java.util.Optional;

@Slf4j
@ToString(exclude = "listener")
@EqualsAndHashCode(exclude = "listener", callSuper = false)
public class VlcPlayer extends AbstractListenerService<PlayerListener> implements Player {
    static final String IDENTIFIER = "VLC";
    static final Resource GRAPHIC_RESOURCE = new ClassPathResource("/external-vlc-icon.png");
    static final String DESCRIPTION = "VLC is a free and open source cross-platform multimedia player";
    static final String COMMAND_PLAY_PAUSE = "pl_pause";
    static final String COMMAND_STOP = "pl_stop";
    static final String COMMAND_SEEK = "seek";
    static final String COMMAND_VOLUME = "volume";

    private final VlcPlayerService service;
    private final VlcListener listener = createListener();

    private PlayerState playerState;

    public VlcPlayer(VlcPlayerService service) {
        this.service = service;
        this.service.addListener(listener);
    }

    @Override
    public String getId() {
        return IDENTIFIER;
    }

    @Override
    public String getName() {
        return IDENTIFIER;
    }

    @Override
    public String getDescription() {
        return DESCRIPTION;
    }

    @Override
    public Optional<Resource> getGraphicResource() {
        return Optional.of(GRAPHIC_RESOURCE);
    }

    @Override
    public PlayerState getState() {
        return playerState;
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return false;
    }

    @Override
    public void dispose() {
        log.trace("Releasing VLC player resources");
        service.removeListener(listener);
        stop();
    }

    @Override
    public void play(PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        if (!service.play(request)) {
            updateState(PlayerState.ERROR);
        }
    }

    @Override
    public void resume() {
        service.executeCommand(COMMAND_PLAY_PAUSE);
    }

    @Override
    public void pause() {
        service.executeCommand(COMMAND_PLAY_PAUSE);
    }

    @Override
    public void stop() {
        log.trace("Stopping the VLC player");
        service.executeCommand(COMMAND_STOP);
        service.stop();
        updateState(PlayerState.STOPPED);
    }

    @Override
    public void seek(long time) {
        service.executeCommand(COMMAND_SEEK, String.valueOf(time / 1000));
    }

    @Override
    public void volume(int volume) {
        var vlcVolume = ((double) volume / 100) * 256;
        service.executeCommand(COMMAND_VOLUME, String.valueOf((int) vlcVolume));
    }

    @Override
    public int getVolume() {
        return 0;
    }

    private void updateState(PlayerState state) {
        playerState = state;
        invokeListeners(e -> e.onStateChanged(state));
    }

    private void onStateChanged(VlcState state) {
        Optional.ofNullable(state)
                .map(StateMapper::map)
                .ifPresent(e -> invokeListeners(listener -> listener.onStateChanged(e)));
    }

    private VlcListener createListener() {
        return new VlcListener() {
            @Override
            public void onTimeChanged(Long time) {
                invokeListeners(e -> e.onTimeChanged(time * 1000));
            }

            @Override
            public void onDurationChanged(Long duration) {
                invokeListeners(e -> e.onDurationChanged(duration * 1000));
            }

            @Override
            public void onStateChanged(VlcState state) {
                VlcPlayer.this.onStateChanged(state);
            }
        };
    }
}
