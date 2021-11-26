package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.popcorn.controllers.components.PlayerHeaderComponent;
import com.github.yoep.player.popcorn.controllers.sections.PopcornPlayerSectionController;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.video.adapter.VideoPlayer;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;

/**
 * The details service is responsible for handling the UI information of the player.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class DetailsService implements PlaybackListener {
    private final VideoService videoService;
    private final PopcornPlayerSectionController playerSectionController;
    private final PlayerHeaderComponent playerHeaderComponent;

    //region PlaybackListener

    @Override
    public void onPlay(PlayRequest request) {
        Assert.notNull(request, "request cannot be null");
        playerHeaderComponent.updateTitle(request.getTitle().orElse(""));
        playerHeaderComponent.updateQuality(request.getQuality().orElse(""));
    }

    @Override
    public void onResume() {
        // no-op
    }

    @Override
    public void onPause() {
        // no-op
    }

    @Override
    public void onSeek(long time) {
        // no-op
    }

    @Override
    public void onVolume(int volume) {

    }

    @Override
    public void onStop() {
        // no-op
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeVideoListener();
    }

    //endregion

    //region Functions

    private void initializeVideoListener() {
        videoService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> {
            onVideoPlayerChanged(newValue);
        });
    }

    private void onVideoPlayerChanged(VideoPlayer player) {
        if (player != null) {
            playerSectionController.setVideoView(player.getVideoSurface());
        }
    }

    //endregion
}
