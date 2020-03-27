package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.providers.models.Episode;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.messages.MediaMessage;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.subtitles.controls.LanguageSelection;
import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;
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
import org.apache.commons.io.FilenameUtils;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;

@Slf4j
@RequiredArgsConstructor
public class PlayerControlsComponent implements Initializable {
    private final List<PlayerControlsListener> listeners = new ArrayList<>();

    private final ActivityManager activityManager;
    private final SubtitleService subtitleService;
    private final LocaleText localeText;

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
    private boolean sliderHovered;

    //region Getters & Setters

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

    /**
     * Check if the controls are currently active and the hiding should be blocked.
     *
     * @return Returns true if the controls are showing, else false.
     */
    public boolean isBlocked() {
        return languageSelection.isShowing() || sliderHovered;
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

    public void toggleFullscreen() {
        log.trace("Toggling full screen mode");
        activityManager.register(new ToggleFullscreenActivity() {
        });
    }

    public void onPlayerStateChanged(PlayerState newValue) {
        switch (newValue) {
            case PLAYING:
                Platform.runLater(() -> playPauseIcon.setText(Icon.PAUSE_UNICODE));
                break;
            case PAUSED:
                Platform.runLater(() -> playPauseIcon.setText(Icon.PLAY_UNICODE));
                break;
        }
    }

    public void onTimeChanged(Number newValue) {
        Platform.runLater(() -> {
            long time = newValue.longValue();

            this.time = time;
            timeLabel.setText(formatTime(time));

            if (!slider.isValueChanging())
                slider.setValue(time);
        });
    }

    public void onDurationChanged(Number newValue) {
        Platform.runLater(() -> {
            long duration = newValue.longValue();

            durationLabel.setText(formatTime(duration));
            slider.setMax(duration);
            this.duration = duration;
        });
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeActivityListeners();
    }

    private void initializeActivityListeners() {
        activityManager.register(PlayVideoActivity.class, this::onPlayVideo);
        activityManager.register(ClosePlayerActivity.class, activity -> reset());
        activityManager.register(FullscreenActivity.class, this::onFullscreenChanged);
    }

    //endregion

    //region Functions

    private void initializeSlider() {
        slider.valueProperty().addListener((observableValue, oldValue, newValue) -> {
            if (slider.isValueChanging()) {
                listeners.forEach(e -> e.onTimeChanged(newValue.longValue()));
            }
        });

        slider.setOnMouseReleased(event -> setVideoTime(slider.getValue() + 1));
        slider.setOnMouseEntered(event -> sliderHovered = true);
        slider.setOnMouseExited(event -> sliderHovered = false);
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
        // update the visibility of the subtitles section
        Platform.runLater(() -> subtitleSection.setVisible(activity.isSubtitlesEnabled()));

        // check if the activity contains media information
        if (activity instanceof PlayMediaActivity) {
            var mediaActivity = (PlayMediaActivity) activity;
            onPlayMedia(mediaActivity);
            return;
        }

        if (activity.isSubtitlesEnabled()) {
            // set the default subtitle to "none" when loading
            SubtitleInfo defaultSubtitle = SubtitleInfo.none();
            updateAvailableSubtitles(Collections.singletonList(defaultSubtitle), defaultSubtitle);

            String filename = FilenameUtils.getName(activity.getUrl());

            log.debug("Retrieving subtitles for \"{}\"", filename);
            subtitleService.retrieveSubtitles(filename).whenComplete(this::handleSubtitlesResponse);
        }
    }

    private void onPlayMedia(PlayMediaActivity activity) {
        this.media = activity.getMedia();

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

            updateAvailableSubtitles(subtitles, subtitle);
        } else {
            log.error("Failed to retrieve subtitles, " + throwable.getMessage(), throwable);
        }
    }

    private void updateAvailableSubtitles(List<SubtitleInfo> subtitles, SubtitleInfo subtitle) {
        log.trace("Updating available subtitles to {}", subtitles.size());
        Platform.runLater(() -> {
            languageSelection.getItems().clear();
            languageSelection.getItems().addAll(subtitles);
            languageSelection.select(subtitle);
        });
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
            languageSelection.getItems().clear();
        });
    }

    @FXML
    private void onPlayPauseClicked() {
        this.listeners.forEach(PlayerControlsListener::onPlayPauseClicked);
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
