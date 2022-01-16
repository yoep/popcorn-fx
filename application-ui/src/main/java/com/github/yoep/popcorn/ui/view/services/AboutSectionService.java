package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.view.listeners.AboutSectionListener;
import com.github.yoep.popcorn.ui.view.model.AboutDetail;
import javafx.collections.MapChangeListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

@Slf4j
@Service
@RequiredArgsConstructor
public class AboutSectionService extends AbstractListenerService<AboutSectionListener> {
    private final PlayerManagerService playerManagerService;

    /**
     * Update all information.
     * This will invoke all listeners with the latest known information.
     */
    public void updateAll() {
        onPlayersChanged(new ArrayList<>(playerManagerService.getPlayers()));
    }

    @PostConstruct
    void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        playerManagerService.playersProperty().addListener((MapChangeListener<? super String, ? super Player>) e -> onPlayersChanged(new ArrayList<>(e.getMap().values())));
    }

    private void onPlayersChanged(List<Player> players) {
        var details = players.stream()
                .map(e -> AboutDetail.builder()
                        .name(e.getName())
                        .state(AboutDetail.State.UNKNOWN)
                        .build())
                .collect(Collectors.toList());
        invokeListeners(e -> e.onPlayersChanged(details));
    }
}
