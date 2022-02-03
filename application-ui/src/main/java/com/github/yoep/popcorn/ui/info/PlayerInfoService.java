package com.github.yoep.popcorn.ui.info;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.AbstractPlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
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
public class PlayerInfoService extends AbstractInfoService {
    private final PlayerManagerService playerManagerService;

    @PostConstruct
    void init() {
        playerManagerService.playersProperty().addListener((MapChangeListener<? super String, ? super Player>) e -> onPlayersChanged(new ArrayList<>(e.getMap().values())));
    }

    private void onPlayersChanged(List<Player> players) {
       updateComponents(players.stream()
                .map(this::createComponentDetails)
                .collect(Collectors.toList()));
    }

    private SimpleComponentDetails createComponentDetails(Player player) {
        var componentDetails = SimpleComponentDetails.builder()
                .name(player.getName())
                .description(player.getDescription())
                .state(mapToComponentState(player.getState()))
                .build();

        player.addListener(new AbstractPlayerListener() {
            @Override
            public void onStateChanged(PlayerState newState) {
                componentDetails.setState(mapToComponentState(newState));
            }
        });

        return componentDetails;
    }

    private static ComponentState mapToComponentState(PlayerState state) {
        switch (state) {
            case ERROR:
                return ComponentState.ERROR;
            case UNKNOWN:
                return ComponentState.UNKNOWN;
            default:
                return ComponentState.READY;
        }
    }
}
