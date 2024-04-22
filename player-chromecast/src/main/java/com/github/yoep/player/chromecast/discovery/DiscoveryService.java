package com.github.yoep.player.chromecast.discovery;

import com.github.yoep.player.chromecast.ChromecastPlayer;
import com.github.yoep.player.chromecast.services.ChromecastService;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import lombok.extern.slf4j.Slf4j;
import su.litvak.chromecast.api.v2.ChromeCast;
import su.litvak.chromecast.api.v2.ChromeCasts;
import su.litvak.chromecast.api.v2.ChromeCastsListener;

import javax.annotation.PreDestroy;
import java.io.IOException;
import java.util.Objects;

@Slf4j
public class DiscoveryService implements ChromeCastsListener {
    private final PlayerManagerService playerService;
    private final ChromecastService chromecastService;

    Thread discoveryThread;

    public DiscoveryService(PlayerManagerService playerService, ChromecastService chromecastService) {
        Objects.requireNonNull(playerService, "playerService cannot be null");
        Objects.requireNonNull(chromecastService, "chromecastService cannot be null");
        this.playerService = playerService;
        this.chromecastService = chromecastService;
        init();
    }

    //region ChromeCastsListener

    @Override
    public void newChromeCastDiscovered(ChromeCast chromeCast) {
        log.info("Discovered new Chromecast device \"{}\"", chromeCast.getName());
        var chromecastPlayer = new ChromecastPlayer(chromeCast, chromecastService);

        playerService.register(chromecastPlayer);
    }

    @Override
    public void chromeCastRemoved(ChromeCast chromeCast) {
        // the discovery has some weird issue once an app is launched
        // causing the Chromecast to always be removed
        // for this reason, we ignore the event
    }

    //endregion

    //region Functions

    private void init() {
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
