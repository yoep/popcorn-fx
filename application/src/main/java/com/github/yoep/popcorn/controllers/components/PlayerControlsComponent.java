package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.messages.MediaMessage;
import com.github.yoep.popcorn.providers.models.Episode;
import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.providers.models.Movie;
import com.github.yoep.popcorn.subtitle.SubtitleService;
import com.github.yoep.popcorn.subtitle.controls.LanguageSelection;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.Slider;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;

@Slf4j
@Component
@RequiredArgsConstructor
public class PlayerControlsComponent implements Initializable {
    private final List<PlayerControlsListener> listeners = new ArrayList<>();

    private final ActivityManager activityManager;
    private final SubtitleService subtitleService;
    private final LocaleText localeText;
    private final VideoPlayer videoPlayer;

    @FXML
    private Icon playPauseIcon;
    @FXML
    private Label timeLabel;
    @FXML
    private Label durationLabel;
    @FXML
    private Slider slider;
    @FXML
    private Pane subtitleSection;
    @FXML
    private LanguageSelection languageSelection;
    @FXML
    private Icon fullscreenIcon;

    private Media media;
    private SubtitleInfo subtitle;
    private Long time;
    private Long duration;

    //region Getters

    /**
     * Get the last known time of the video playback.
     *
     * @return Returns the last time if known, else null.
     */
    public Long getTime() {
        return time;
    }

    /**
     * Get the last known duration of the video.
     *
     * @return Returns the last duration if known, else null.
     */
    public Long getDuration() {
        return duration;
    }

    //endregion

    //region Methods

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSlider();
        initializeLanguageSelection();
    }

    public void addListener(PlayerControlsListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    public void removeListener(PlayerControlsListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }

    public void increaseVideoTime(double amount) {
        log.trace("Increasing video time with {}", amount);
        double newSliderValue = slider.getValue() + amount;
        double maxSliderValue = slider.getMax();

        if (newSliderValue > maxSliderValue)
            newSliderValue = maxSliderValue;

        setVideoTime(newSliderValue);
    }

    public void changePlayPauseState() {
        if (videoPlayer.getPlayerState() == PlayerState.PAUSED) {
            log.trace("Video player state is being changed to \"resume\"");
            videoPlayer.resume();
        } else {
            log.trace("Video player state is being changed to \"paused\"");
            videoPlayer.pause();
        }
    }

    public void toggleFullscreen() {
        log.trace("Toggling full screen mode");
        activityManager.register(new ToggleFullscreenActivity() {
        });
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeActivityListeners();
        initializeVideoPlayer();
    }

    private void initializeActivityListeners() {
        activityManager.register(PlayVideoActivity.class, this::onPlayVideo);
        activityManager.register(ClosePlayerActivity.class, activity -> reset());
        activityManager.register(FullscreenActivity.class, this::onFullscreenChanged);
    }

    private void initializeVideoPlayer() {
        videoPlayer.playerStateProperty().addListener((observable, oldValue, newValue) -> {
            switch (newValue) {
                case PLAYING:
                    Platform.runLater(() -> playPauseIcon.setText(Icon.PAUSE_UNICODE));
                    break;
                case PAUSED:
                    Platform.runLater(() -> playPauseIcon.setText(Icon.PLAY_UNICODE));
                    break;
            }
        });
        videoPlayer.timeProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> {
            long time = newValue.longValue();

            timeLabel.setText(formatTime(time));
            slider.setValue(time);
            this.time = time;
        }));
        videoPlayer.durationProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> {
            long duration = newValue.longValue();

            durationLabel.setText(formatTime(duration));
            slider.setMax(duration);
            this.duration = duration;
        }));
    }

    //endregion

    //region Functions

    private void initializeSlider() {
        slider.valueChangingProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                videoPlayer.pause();
            } else {
                videoPlayer.resume();
            }
        });
        slider.valueProperty().addListener((observableValue, oldValue, newValue) -> {
            if (slider.isValueChanging()) {
                videoPlayer.seek(newValue.longValue());
            }
        });
        slider.setOnMouseReleased(event -> setVideoTime(slider.getValue() + 1));
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
    }

    private void onPlayVideo(PlayVideoActivity activity) {
        Platform.runLater(() -> subtitleSection.setVisible(false));

        // check if the activity contains media information
        if (activity instanceof PlayMediaActivity) {
            var mediaActivity = (PlayMediaActivity) activity;
            onPlayMedia(mediaActivity);
        }
    }

    private void onPlayMedia(PlayMediaActivity activity) {
        this.media = activity.getMedia();

        Platform.runLater(() -> subtitleSection.setVisible(true));

        // set the subtitle for the playback
        this.subtitle = activity.getSubtitle()
                .orElse(SubtitleInfo.none());

        if (media instanceof Movie) {
            Movie movie = (Movie) activity.getMedia();
            subtitleService.retrieveSubtitles(movie).whenComplete(this::handleSubtitlesResponse);
        } else if (media instanceof Episode) {
            Episode episode = (Episode) activity.getMedia();

            subtitleService.retrieveSubtitles(episode.getShow(), episode).whenComplete(this::handleSubtitlesResponse);
        } else {
            log.error("Failed to retrieve subtitles, missing episode information");
        }
    }

    private void onFullscreenChanged(FullscreenActivity activity) {
        if (activity.isFullscreen()) {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.COLLAPSE_UNICODE));
        } else {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.EXPAND_UNICODE));
        }
    }

    private void onSubtitleChanged(SubtitleInfo newValue) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onSubtitleChanged(newValue));
        }
    }

    private void onSubtitleSizeChanged(int pixelChange) {
        synchronized (listeners) {
            listeners.forEach(e -> e.onSubtitleSizeChanged(pixelChange));
        }
    }

    private void setVideoTime(double time) {
        slider.setValueChanging(true);
        slider.setValue(time);
        slider.setValueChanging(false);
    }

    private void handleSubtitlesResponse(final List<SubtitleInfo> subtitles, Throwable throwable) {
        if (throwable == null) {
            final SubtitleInfo subtitle = this.subtitle != null ? this.subtitle : subtitleService.getDefault(subtitles);

            Platform.runLater(() -> {
                languageSelection.getItems().clear();
                languageSelection.getItems().addAll(subtitles);
                languageSelection.select(subtitle);
            });
        } else {
            log.error("Failed to retrieve subtitles, " + throwable.getMessage(), throwable);
        }
    }

    private String formatTime(long time) {
        return String.format("%02d:%02d",
                TimeUnit.MILLISECONDS.toMinutes(time),
                TimeUnit.MILLISECONDS.toSeconds(time) % 60);
    }

    private void reset() {
        log.trace("Video player controls are being reset");
        this.media = null;
        this.subtitle = null;
        this.time = null;
        this.duration = null;

        Platform.runLater(() -> {
            slider.setValue(0);
            timeLabel.setText(formatTime(0));
            durationLabel.setText(formatTime(0));
        });
    }

    @FXML
    private void onPlayPauseClicked() {
        changePlayPauseState();
    }

    @FXML
    private void onFullscreenClicked() {
        toggleFullscreen();
    }

    @FXML
    private void onSubtitleSmaller() {
        onSubtitleSizeChanged(-4);
    }

    @FXML
    private void onSubtitleLarger() {
        onSubtitleSizeChanged(4);
    }

    //endregion
}
