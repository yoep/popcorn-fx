package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.controllers.components.PlayerControlsComponent;
import com.github.yoep.player.popcorn.controllers.components.PlayerHeaderComponent;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

/**
 * The playback event service is for handling the events triggered by the playback.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlaybackEventService implements PlaybackListener, PlayerListener {
    private final PopcornPlayer player;
    private final VideoService videoService;
    private final PlayerHeaderComponent playerHeader;
    private final PlayerControlsComponent playerControls;

    //region PlaybackListener

    @Override
    public void onPlay(PlayRequest request) {
        playerHeader.updateTitle(request.getTitle().orElse(null));
        playerHeader.updateQuality(request.getQuality().orElse(null));
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
        // no-op
    }

    @Override
    public void onStop() {
        // no-op
    }


    //endregion

    //region PlayerListener

    @Override
    public void onDurationChanged(long newDuration) {
        onPlayerDurationChanged(newDuration);
    }

    @Override
    public void onTimeChanged(long newTime) {
        onPlayerTimeChanged(newTime);
    }

    @Override
    public void onStateChanged(PlayerState newState) {
        onPlayerStateChanged(newState);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeListeners();
    }

    //endregion

    //region Functions

    private void initializeListeners() {
        player.addListener(this);
        videoService.addListener(this);
    }

    private void onPlayerDurationChanged(Long duration) {
        playerControls.updateDuration(duration);
    }

    private void onPlayerTimeChanged(Long time) {
        playerControls.updateTime(time);
    }

    private void onPlayerStateChanged(PlayerState newState) {
        playerControls.updatePlaybackState(newState != PlayerState.PAUSED);
    }

    //endregion
}
