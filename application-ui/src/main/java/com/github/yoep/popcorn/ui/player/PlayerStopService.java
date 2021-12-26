package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayTorrentEvent;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Optional;

/**
 * The player stop service is responsible for handling the stopped event sent by the player.
 * It will translate the {@link com.github.yoep.popcorn.backend.adapters.player.state.PlayerState#STOPPED} to a {@link PlayerStoppedEvent}.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerStopService {
    private final PlayerEventService playerEventService;
    private final TorrentStreamService torrentStreamService;
    private final ApplicationEventPublisher eventPublisher;

    private Media media;
    private String quality;
    private String url;
    private Long time;
    private Long duration;
    private PlayerState lastKnownState;

    //region Methods

    @EventListener
    public void onPlayMedia(PlayMediaEvent event) {
        this.media = event.getMedia();
        this.quality = event.getQuality();
        this.url = event.getUrl();
    }

    @EventListener
    public void onPlayTorrent(PlayTorrentEvent event) {
        this.url = event.getUrl();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        playerEventService.addListener(new PlayerListener() {
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
                if (newState == PlayerState.STOPPED) {
                    onPlayerStopped();
                } else {
                    lastKnownState = newState;
                }
            }
        });
    }

    //endregion

    //region Functions

    private void reset() {
        this.media = null;
        this.quality = null;
        this.time = null;
        this.duration = null;
        this.lastKnownState = null;
    }

    private void onPlayerDurationChanged(long duration) {
        // some players will update the duration to 0 when they're being stopped
        // because of this, the onPlayerStopped will show incorrect behavior
        // so to avoid this, we don't update the duration to 0 when we receive a duration of 0
        if (duration > 0 || (this.duration == null && duration == 0))
            this.duration = duration;
    }

    private void onPlayerTimeChanged(long time) {
        this.time = time;
    }

    private void onPlayerStopped() {
        if (!isAllowedToClose())
            return;

        // close the player
        eventPublisher.publishEvent(new ClosePlayerEvent(this));

        log.trace("Publishing player stopped event with info: [time: {}, duration: {}]", time, duration);
        eventPublisher.publishEvent(PlayerStoppedEvent.builder()
                .source(this)
                .url(url)
                .media(media)
                .quality(quality)
                .time(Optional.ofNullable(time)
                        .orElse(PlayerStoppedEvent.UNKNOWN))
                .duration(Optional.ofNullable(duration)
                        .orElse(PlayerStoppedEvent.UNKNOWN))
                .build());
        torrentStreamService.stopAllStreams();

        // reset the current known media information
        reset();
    }

    private boolean isAllowedToClose() {
        // check the last known state of the player
        // if the state is error, we always allow it to close
        if (lastKnownState == PlayerState.ERROR) {
            return true;
        }

        // check if the duration is not 0 for the active player
        // if so, don't close the player and wait
        // the playback of youtube videos in VLC will report a STOPPED event before actually starting the video playback
        // this causes the player to instantly close before the actual video playback has started
        return Optional.ofNullable(duration)
                .map(e -> e != 0)
                .orElse(false);
    }

    //endregion
}
