package com.github.yoep.popcorn.ui.player.internal;

import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class InternalPlayerService {
    private final PlayerService playerService;

    private final Player internalPlayer = new InternalPlayer();

    @PostConstruct
    void init() {
        log.trace("Registering the internal Popcorn Time player");
        playerService.register(internalPlayer);
        playerService.setActivePlayer(internalPlayer);
    }
}
