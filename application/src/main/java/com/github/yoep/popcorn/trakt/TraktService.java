package com.github.yoep.popcorn.trakt;

import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.TraktSettings;
import lombok.RequiredArgsConstructor;
import org.springframework.scheduling.annotation.Async;
import org.springframework.security.oauth2.client.OAuth2RestOperations;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestClientException;
import org.springframework.web.util.UriComponentsBuilder;

import java.util.concurrent.CompletableFuture;

@Service
@RequiredArgsConstructor
public class TraktService {
    private final OAuth2RestOperations traktTemplate;
    private final PopcornProperties popcornProperties;
    private final SettingsService settingsService;

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
        try {
            getWatched();
            return CompletableFuture.completedFuture(true);
        } catch (RestClientException ex) {
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
        getSettings().setAccessToken(null);
    }

    public String getWatched() {
        String url = UriComponentsBuilder.fromUri(popcornProperties.getTrakt().getUrl())
                .path("/sync/watched/movies")
                .toUriString();

        return traktTemplate.getForEntity(url, String.class).getBody();
    }

    private TraktSettings getSettings() {
        return settingsService.getSettings().getTraktSettings();
    }
}
