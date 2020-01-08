package com.github.yoep.popcorn.trakt;

import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.TraktSettings;
import com.github.yoep.popcorn.trakt.models.WatchedMovie;
import com.github.yoep.popcorn.trakt.models.WatchedShow;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.scheduling.annotation.Async;
import org.springframework.security.oauth2.client.OAuth2RestOperations;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestClientException;
import org.springframework.web.util.UriComponentsBuilder;

import javax.annotation.PostConstruct;
import java.util.List;
import java.util.concurrent.CompletableFuture;

import static java.util.Arrays.asList;

@Slf4j
@Service
@RequiredArgsConstructor
public class TraktService {
    private final OAuth2RestOperations traktTemplate;
    private final PopcornProperties popcornProperties;
    private final SettingsService settingsService;
    private final TaskExecutor taskExecutor;

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
     * Forget the current authorized trakt user.
     * This will remove the access token from the settings.
     */
    public void forget() {
        log.trace("Forgetting the authorization of trakt.tv");
        getSettings().setAccessToken(null);
    }

    public List<WatchedMovie> getWatchedMovies() {
        log.trace("Retrieving watched movies from trakt.tv");
        return asList(getWatched("movies", WatchedMovie[].class));
    }

    public List<WatchedShow> getWatchedShow() {
        log.trace("Retrieving watched shows from trakt.tv");
        return asList(getWatched("shows", WatchedShow[].class));
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
        List<WatchedMovie> movies = getWatchedMovies();
    }

    //endregion

    //region Functions

    private <T> T getWatched(String item, Class<T> type) {
        String url = UriComponentsBuilder.fromUri(popcornProperties.getTrakt().getUrl())
                .path("/sync/watched/{item}")
                .buildAndExpand(item)
                .toUriString();

        return traktTemplate.getForEntity(url, type).getBody();
    }

    private TraktSettings getSettings() {
        return settingsService.getSettings().getTraktSettings();
    }

    //endregion
}
