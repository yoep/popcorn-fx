package com.github.yoep.player.vlc.discovery;

import com.github.yoep.player.vlc.VlcPlayer;
import com.github.yoep.player.vlc.services.VlcPlayerService;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.io.File;
import java.util.Arrays;
import java.util.Optional;
import java.util.function.Supplier;

@Slf4j
@RequiredArgsConstructor
public class VlcDiscovery {
    static final String FILENAME = "vlc";
    static final String SYSTEM_PATH = "PATH";

    private final PlayerManagerService playerManagerService;
    private final VlcPlayerService vlcPlayerService;

    String discoveredVlcPath;
    Supplier<String> environmentPathSupplier = () -> System.getenv(SYSTEM_PATH);

    @PostConstruct
    void init() {
        new Thread(this::startDiscovery, "VlcDiscovery").start();
    }

    private void startDiscovery() {
        log.trace("Starting external VLC player discovery");
        Optional.ofNullable(environmentPathSupplier.get())
                .map(e -> e.split(File.pathSeparator))
                .stream()
                .flatMap(Arrays::stream)
                .map(File::new)
                .filter(this::containsVlcExecutable)
                .findFirst()
                .ifPresentOrElse(this::handleFoundVlcPath, this::handleVlcNotFound);
    }

    private void handleFoundVlcPath(File parentDirectory) {
        discoveredVlcPath = parentDirectory.getAbsolutePath();
        playerManagerService.register(new VlcPlayer(vlcPlayerService));
        log.info("VLC external player has been found at {}", discoveredVlcPath);
    }

    private void handleVlcNotFound() {
        log.debug("VLC external player was not found");
    }

    private boolean containsVlcExecutable(File directory) {
        log.trace("Checking if {} contains VLC executable", directory.getAbsolutePath());
        return Optional.ofNullable(directory.listFiles())
                .stream()
                .flatMap(Arrays::stream)
                .anyMatch(file -> file.getName().startsWith(FILENAME));
    }
}
