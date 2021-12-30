package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerHeaderService extends AbstractListenerService<PlayerHeaderListener> {
    private final PopcornPlayer player;
    private final VideoService videoService;

    private final PlaybackListener listener = createListener();

    public void stop() {
        player.stop();
    }

    @PostConstruct
    void init() {
        videoService.addListener(listener);
    }

    private void onPlayRequest(PlayRequest request) {
        invokeListeners(e -> e.onTitleChanged(request.getTitle().orElse("Unknown")));
        invokeListeners(e -> e.onQualityChanged(request.getQuality().orElse(null)));
    }

    private PlaybackListener createListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(PlayRequest request) {
                onPlayRequest(request);
            }
        };
    }
}
