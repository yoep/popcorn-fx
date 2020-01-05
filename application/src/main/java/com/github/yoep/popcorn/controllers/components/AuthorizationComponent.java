package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.web.WebEngine;
import javafx.scene.web.WebErrorEvent;
import javafx.scene.web.WebEvent;
import javafx.scene.web.WebView;
import javafx.stage.Stage;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
public class AuthorizationComponent extends ScaleAwareImpl implements Initializable {
    private boolean initialized;
    private AuthorizationRequest authorizationRequest;

    @FXML
    private WebView webView;

    //region Methods

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initialized = true;

        if (authorizationRequest != null)
            startAuthorization();
    }

    /**
     * Start the authorization process for the given authorization request.
     *
     * @param authorizationRequest The authorization request to execute.
     */
    public void startAuthorization(AuthorizationRequest authorizationRequest) {
        this.authorizationRequest = authorizationRequest;

        if (initialized)
            startAuthorization();
    }

    //endregion

    //region Functions

    private void startAuthorization() {
        WebEngine engine = webView.getEngine();

        engine.setUserAgent("Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/65.0.3325.181 Safari/537.36");
        engine.load(authorizationRequest.getAuthorizationUrl());
        engine.locationProperty().addListener((observable, oldValue, newValue) -> verifyIfRedirectIsCallback(newValue));
        engine.setOnError(this::handleEngineError);
        engine.setOnAlert(this::handleEngineAlert);
    }

    private void verifyIfRedirectIsCallback(String url) {
        if (url.contains(authorizationRequest.getRedirectUrl())) {
            authorizationRequest.onComplete(url);

            closeWindow();
        }
    }

    private void closeWindow() {
        Stage stage = (Stage) webView.getScene().getWindow();
        stage.close();

        this.authorizationRequest = null;
    }

    private void handleEngineError(WebErrorEvent webErrorEvent) {
        log.error(webErrorEvent.getMessage(), webErrorEvent.getException());
    }

    private void handleEngineAlert(WebEvent<String> event) {
        log.warn(event.toString());
    }

    //endregion
}
