package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.stereotype.Service;

@Slf4j
@Service
public record PlayerExternalComponentService(PlayerManagerService playerManagerService, ApplicationEventPublisher eventPublisher) {
    public void togglePlaybackState() {
        playerManagerService.getActivePlayer()
                .ifPresent(e -> {
                    if (e.getState() == PlayerState.PAUSED) {
                        e.resume();
                    } else {
                        e.pause();
                    }
                });
    }

    public void closePlayer() {
        playerManagerService.getActivePlayer()
                .ifPresent(Player::stop);
        eventPublisher.publishEvent(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));
    }
}
