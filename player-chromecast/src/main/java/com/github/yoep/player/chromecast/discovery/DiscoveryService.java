package com.github.yoep.player.chromecast.discovery;

import com.github.yoep.player.chromecast.ChromecastPlayer;
import com.github.yoep.player.chromecast.services.ChromecastService;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import su.litvak.chromecast.api.v2.ChromeCast;
import su.litvak.chromecast.api.v2.ChromeCasts;
import su.litvak.chromecast.api.v2.ChromeCastsListener;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.IOException;

@Slf4j
@RequiredArgsConstructor
public class DiscoveryService implements ChromeCastsListener {
    private final PlayerManagerService playerService;
    private final ChromecastService chromecastService;

    private Thread discoveryThread;

    //region ChromeCastsListener

    @Override
    public void newChromeCastDiscovered(ChromeCast chromeCast) {
        log.info("Discovered new Chromecast device \"{}\"", chromeCast.getName());
        var chromecastPlayer = new ChromecastPlayer(chromeCast, chromecastService);

        playerService.register(chromecastPlayer);
    }

    @Override
    public void chromeCastRemoved(ChromeCast chromeCast) {
        log.debug("Chromecast device \"{}\" has been removed", chromeCast.getName());
        playerService.getPlayers().stream()
                .filter(e -> e instanceof ChromecastPlayer)
                .filter(e -> e.getId().equals(chromeCast.getName()))
                .findFirst()
                .ifPresent(playerService::unregister);
    }

    //endregion

    //region Functions

    @PostConstruct
    void init() {
        discoveryThread = new Thread(this::startDiscovery, "ChromecastDiscovery");
        discoveryThread.start();
    }

    @PreDestroy
    void onDestroy() {
        stopDiscovery();

        if (discoveryThread != null && discoveryThread.isAlive()) {
            discoveryThread.interrupt();
        }
    }

    private void startDiscovery() {
        try {
            ChromeCasts.registerListener(this);

            log.debug("Starting Chromecast device discovery");
            ChromeCasts.startDiscovery();
        } catch (IOException ex) {
            log.error("Chromecast discovery failed, {}", ex.getMessage(), ex);
        }
    }

    private void stopDiscovery() {
        try {
            log.debug("Stopping Chromecast discovery");
            ChromeCasts.stopDiscovery();
        } catch (IOException ex) {
            log.error("Chromecast discovery failed to stop, {}", ex.getMessage(), ex);
        }
    }

    //endregion
}
