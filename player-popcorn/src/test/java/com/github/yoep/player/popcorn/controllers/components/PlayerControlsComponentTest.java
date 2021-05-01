package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.player.popcorn.controls.ProgressSliderControl;
import com.github.yoep.player.popcorn.services.PlaybackService;
import javafx.scene.control.Label;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.stage.Stage;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.framework.junit5.ApplicationTest;
import org.testfx.util.WaitForAsyncUtils;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.testfx.assertions.api.Assertions.assertThat;


class PlayerControlsComponentTest extends ApplicationTest {
    private PlaybackService playbackService;
    private Icon playPauseIcon;
    private Label durationLabel;
    private Label timeLabel;
    private ProgressSliderControl playProgress;
    private PlayerControlsComponent controller;

    @Override
    public void start(Stage stage) {
        playbackService = mock(PlaybackService.class);
        playPauseIcon = mock(Icon.class);
        durationLabel = new Label();
        timeLabel = new Label();
        playProgress = mock(ProgressSliderControl.class);

        controller = new PlayerControlsComponent(playbackService);
        controller.playPauseIcon = playPauseIcon;
        controller.durationLabel = durationLabel;
        controller.timeLabel = timeLabel;
        controller.playProgress = playProgress;
        WaitForAsyncUtils.waitForFxEvents(100);
    }

    @Test
    void testOnPlayPauseClicked_whenInvoked_shouldConsumeTheEvent() {
        var event = createMouseEvent();

        controller.onPlayPauseClicked(event);

        assertTrue(event.isConsumed());
    }

    @Test
    void testOnPlayPauseClicked_whenInvoked_shouldToggleThePlayback() {
        var event = createMouseEvent();

        controller.onPlayPauseClicked(event);

        verify(playbackService).togglePlayerPlaybackState();
    }

    @Test
    void testUpdatePlaybackState_whenIsPlayingIsTrue_shouldUpdateIconToPause() {
        controller.updatePlaybackState(true);

        FxAssert.verifyThat(playPauseIcon, e -> e.getText().equals(Icon.PAUSE_UNICODE));
    }

    @Test
    void testUpdatePlaybackState_whenIsPlayingIsFalse_shouldUpdateIconToPlay() {
        controller.updatePlaybackState(false);

        FxAssert.verifyThat(playPauseIcon, e -> e.getText().equals(Icon.PLAY_UNICODE));
    }

    @Test
    void testUpdateDuration_whenDurationIsOneMinute_shouldUpdateLabelWithExpectedDuration() {
        var duration = 60000L;
        var expectedResult = "01:00";

        controller.updateDuration(duration);

        WaitForAsyncUtils.waitForFxEvents(100);
        assertThat(durationLabel).hasText(expectedResult);
    }

    @Test
    void testUpdateTime_whenDurationIsOneMinuteAndThirtySeconds_shouldUpdateLabelWithExpectedDuration() {
        var time = 90000L;
        var expectedResult = "01:30";

        controller.updateTime(time);

        WaitForAsyncUtils.waitForFxEvents(100);
        assertThat(timeLabel).hasText(expectedResult);
    }

    private MouseEvent createMouseEvent() {
        return new MouseEvent(MouseEvent.ANY, 0.0, 0.0, 0.0, 0.0, MouseButton.PRIMARY, 0, false, false, false, false, false, false, false, false, false,
                false, null);
    }
}
