package com.github.yoep.popcorn.ui.trakt;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.TraktSettings;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.messages.TraktMessage;
import com.github.yoep.popcorn.ui.trakt.models.*;
import javafx.application.Platform;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.collections4.CollectionUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.http.ResponseEntity;
import org.springframework.scheduling.annotation.Async;
import org.springframework.security.oauth2.client.OAuth2RestOperations;
import org.springframework.security.oauth2.client.resource.OAuth2AccessDeniedException;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestClientException;
import org.springframework.web.util.UriComponentsBuilder;

import javax.annotation.PostConstruct;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

import static java.util.Arrays.asList;

@Slf4j
@Service
@RequiredArgsConstructor
public class TraktService {
    private final OAuth2RestOperations traktTemplate;
    private final PopcornProperties properties;
    private final ApplicationConfig settingsService;
    private final WatchedService watchedService;
    private final TaskExecutor taskExecutor;
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;

    //region Methods

    /**
     * Check if the user is authorized for trakt.
     *
     * @return Returns true if the user already has an access token for trakt, else false.
     */
    public boolean isAuthorized() {
        return getSettings().getAccessToken().isPresent();
    }

    /**
     * Authorize the user against the trakt API.
     * This method will automatically update the trakt settings with the access token on success.
     *
     * @return Returns true if the user has been authenticated, else false.
     */
    @Async
    public CompletableFuture<Boolean> authorize() {
        log.trace("Trying to authorize on trakt.tv");
        try {
            getWatchedMovies();
            log.debug("Trakt.tv authorization succeeded");
            return CompletableFuture.completedFuture(true);
        } catch (RestClientException ex) {
            log.warn("Trakt.tv authorization failed");
            return CompletableFuture.completedFuture(false);
        } catch (Exception ex) {
            return CompletableFuture.failedFuture(ex);
        }
    }

    /**
     * Get the watchlist of the authorized user.
     *
     * @return Returns the watchlist of the user.
     */
    @Async
    public CompletableFuture<List<WatchListItem>> getWatchlist() {
        var uri = UriComponentsBuilder.fromUri(properties.getTrakt().getUrl())
                .path("sync/watchlist")
                .build(Collections.emptyMap());

        log.trace("Retrieving the user's watchlist at \"{}\"", uri.toString());
        ResponseEntity<WatchListItem[]> response = traktTemplate.getForEntity(uri, WatchListItem[].class);

        return Optional.ofNullable(response.getBody())
                .map(e -> {
                    log.trace("Retrieved {} items from the user's watchlist", e.length);
                    return CompletableFuture.completedFuture(asList(e));
                })
                .orElseGet(() -> {
                    log.trace("Failed to retrieve the user's watchlist, body is null");
                    return CompletableFuture.failedFuture(new TraktException("Failed to retrieve watchlist, response body is null"));
                });
    }

    /**
     * Forget the current authorized trakt user.
     * This will remove the access token from the settings.
     */
    public void forget() {
        log.trace("Forgetting the authorization of trakt.tv");
        getSettings().setAccessToken(null);
    }

    //endregion

    //region PostConstructor

    @PostConstruct
    private void init() {
        // check if the user is authorized
        // if so, run a synchronization at the start of the application
        if (isAuthorized())
            taskExecutor.execute(this::synchronize);
    }

