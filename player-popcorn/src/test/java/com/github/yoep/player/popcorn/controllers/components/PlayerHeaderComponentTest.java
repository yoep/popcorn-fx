package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.services.PlaybackService;
import javafx.scene.control.Label;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.stage.Stage;
import org.junit.jupiter.api.Test;
import org.testfx.framework.junit5.ApplicationTest;
import org.testfx.util.WaitForAsyncUtils;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.testfx.assertions.api.Assertions.assertThat;

class PlayerHeaderComponentTest extends ApplicationTest {
    private PlaybackService playbackService;
    private LocaleText localeText;
    private Label title;
    private PlayerHeaderComponent controller;

    @Override
    public void start(Stage stage) {
        playbackService = mock(PlaybackService.class);
        localeText = mock(LocaleText.class);
        title = new Label();

        controller = new PlayerHeaderComponent(playbackService, localeText);
        controller.title = title;

        WaitForAsyncUtils.waitForFxEvents(10);
    }

    @Test
    void testUpdateTitle_whenTitleIsUpdated_shouldSetTheTitleText() {
        var expectedTitle = "my video title";

        controller.updateTitle(expectedTitle);

        WaitForAsyncUtils.waitForFxEvents(100);
        assertThat(title).hasText(expectedTitle);
    }

    @Test
    void testClose_whenClicked_shouldStopThePlayback() {
        var event = createMouseEvent();

        controller.close(event);

        verify(playbackService).stop();
    }

    @Test
    void testClose_whenClicked_shouldConsumeTheEvent() {
        var event = createMouseEvent();

        controller.close(event);

        assertTrue(event.isConsumed());
    }

    private MouseEvent createMouseEvent() {
        return new MouseEvent(MouseEvent.ANY, 0.0, 0.0, 0.0, 0.0, MouseButton.PRIMARY, 0, false, false, false, false, false, false, false, false, false,
                false, null);
    }
}
