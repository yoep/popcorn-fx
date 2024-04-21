package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.info.PlayerInfoService;
import com.github.yoep.popcorn.ui.info.VideoInfoService;
import com.github.yoep.popcorn.ui.view.listeners.AboutSectionListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.util.List;

@Slf4j
@RequiredArgsConstructor
public class AboutSectionService extends AbstractListenerService<AboutSectionListener> {
    private final PlayerInfoService playerInfoService;
    private final VideoInfoService videoInfoService;

    /**
     * Update all information.
     * This will invoke all listeners with the latest known information.
     */
    public void updateAll() {
        onPlayersChanged(playerInfoService.getComponentDetails());
        onVideoPlayersChanged(videoInfoService.getComponentDetails());
    }

    @PostConstruct
    void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        playerInfoService.addListener(this::onPlayersChanged);
        videoInfoService.addListener(this::onVideoPlayersChanged);
    }

    private void onVideoPlayersChanged(List<ComponentInfo> componentInfos) {
        invokeListeners(e -> e.onVideoPlayersChanged(componentInfos));
    }

    private void onPlayersChanged(List<ComponentInfo> players) {
        invokeListeners(e -> e.onPlayersChanged(players));
    }
}
