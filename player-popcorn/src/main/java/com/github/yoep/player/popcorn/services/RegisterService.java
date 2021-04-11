package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerService;
import com.github.yoep.player.popcorn.PopcornPlayer;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

/**
 * The register service is responsible for registering the popcorn player in the application.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class RegisterService {
    private final PlayerService playerService;

    private final Player player = createPlayer();

    @PostConstruct
    void init() {
        log.trace("Registering the embedded Popcorn Time player");
        playerService.register(player);
        playerService.setActivePlayer(player);
    }

    private PopcornPlayer createPlayer() {
        return new PopcornPlayer();
    }
}
