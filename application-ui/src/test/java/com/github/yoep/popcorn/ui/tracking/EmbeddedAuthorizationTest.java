package com.github.yoep.popcorn.ui.tracking;

import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.AuthorizationComponent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class EmbeddedAuthorizationTest {
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private LocaleText localeText;
    @InjectMocks
    private EmbeddedAuthorization authorization;

    @Test
    void testOpen() {
        var authorizationUri = "https://my-tracking-provider.com/authorizationUri";
        when(viewLoader.load(isA(String.class), isA(Object.class))).thenReturn(new Pane());
        when(localeText.get(SettingsMessage.TRAKT_LOGIN_TITLE)).thenReturn("Trakt authorization");

        authorization.open(authorizationUri);
        WaitForAsyncUtils.waitForFxEvents();

        verify(viewLoader).load(eq(EmbeddedAuthorization.AUTHORIZATION_COMPONENT_VIEW), isA(AuthorizationComponent.class ));
        verify(localeText).get(SettingsMessage.TRAKT_LOGIN_TITLE);
    }
}