package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import javafx.application.Platform;
import javafx.scene.web.WebView;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class AuthorizationComponentTest {
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;

    @Test
    void testInitialize_shouldStartAuthorizationProcess() throws TimeoutException {
        var authorizationUri = "http://my-authorizarion-url/";
        var component = new AuthorizationComponent(authorizationUri);
        Platform.runLater(() -> component.webView = new WebView());

        WaitForAsyncUtils.waitFor(2000, TimeUnit.MILLISECONDS, () -> component.webView != null);
        Platform.runLater(() -> component.initialize(url, resourceBundle));
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(1000, TimeUnit.MILLISECONDS, () -> authorizationUri.equals(component.webView.getEngine().getLocation()));
    }
}