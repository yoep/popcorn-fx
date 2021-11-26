package com.github.yoep.player.popcorn.services;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerManagerService;
import com.github.yoep.player.popcorn.PopcornPlayer;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

/**
 * The register service is responsible for registering the popcorn player in the application.
 */
@Slf4j
@Service
public class RegisterService {
    static final String PLAYER_SECTION_VIEW = "common/sections/popcorn-player.section.fxml";

    private final PlayerManagerService playerService;
    private final ViewLoader viewLoader;
    private final Player player;

    public RegisterService(PlayerManagerService playerService, ViewLoader viewLoader) {
        this.playerService = playerService;
        this.viewLoader = viewLoader;
        this.player = createPopcornPlayer();
    }

    public Player getPlayer() {
        return player;
    }

    @PostConstruct
    void init() {
        log.trace("Registering the embedded Popcorn Time player");
        playerService.register(player);
        playerService.setActivePlayer(player);
    }

    private PopcornPlayer createPopcornPlayer() {
        var embeddablePlayer = viewLoader.load(PLAYER_SECTION_VIEW);

        return new PopcornPlayer(embeddablePlayer);
    }
}
