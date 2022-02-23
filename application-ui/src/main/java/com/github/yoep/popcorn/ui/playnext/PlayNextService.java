package com.github.yoep.popcorn.ui.playnext;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.media.providers.MediaException;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.player.PlayerEventService;
import javafx.beans.property.ReadOnlyLongProperty;
import javafx.beans.property.ReadOnlyLongWrapper;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.ReadOnlyObjectWrapper;
import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Optional;
import java.util.stream.Collectors;

/**
 * The {@link PlayNextService} is responsible for determining if the playing next should be activated for the current playback.
 * This service listens on the {@link PlayMediaEvent} and bases itself around the {@link Media} type.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlayNextService {
    public static final String NEXT_EPISODE_PROPERTY = "nextEpisode";
    public static final String PLAYING_IN_PROPERTY = "playingIn";
    public static final int COUNTDOWN_FROM = 60;

    private final ApplicationEventPublisher eventPublisher;
    private final PlayerEventService playerEventService;
    private final PlayerManagerService playerManagerService;
    private final SettingsService settingsService;

    private final ReadOnlyObjectWrapper<NextEpisode> nextEpisode = new ReadOnlyObjectWrapper<>(this, NEXT_EPISODE_PROPERTY);
    private final ReadOnlyLongWrapper playingIn = new ReadOnlyLongWrapper(this, PLAYING_IN_PROPERTY, COUNTDOWN_FROM);

    private Show show;
    private String quality;
    private long duration;

    //region Properties

    /**
     * Get the next episode that should be played for the current playback.
     *
     * @return Returns the next episode if available, else {@link Optional#empty()}.
     */
    public Optional<NextEpisode> getNextEpisode() {
        return Optional.ofNullable(nextEpisode.get());
    }

    /**
     * Get the next episode property.
     *
     * @return Returns the next episode property.
     */
    public ReadOnlyObjectProperty<NextEpisode> nextEpisodeProperty() {
        return nextEpisode.getReadOnlyProperty();
    }

    /**
     * Get the playing in value of the next episode.
     *
     * @return Returns the current playing in value.
     */
    public long getPlayingIn() {
        return playingIn.get();
    }

    /**
     * Get the playing in property for the next episode.
     *
     * @return Returns the playing in property.
     */
    public ReadOnlyLongProperty playingInProperty() {
        return playingIn.getReadOnlyProperty();
    }

    //endregion

    //region Methods

    /**
     * Play the next episode now and stop the playing in countdown.
     */
    public void playNextEpisodeNow() {
        onPlayNextEpisode();
    }

    /**
     * Stop the play next event.
     */
    public void stop() {
        playerManagerService.getActivePlayer()
                .ifPresent(Player::stop);
        reset();
    }

    @EventListener
    public void onPlayVideo(PlayVideoEvent event) {
        // check if the play next option is enabled
        // if not, ignore this event
        if (isPlayNextDisabled()) {
            reset();
            return;
        }

        if (PlayMediaEvent.class.isAssignableFrom(event.getClass())) {
            var mediaEvent = (PlayMediaEvent) event;

            onPlayMedia(mediaEvent);
        } else {
            reset();
        }
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeVideoPlayerListeners();
    }

    private void initializeVideoPlayerListeners() {
        playerEventService.addListener(new PlayerListener() {
            @Override
            public void onDurationChanged(long newDuration) {
                PlayNextService.this.onDurationChanged(newDuration);
            }

            @Override
            public void onTimeChanged(long newTime) {
                PlayNextService.this.onTimeChanged(newTime);
            }

            @Override
            public void onStateChanged(PlayerState newState) {
                // no-op
            }
        });
    }

    //endregion

    //region Functions

    private void onTimeChanged(long time) {
        // check if the next episode to be played is known and the play next option is enabled
        // if not, ignore this event
        if (getNextEpisode().isEmpty() || isPlayNextDisabled()) {
            return;
        }

        var remainingTime = (duration - time) / 1000;

        if (remainingTime <= COUNTDOWN_FROM) {
            playingIn.set(remainingTime);

            if (remainingTime <= 1) {
                onPlayNextEpisode();
            }
        }
    }

    private void onDurationChanged(long newValue) {
        this.duration = newValue;
    }

    private void onPlayMedia(PlayMediaEvent event) {
        var media = event.getMedia();

        // check if the current media is a show/serie
        // if not, ignore the update of information
        if (!isShow(media)) {
            reset();
            return;
        }

        // remember the show item for later use
        this.show = (Show) media;

        var episode = event.getSubMediaItem()
                .map(e -> (Episode) e)
                .orElseThrow(() -> new MediaException("Expected an episode media item to be present"));
        var sortedEpisodes = show.getEpisodes().stream()
                .sorted()
                .collect(Collectors.toList());
        var nextEpisodeIndex = sortedEpisodes.indexOf(episode) + 1;

        if (nextEpisodeIndex < sortedEpisodes.size()) {
            setNextEpisode(sortedEpisodes.get(nextEpisodeIndex), event.getQuality());
        } else {
            reset();
        }
    }

    private void onPlayNextEpisode() {
        var nextEpisode = getNextEpisode();

        // check if the next episode is known
        // if not, ignore this action
        if (nextEpisode.isEmpty()) {
            log.warn("Unable to play next episode, nex episode is unwknown");
            return;
        }

        var mediaTorrentInfo = nextEpisode.get().getEpisode().getTorrents().get(quality);

        // stop the video playback
        playerManagerService.getActivePlayer()
                .ifPresent(Player::stop);

        // start loading the next episode
        eventPublisher.publishEvent(LoadMediaTorrentEvent.builder()
                .source(this)
                .torrent(mediaTorrentInfo)
                .media(nextEpisode.get().getShow())
                .subItem(nextEpisode.get().getEpisode())
                .quality(quality)
                .subtitle(null)
                .build());
    }

    private void setNextEpisode(Episode nextEpisode, String quality) {
        this.nextEpisode.set(new NextEpisode(show, nextEpisode));
        this.quality = quality;
    }

    private void reset() {
        this.show = null;
        this.quality = null;
        this.duration = 0;
        this.nextEpisode.set(null);
    }

    private boolean isPlayNextDisabled() {
        var settings = settingsService.getSettings();

        return !settings.getPlaybackSettings().isAutoPlayNextEpisodeEnabled();
    }

    private boolean isShow(Media media) {
        return media instanceof Show;
    }

    //endregion

    @Data
    @AllArgsConstructor
    public static class NextEpisode {
        private final Show show;
        private final Episode episode;
    }
}
