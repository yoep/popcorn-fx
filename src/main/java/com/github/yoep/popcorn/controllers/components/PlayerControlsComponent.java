package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.video.VideoPlayer;
import com.github.yoep.popcorn.media.video.state.PlayerState;
import com.github.yoep.popcorn.media.video.time.TimeListener;
import com.github.yoep.popcorn.messages.MediaMessage;
import com.github.yoep.popcorn.subtitle.SubtitleService;
import com.github.yoep.popcorn.subtitle.controls.LanguageSelection;
import com.github.yoep.popcorn.subtitle.models.Language;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
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
import java.util.*;
import java.util.concurrent.TimeUnit;
import java.util.stream.Collectors;

@Slf4j
@Component
@RequiredArgsConstructor
public class PlayerControlsComponent implements Initializable {
    private final List<PlayerControlsListener> listeners = new ArrayList<>();

    private final ActivityManager activityManager;
    private final SubtitleService subtitleService;
    private final LocaleText localeText;

    @FXML
    private Icon playPauseIcon;
    @FXML
    private Label currentTime;
    @FXML
    private Label duration;
    @FXML
    private Slider slider;
    @FXML
    private Pane subtitleSection;
    @FXML
    private LanguageSelection languageSelection;
    @FXML
    private Icon fullscreenIcon;

    private VideoPlayer videoPlayer;
    private Media media;
    private SubtitleInfo subtitle;

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

    /**
     * Set the video player that this control component manages.
     *
     * @param videoPlayer The video player to manage with the controls of this component.
     */
    void setVideoPlayer(VideoPlayer videoPlayer) {
        this.videoPlayer = videoPlayer;
        initializeVideoPlayer();
    }

    void increaseVideoTime(double amount) {
        log.trace("Increasing video time with {}", amount);
        double newSliderValue = slider.getValue() + amount;
        double maxSliderValue = slider.getMax();

        if (newSliderValue > maxSliderValue)
            newSliderValue = maxSliderValue;

        setVideoTime(newSliderValue);
    }

    void changePlayPauseState() {
        if (videoPlayer.getPlayerState() == PlayerState.PAUSED) {
            log.trace("Video player state is being changed to \"resume\"");
            videoPlayer.resume();
        } else {
            log.trace("Video player state is being changed to \"paused\"");
            videoPlayer.pause();
        }
    }

    void toggleFullscreen() {
        log.trace("Toggling full screen mode");
        activityManager.register(new ToggleFullscreenActivity() {
        });
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(PlayVideoActivity.class, this::onPlayVideo);
        activityManager.register(PlayerCloseActivity.class, activity -> reset());
        activityManager.register(FullscreenActivity.class, this::onFullscreenChanged);
        activityManager.register(SubtitlesRetrievedActivity.class, this::onSubtitlesRetrieved);
    }

    //endregion

    //region Functions

    private void initializeVideoPlayer() {
        videoPlayer.addListener((oldState, newState) -> {
            switch (newState) {
                case PLAYING:
                    Platform.runLater(() -> playPauseIcon.setText(Icon.PAUSE_UNICODE));
                    break;
                case PAUSED:
                    Platform.runLater(() -> playPauseIcon.setText(Icon.PLAY_UNICODE));
                    break;
            }
        });
        videoPlayer.addListener(new TimeListener() {
            @Override
            public void onTimeChanged(long newTime) {
                Platform.runLater(() -> {
                    currentTime.setText(formatTime(newTime));
                    slider.setValue(newTime);
                });
            }

            @Override
            public void onLengthChanged(long newLength) {
                Platform.runLater(() -> {
                    duration.setText(formatTime(newLength));
                    slider.setMax(newLength);
                });
            }
        });
    }

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
                videoPlayer.setTime(newValue.longValue());
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
                        Language language = Language.valueOf(item.getLanguage());

                        if (language != null) {
                            setText(language.getNativeName());
                        } else {
                            setText(item.getLanguage());
                        }
                    }
                }
            }
        });
        languageSelection.addListener(this::onSubtitleChanged);
    }

    private void onPlayVideo(PlayVideoActivity activity) {
        this.media = activity.getMedia();

        Platform.runLater(() -> subtitleSection.setVisible(activity.getQuality().isPresent()));

        activity.getSubtitle().ifPresentOrElse(
                subtitle -> this.subtitle = subtitle,
                () -> {
                    this.subtitle = SubtitleInfo.none();
                    setSubtitles(Collections.singletonList(this.subtitle));
                }
        );

        if (media instanceof Movie) {
            subtitleService.retrieveSubtitles((Movie) activity.getMedia());
        }
    }

    private void onFullscreenChanged(FullscreenActivity activity) {
        if (activity.isFullscreen()) {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.COLLAPSE_UNICODE));
        } else {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.EXPAND_UNICODE));
        }
    }

    public void onSubtitlesRetrieved(SubtitlesRetrievedActivity activity) {
        if (this.media == null || !Objects.equals(activity.getImdbId(), media.getImdbId()))
            return;

        setSubtitles(activity.getSubtitles());
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

    private void setSubtitles(List<SubtitleInfo> subtitles) {
        final var filteredSubtitles = subtitles.stream()
                .filter(e -> e.getFlagResource().isPresent())
                .collect(Collectors.toList());

        Platform.runLater(() -> {
            languageSelection.getItems().clear();
            languageSelection.getItems().addAll(filteredSubtitles);
            languageSelection.select(this.subtitle);
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

        Platform.runLater(() -> {
            slider.setValue(0);
            currentTime.setText(formatTime(0));
            duration.setText(formatTime(0));
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
