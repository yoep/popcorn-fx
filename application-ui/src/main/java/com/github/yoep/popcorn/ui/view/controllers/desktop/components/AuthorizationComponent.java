package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.ui.scale.PopcornScaleAware;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.web.WebEngine;
import javafx.scene.web.WebErrorEvent;
import javafx.scene.web.WebEvent;
import javafx.scene.web.WebView;
import javafx.stage.Stage;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.ResourceBundle;

@Slf4j
public class AuthorizationComponent extends PopcornScaleAware implements Initializable {
    static final String CALLBACK_HOST = "http://localhost";

    private final String authorizationUri;

    @FXML
    WebView webView;

    public AuthorizationComponent(String authorizationUri) {
        Objects.requireNonNull(authorizationUri, "authorizationUri cannot be null");
        this.authorizationUri = authorizationUri;
    }

    //region Methods

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        startAuthorization();
    }

    //endregion

    //region Functions

    private void startAuthorization() {
        WebEngine engine = webView.getEngine();

        engine.setUserAgent("Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/65.0.3325.181 Safari/537.36");
        engine.load(authorizationUri);
        engine.locationProperty().addListener((observable, oldValue, newValue) -> verifyIfRedirectIsCallback(newValue));
        engine.setOnError(this::handleEngineError);
        engine.setOnAlert(this::handleEngineAlert);
    }

    private void verifyIfRedirectIsCallback(String url) {
        if (url.startsWith(CALLBACK_HOST)) {
            closeWindow();
        }
    }

    private void closeWindow() {
        Stage stage = (Stage) webView.getScene().getWindow();
        stage.close();
    }

    private void handleEngineError(WebErrorEvent webErrorEvent) {
        log.error(webErrorEvent.getMessage(), webErrorEvent.getException());
    }

    private void handleEngineAlert(WebEvent<String> event) {
        log.warn(event.toString());
    }

    //endregion
}
