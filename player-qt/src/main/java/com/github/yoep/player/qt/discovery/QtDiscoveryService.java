package com.github.yoep.player.qt.discovery;

import com.github.yoep.player.qt.QtPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;

@Slf4j
@RequiredArgsConstructor
public class QtDiscoveryService {
    private final PlayerManagerService playerManagerService;

    @PostConstruct
    void init() {
        new Thread(this::initializePlayer, "QT-loader").start();
    }

    private void initializePlayer() {
        try {
            log.trace("Creating new QT player");
            var player = new QtPlayer();

            log.trace("Registering new QT player");
            playerManagerService.register(player);
            log.debug("QT player has been loaded");
        } catch (Throwable ex) {
            log.debug("Failed to load the QT player, {}", ex.getMessage(), ex);
            log.warn("QT player could not be loaded ");
        }
    }
}
