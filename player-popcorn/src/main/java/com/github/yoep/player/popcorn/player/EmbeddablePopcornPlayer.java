package com.github.yoep.player.popcorn.player;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.embaddable.EmbeddablePlayer;
import com.github.yoep.popcorn.backend.adapters.player.embaddable.LayoutMode;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import javafx.scene.Node;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.util.Optional;

@Slf4j
@Component
@RequiredArgsConstructor
@EqualsAndHashCode(exclude = "embeddablePlayer")
@ToString(exclude = "embeddablePlayer")
public class EmbeddablePopcornPlayer implements EmbeddablePlayer {
    static final String PLAYER_SECTION_VIEW = "common/sections/popcorn-player.section.fxml";

    private final PopcornPlayer popcornPlayer;
    private final PlayerManagerService playerService;
    private final ViewLoader viewLoader;

    private Node embeddablePlayer;

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
    public Optional<Resource> getGraphicResource() {
        return popcornPlayer.getGraphicResource();
    }

    @Override
    public PlayerState getState() {
        return popcornPlayer.getState();
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return true;
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
    public void play(PlayRequest request) {
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
    public Node getEmbeddedPlayer() {
        return embeddablePlayer;
    }

    @Override
    public void setLayoutMode(LayoutMode mode) {
        //TODO: implement
    }

    //endregion

    //region Init

    @PostConstruct
    void init() {
        initializeEmbeddablePlayer();
        registerPlayer();
    }

    private void initializeEmbeddablePlayer() {
        log.trace("Loading embeddable Popcorn Time player view");
        this.embeddablePlayer = viewLoader.load(PLAYER_SECTION_VIEW);
    }

    private void registerPlayer() {
        log.trace("Registering the embedded Popcorn Time player");
        playerService.register(this);
        playerService.setActivePlayer(this);
    }

    //endregion
}
