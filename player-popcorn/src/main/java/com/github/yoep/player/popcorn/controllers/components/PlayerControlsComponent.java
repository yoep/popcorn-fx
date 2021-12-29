package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.controls.ProgressSliderControl;
import com.github.yoep.player.popcorn.messages.MediaMessage;
import com.github.yoep.player.popcorn.services.PlaybackService;
import com.github.yoep.player.popcorn.services.SubtitleEventService;
import com.github.yoep.player.popcorn.subtitles.controls.LanguageSelection;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.Objects;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerControlsComponent implements Initializable {
    private final PlaybackService playbackService;
    private final ScreenService screenService;
    private final LocaleText localeText;
    private final SubtitleService subtitleService;
    private final SubtitleEventService popcornSubtitleService;

    @FXML
    Icon playPauseIcon;
    @FXML
    Label timeLabel;
    @FXML
    ProgressSliderControl playProgress;
    @FXML
    Label durationLabel;
    @FXML
    LanguageSelection languageSelection;
    @FXML
    Icon fullscreenIcon;
    @FXML
    Pane subtitleSection;

    //region Methods

    public void updatePlaybackState(boolean isPlaying) {
        if (isPlaying) {
            Platform.runLater(() -> playPauseIcon.setText(Icon.PAUSE_UNICODE));
        } else {
            Platform.runLater(() -> playPauseIcon.setText(Icon.PLAY_UNICODE));
        }
    }

    public void updateDuration(Long duration) {
        Platform.runLater(() -> {
            durationLabel.setText(formatTime(duration));
            playProgress.setDuration(duration);
        });
    }

    public void updateTime(Long time) {
        Platform.runLater(() -> {
            timeLabel.setText(formatTime(time));

            if (!playProgress.isValueChanging())
                playProgress.setTime(time);
        });
    }

    public void updateFullscreenState(Boolean isFullscreen) {
        if (isFullscreen) {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.COMPRESS_UNICODE));
        } else {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.EXPAND_UNICODE));
        }
    }

    public void updateSubtitleVisibility(boolean isSubtitlesEnabled) {
        // update the visibility of the subtitles section
        Platform.runLater(() -> subtitleSection.setVisible(isSubtitlesEnabled));
    }

    public void updateAvailableSubtitles(List<SubtitleInfo> subtitles, SubtitleInfo subtitle) {
        Objects.requireNonNull(subtitles, "subtitles cannot be null");
        log.trace("Updating available subtitles to {}", subtitles.size());
        Platform.runLater(() -> {
            languageSelection.getItems().clear();
            languageSelection.getItems().addAll(subtitles);
            languageSelection.select(subtitle);
        });
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeSlider();
        initializeLanguageSelection();
    }

    private void initializeSlider() {
        playProgress.valueChangingProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                playbackService.pause();
            } else {
                playbackService.resume();
            }
        });
        playProgress.timeProperty().addListener((observableValue, oldValue, newValue) -> {
            if (playProgress.isValueChanging()) {
                playbackService.seek(newValue.longValue());
            }
        });

        playProgress.setOnMouseReleased(event -> setVideoTime(playProgress.getTime() + 1.0));
    }

    private void initializeLanguageSelection() {
        languageSelection.getListView().setCellFactory(param -> new ListCell<>() {
            @Override
            protected void updateItem(SubtitleInfo item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    if (item.isNone()) {
                        setText(localeText.get(MediaMessage.SUBTITLE_NONE));
                    } else {
                        setText(item.getLanguage().getNativeName());
                    }
                }
            }
        });
        languageSelection.addListener(this::onSubtitleChanged);
        subtitleService.activeSubtitleProperty().addListener((observable, oldValue, newValue) ->
                languageSelection.select(newValue.getSubtitleInfo().orElse(SubtitleInfo.none())));
    }

    public void reset() {
        Platform.runLater(() -> {
            playProgress.setTime(0);
            languageSelection.getItems().clear();
        });
    }

    //endregion

    //region Functions

    private String formatTime(long time) {
        return String.format("%02d:%02d",
                TimeUnit.MILLISECONDS.toMinutes(time),
                TimeUnit.MILLISECONDS.toSeconds(time) % 60);
    }

    private void setVideoTime(double time) {
        playProgress.setValueChanging(true);
        playProgress.setTime((long) time);
        playProgress.setValueChanging(false);
    }

    private void onSubtitleChanged(SubtitleInfo newValue) {
        popcornSubtitleService.setSubtitle(newValue);
    }

    private void onSubtitleSizeChanged(int pixelChange) {
        popcornSubtitleService.setSubtitleSize(popcornSubtitleService.getSubtitleSize() + pixelChange);
    }

    @FXML
    void onPlayPauseClicked(MouseEvent event) {
        event.consume();
        playbackService.togglePlayerPlaybackState();
    }

    @FXML
    void onFullscreenClicked(MouseEvent event) {
        event.consume();
        screenService.toggleFullscreen();
    }

    @FXML
    void onSubtitleSmaller() {
        onSubtitleSizeChanged(-4);
    }

    @FXML
    void onSubtitleLarger() {
        onSubtitleSizeChanged(4);
    }

    //endregion
}