    private void synchronize() {
        log.debug("Starting Trakt.tv synchronisation");
        try {
            syncMovies();
        } catch (OAuth2AccessDeniedException ex) {
            handleAccessDenied(ex);
        } catch (Exception ex) {
            log.error("Failed to sync trakt.tv movies, " + ex.getMessage(), ex);
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(TraktMessage.SYNCHRONIZATION_FAILED)));
        }

        try {
            syncShows();
        } catch (OAuth2AccessDeniedException ex) {
            // no-op
        } catch (Exception ex) {
            log.error("Failed to sync trakt.tv shows, " + ex.getMessage(), ex);
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(TraktMessage.SYNCHRONIZATION_FAILED)));
        }
    }

    //endregion

    //region Functions

    private List<WatchedMovie> getWatchedMovies() {
        log.trace("Retrieving watched movies from trakt.tv");
        return asList(getWatched("movies", WatchedMovie[].class));
    }

    private List<WatchedShow> getWatchedShows() {
        log.trace("Retrieving watched shows from trakt.tv");
        return asList(getWatched("shows", WatchedShow[].class));
    }

    private <T> T getWatched(String item, Class<T> type) {
        var url = UriComponentsBuilder.fromUri(properties.getTrakt().getUrl())
                .path("/sync/watched/{item}")
                .buildAndExpand(item)
                .toUriString();

        log.trace("Requesting watched trakt.tv items from {}", url);
        return traktTemplate.getForEntity(url, type).getBody();
    }

    private void syncMovies() {
        var movies = getWatchedMovies();

        // synchronize movies locally
        try {
            log.trace("Synchronizing {} trakt.tv movies to local DB", movies.size());
            movies.stream()
                    .map(WatchedMovie::getMovie)
                    .map(e -> {
                        var movie = new MovieOverview();
                        movie.imdbId = e.getId();
                        movie.title = e.getTitle();
                        movie.year = "" + e.getYear();
                        movie.images = new Images.ByValue();
                        return movie;
                    })
                    .forEach(watchedService::addToWatchList);
            log.debug("Trakt.tv movies to local DB sync completed");
        } catch (Exception ex) {
            log.error("Failed to synchronize movies to local DB with error: " + ex.getMessage(), ex);
        }

        // synchronize movies to remote
        try {
            log.trace("Gathering movies that need to be synced to trakt.tv");
            List<String> moviesToSync = watchedService.getWatchedMovies().stream()
                    .filter(key -> movies.stream()
                            .noneMatch(movie -> movie.getMovie().getId().equals(key)))
                    .collect(Collectors.toList());

            if (CollectionUtils.isNotEmpty(moviesToSync)) {
                addMoviesToTraktWatchlist(moviesToSync);
                log.debug("Local DB movies to trakt.tv sync completed");
            }
        } catch (Exception ex) {
            log.error("Failed to synchronize movies to trakt.tv with error: " + ex.getMessage(), ex);
        }

        log.info("Trakt.tv movie synchronisation completed");
    }

    private void syncShows() {
        var shows = getWatchedShows();

        // synchronize shows to remote
        try {
            log.trace("Gathering shows that need to be synced to trakt.tv");
            List<String> showsToSync = watchedService.getWatchedShows().stream()
                    .filter(key -> shows.stream()
                            .noneMatch(show -> key.equals(String.valueOf(show.getShow().getIds().getTvdb()))))
                    .collect(Collectors.toList());

            if (CollectionUtils.isNotEmpty(showsToSync)) {
                addShowsToTraktWatchlist(showsToSync);
                log.debug("Local DB shows to trakt.tv sync completed");
            }
        } catch (Exception ex) {
            log.error("Failed to synchronize shows to trakt.tv with error: " + ex.getMessage(), ex);
        }

        log.info("Trakt.tv show synchronisation completed");
    }

    private void addMoviesToTraktWatchlist(List<String> keys) {
        log.trace("Synchronizing {} local DB movies to trakt.tv", keys.size());
        AddToWatchlistRequest request = AddToWatchlistRequest.builder()
                .movies(keys.stream()
                        .map(this::toMovie)
                        .collect(Collectors.toList()))
                .build();

        executeWatchlistRequest(request);
    }

    private void addShowsToTraktWatchlist(List<String> keys) {
        log.trace("Synchronizing {} local DB shows to trakt.tv", keys.size());
        AddToWatchlistRequest request = AddToWatchlistRequest.builder()
                .shows(keys.stream()
                        .map(this::toShow)
                        .collect(Collectors.toList()))
                .build();

        executeWatchlistRequest(request);
    }

    private void executeWatchlistRequest(AddToWatchlistRequest request) {
        String url = UriComponentsBuilder.fromUri(properties.getTrakt().getUrl())
                .path("/sync/watchlist")
                .toUriString();

        ResponseEntity<Void> response = traktTemplate.postForEntity(url, request, Void.class);
        log.debug("Trakt.tv responded with status code {}", response.getStatusCodeValue());
    }

    private TraktMovie toMovie(String key) {
        return TraktMovie.builder()
                .ids(TraktMovieIds.builder()
                        .imdb(key)
                        .build())
                .build();
    }

    private TraktShow toShow(String key) {
        return TraktShow.builder()
                .ids(TraktShowIds.builder()
                        .tvdb(Integer.parseInt(key))
                        .build())
                .build();
    }

    private TraktSettings getSettings() {
        return settingsService.getSettings().getTraktSettings();
    }

    private void handleAccessDenied(OAuth2AccessDeniedException ex) {
        var traktSettings = getSettings();
        log.warn(ex.getMessage(), ex);

        traktSettings.setAccessToken(null);

        Platform.runLater(() -> {
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(TraktMessage.TOKEN_EXPIRED)));

            authorize().whenComplete((successful, throwable) -> {
                if (throwable == null && successful) {
                    log.info("Successfully re-authenticated with Trakt.tv");
                    eventPublisher.publishEvent(new SuccessNotificationEvent(this, localeText.get(TraktMessage.AUTHENTICATION_SUCCESS)));
                } else {
                    eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(TraktMessage.AUTHENTICATION_FAILED)));
                }
            });
        });
    }

    //endregion
}
