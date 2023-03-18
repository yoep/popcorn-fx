package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import javafx.application.Platform;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Objects;

@Slf4j
@Service
@RequiredArgsConstructor
public class DetailsComponentService extends AbstractListenerService<DetailsComponentListener> {
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final OptionsService optionsService;
    private final EventPublisher eventPublisher;
    private final SubtitlePickerService subtitlePickerService;

    private final FavoriteEventCallback favoriteEventCallback = createFavoriteCallback();
    private final WatchedEventCallback watchedEventCallback = createWatchedCallback();

    private Media lastShownMediaItem;

    public boolean isWatched(Media media) {
        return watchedService.isWatched(media);
    }

    public boolean isLiked(Media media) {
        return favoriteService.isLiked(media);
    }

    public boolean isTvMode() {
        return optionsService.isTvMode();
    }

    public void updateWatchedStated(Media media, boolean isWatched) {
        Objects.requireNonNull(media, "media cannot be null");
        if (isWatched) {
            watchedService.addToWatchList(media);
        } else {
            watchedService.removeFromWatchList(media);
        }
    }

    public void toggleWatchedState() {
        if (lastShownMediaItem == null) {
            log.warn("Unable to update watch state, media item is unknown");
            return;
        }

        updateWatchedStated(lastShownMediaItem, !watchedService.isWatched(lastShownMediaItem));
    }

    public void toggleLikedState() {
        if (lastShownMediaItem == null) {
            log.warn("Unable to update liked state, media item is unknown");
            return;
        }

        if (favoriteService.isLiked(lastShownMediaItem)) {
            favoriteService.removeFromFavorites(lastShownMediaItem);
        } else {
            favoriteService.addToFavorites(lastShownMediaItem);
        }
    }

    public void onCustomSubtitleSelected(Runnable onCancelled) {
        Platform.runLater(() -> {
            var subtitleInfo = subtitlePickerService.pickCustomSubtitle();

            // if a custom subtitle was picked by the user, update the subtitle with the custom subtitle
            // otherwise, the subtitle pick was cancelled and we need to reset the selected language to disabled
            subtitleInfo.ifPresentOrElse(
                    e -> {
                    },
                    onCancelled::run);
        });
    }

    @PostConstruct
    void init() {
        favoriteService.registerListener(favoriteEventCallback);
        watchedService.registerListener(watchedEventCallback);
        eventPublisher.register(ShowDetailsEvent.class, event -> {
            if (event instanceof ShowMovieDetailsEvent movieEvent) {
                lastShownMediaItem = movieEvent.getMedia();
            } else if (event instanceof ShowSerieDetailsEvent serieEvent) {
                lastShownMediaItem = serieEvent.getMedia();
            } else {
                log.warn("Unknown details events received, event: {}", event.getClass().getSimpleName());
            }
            return event;
        });
    }

    private FavoriteEventCallback createFavoriteCallback() {
        return event -> {
            switch (event.tag) {
                case LikedStateChanged -> {
                    var stateChanged = event.getUnion().getLiked_state_changed();
                    invokeListeners(e -> e.onLikedChanged(stateChanged.getNewState()));
                }
            }
        };
    }

    private WatchedEventCallback createWatchedCallback() {
        return event -> {
            switch (event.tag) {
                case WatchedStateChanged -> {
                    var stateChanged = event.getUnion().getWatched_state_changed();
                    invokeListeners(e -> e.onWatchChanged(stateChanged.getImdbId(), stateChanged.getNewState()));
                }
            }
        };
    }
}
