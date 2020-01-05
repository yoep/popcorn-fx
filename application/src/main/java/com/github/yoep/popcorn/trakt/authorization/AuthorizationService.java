package com.github.yoep.popcorn.trakt.authorization;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.controllers.LoginController;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Async;
import org.springframework.security.oauth2.client.resource.UserRedirectRequiredException;
import org.springframework.stereotype.Service;
import org.springframework.web.util.UriComponentsBuilder;

import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
@RequiredArgsConstructor
public class AuthorizationService {
    private final ViewLoader viewLoader;
    private final LoginController loginController;

    //region Methods

    /**
     * Start the authorization process based on the given user redirect.
     *
     * @param userRedirectRequired Set the user redirect for the authorization process.
     * @param expectedRedirectUri  The expected redirect URI for the callback.
     * @return Returns the authorization code that has been returned.
     */
    @Async
    public CompletableFuture<String> startAuthorization(UserRedirectRequiredException userRedirectRequired, String expectedRedirectUri) {
        log.debug("User has not been authorized yet, starting authorization process");
        var completableFuture = new CompletableFuture<String>();

        openLoginDialog();
        loginController.startAuthorization(getRedirectUrl(userRedirectRequired), expectedRedirectUri)
                .whenComplete((url, throwable) -> {
                    if (throwable == null) {
                        completableFuture.complete(authorize(url));
                    } else {
                        completableFuture.completeExceptionally(new AccessTokenException("Authorization failed, " + throwable.getMessage(), throwable));
                    }
                });

        return completableFuture;
    }

    //endregion

    //region Functions

    private String authorize(String url) {
        Map<String, String> params = getParameters(url);

        if (params.containsKey("code")) {
            return params.get("code");
        } else {
            throw new AccessTokenException(params.getOrDefault("error", "Unable to obtain access code"));
        }
    }

    private String getRedirectUrl(UserRedirectRequiredException e) {
        String redirectUri = e.getRedirectUri();
        UriComponentsBuilder builder = UriComponentsBuilder
                .fromHttpUrl(redirectUri);
        Map<String, String> requestParams = e.getRequestParams();
        for (Map.Entry<String, String> param : requestParams.entrySet()) {
            builder.queryParam(param.getKey(), param.getValue());
        }

        if (e.getStateKey() != null) {
            builder.queryParam("state", e.getStateKey());
        }

        builder.queryParam("show_dialog", "true");
        return builder.build().encode().toUriString();
    }

    private Map<String, String> getParameters(String url) {
        Map<String, String> params = new HashMap<>();
        String[] parameters = url.substring(url.indexOf("?") + 1).split("&");

        for (String param : parameters) {
            String[] paramPair = param.split("=");
            params.put(paramPair[0], paramPair[1]);
        }

        return params;
    }

    private void openLoginDialog() {
        log.debug("Showing trakt.tv login window");
        viewLoader.showWindow("login.fxml", ViewProperties.builder()
                .icon("icon.png")
                .maximizable(false)
                .dialog(true)
                .build());
    }

    //endregion
}
