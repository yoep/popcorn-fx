package com.github.yoep.popcorn.ui.tracking;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.backend.media.tracking.AuthorizationOpenCallback;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.AuthorizationComponent;
import javafx.application.Platform;
import javafx.scene.layout.Pane;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class EmbeddedAuthorization implements AuthorizationOpenCallback {
    private final ViewLoader viewLoader;
    private final LocaleText localeText;

    @Override
    public byte callback(String uri) {
        try {
            final var controller = new AuthorizationComponent(uri);

            Platform.runLater(() -> {
                var pane = viewLoader.<Pane>load("components/authorization.component.fxml", controller);

                viewLoader.showWindow(pane, controller, ViewProperties.builder()
                        .icon("icon.png")
                        .title(localeText.get(SettingsMessage.TRAKT_LOGIN_TITLE))
                        .resizable(false)
                        .dialog(true)
                        .build());
            });

            return 1;
        } catch (Exception ex) {
            log.error("Failed to authorize tracking provider, {}", ex.getMessage(), ex);
        }

        return 0;
    }
}
