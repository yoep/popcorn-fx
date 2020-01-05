package com.github.yoep.popcorn.controllers;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.web.WebEngine;
import javafx.scene.web.WebErrorEvent;
import javafx.scene.web.WebEvent;
import javafx.scene.web.WebView;
import javafx.stage.Stage;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Controller
public class LoginControllerImpl extends ScaleAwareImpl implements LoginController, Initializable {
    private boolean initialized;

    private CompletableFuture<String> completableFuture;
    private String expectedRedirectUrl;
    private String url;

    @FXML
    private WebView webView;

    //region Methods

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initialized = true;

        if (completableFuture != null)
            startAuthorization();
    }

    @Override
    public CompletableFuture<String> startAuthorization(String url, String expectedRedirectUrl) {
        this.completableFuture = new CompletableFuture<>();
        this.expectedRedirectUrl = expectedRedirectUrl;
        this.url = url;

        if (initialized)
            startAuthorization();

        return completableFuture;
    }

    //endregion

    //region Functions

    private void startAuthorization() {
        WebEngine engine = webView.getEngine();

        engine.setUserAgent("Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/65.0.3325.181 Safari/537.36");
        engine.load(url);
        engine.locationProperty().addListener((observable, oldValue, newValue) -> verifyIfRedirectIsCallback(newValue));
        engine.setOnError(this::handleEngineError);
        engine.setOnAlert(this::handleEngineAlert);
    }

    private void verifyIfRedirectIsCallback(String url) {
        if (url.contains(expectedRedirectUrl)) {
            this.completableFuture.complete(url);

            closeWindow();
        }
    }

    private void closeWindow() {
        Stage stage = (Stage) webView.getScene().getWindow();
        stage.close();

        this.completableFuture = null;
    }

    private void handleEngineError(WebErrorEvent webErrorEvent) {
        log.error(webErrorEvent.getMessage(), webErrorEvent.getException());
    }

    private void handleEngineAlert(WebEvent<String> event) {
        log.warn(event.toString());
    }

    //endregion
}
