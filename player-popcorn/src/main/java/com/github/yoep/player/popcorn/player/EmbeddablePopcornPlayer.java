package com.github.yoep.player.popcorn.player;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import javafx.scene.Node;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.InputStream;
import java.util.Objects;
import java.util.Optional;

@Slf4j
@ToString(exclude = "embeddablePlayer")
public class EmbeddablePopcornPlayer implements com.github.yoep.popcorn.backend.adapters.player.Player {
    static final String PLAYER_SECTION_VIEW = "common/popcorn/sections/popcorn-player.section.fxml";

    private final PlayerManagerService playerService;
    private final ViewLoader viewLoader;
    private final PopcornPlayer popcornPlayer;

    private Node embeddablePlayer;

    public EmbeddablePopcornPlayer(PlayerManagerService playerService, ViewLoader viewLoader, PopcornPlayer popcornPlayer) {
        this.playerService = playerService;
        this.viewLoader = viewLoader;
        this.popcornPlayer = popcornPlayer;
        init();
    }

    //region EmbeddablePlayer

    @Override
    public String getId() {
        return popcornPlayer.getId();
    }

    @Override
    public String getName() {
        return popcornPlayer.getName();
    }

    @Override
    public String getDescription() {
        return popcornPlayer.getDescription();
    }

    @Override
    public Optional<InputStream> getGraphicResource() {
        return popcornPlayer.getGraphicResource();
    }

    @Override
    public Player.State getState() {
        return popcornPlayer.getState();
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return true;
    }

    @Override
    public Optional<Node> getEmbeddedPlayer() {
        return Optional.ofNullable(embeddablePlayer);
    }

    @Override
    public void dispose() {
        popcornPlayer.dispose();
    }

    @Override
    public void addListener(PlayerListener listener) {
        popcornPlayer.addListener(listener);
    }

    @Override
    public void removeListener(PlayerListener listener) {
        popcornPlayer.removeListener(listener);
    }

    @Override
    public void play(Player.PlayRequest request) {
        popcornPlayer.play(request);
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
        popcornPlayer.volume(volume);
    }

    @Override
    public int getVolume() {
        return popcornPlayer.getVolume();
    }

    @Override
    public final boolean equals(Object o) {
        if (!(o instanceof com.github.yoep.popcorn.backend.adapters.player.Player that)) return false;

        return Objects.equals(getId(), that.getId());
    }

    @Override
    public int hashCode() {
        return Objects.hashCode(getId());
    }

    //endregion

    //region Init

    private void init() {
        initializeEmbeddablePlayer();
        registerPlayer();
    }

    private void initializeEmbeddablePlayer() {
        log.trace("Loading embeddable Popcorn Time player view");
        this.embeddablePlayer = viewLoader.load(PLAYER_SECTION_VIEW);
    }

    private void registerPlayer() {
        log.trace("Registering the embedded Popcorn Time player");
        playerService.register(this).whenComplete((response, throwable) -> {
            if (throwable == null) {
                log.debug("Registered embeddable Popcorn Time player");
                playerService.setActivePlayer(this);
            } else {
                log.error("Failed to register embeddable Popcorn Time player, {}", throwable.getMessage(), throwable);
            }
        });
    }

    //endregion
}
