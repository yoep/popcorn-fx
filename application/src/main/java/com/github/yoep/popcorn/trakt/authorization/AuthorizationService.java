package com.github.yoep.popcorn.trakt.authorization;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.view.controllers.desktop.components.AuthorizationComponent;
import com.github.yoep.popcorn.view.controllers.desktop.components.AuthorizationRequest;
import com.github.yoep.popcorn.messages.SettingsMessage;
import javafx.application.Platform;
import javafx.scene.layout.Pane;
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
    private final LocaleText localeText;

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
        final var completableFuture = new CompletableFuture<String>();
        final var controller = new AuthorizationComponent();

        Platform.runLater(() ->{
            Pane pane = viewLoader.load("components/authorization.component.fxml", controller);

            controller.startAuthorization(new AuthorizationRequest() {
                @Override
                public String getAuthorizationUrl() {
                    return AuthorizationService.this.getRedirectUrl(userRedirectRequired);
                }

                @Override
                public String getRedirectUrl() {
                    return expectedRedirectUri;
                }

                @Override
                public void onComplete(String url) {
                    completableFuture.complete(authorize(url));
                }
            });

            viewLoader.showWindow(pane, controller, ViewProperties.builder()
                    .icon("icon.png")
                    .title(localeText.get(SettingsMessage.TRAKT_LOGIN_TITLE))
                    .resizable(false)
                    .dialog(true)
                    .build());
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
        return builder.toUriString();
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

    //endregion
}
