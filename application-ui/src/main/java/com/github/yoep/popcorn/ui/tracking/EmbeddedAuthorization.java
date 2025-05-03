package com.github.yoep.popcorn.ui.tracking;

import com.github.yoep.popcorn.backend.media.tracking.TrackingAuthorization;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.ViewProperties;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.AuthorizationComponent;
import javafx.application.Platform;
import javafx.scene.layout.Pane;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@ToString
@EqualsAndHashCode
public class EmbeddedAuthorization implements TrackingAuthorization {
    static final String AUTHORIZATION_COMPONENT_VIEW = "components/authorization.component.fxml";

    private final ViewLoader viewLoader;
    private final LocaleText localeText;

    public EmbeddedAuthorization(ViewLoader viewLoader, LocaleText localeText) {
        this.viewLoader = viewLoader;
        this.localeText = localeText;
    }

    @Override
    public void open(String authorizationUri) {
        try {
            final var controller = new AuthorizationComponent(authorizationUri);

            Platform.runLater(() -> {
                var pane = viewLoader.<Pane>load(AUTHORIZATION_COMPONENT_VIEW, controller);

                viewLoader.showWindow(pane, controller, ViewProperties.builder()
                        .icon("icon.png")
                        .title(localeText.get(SettingsMessage.TRAKT_LOGIN_TITLE))
                        .resizable(false)
                        .dialog(true)
                        .build());
            });
        } catch (Exception ex) {
            log.error("Failed to authorize tracking provider, {}", ex.getMessage(), ex);
        }
    }
}
