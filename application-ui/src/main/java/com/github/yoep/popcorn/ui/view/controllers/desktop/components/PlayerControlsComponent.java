package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.PlayMediaEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.messages.MediaMessage;
import com.github.yoep.popcorn.ui.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.controls.LanguageSelection;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractPlayerControlsComponent;
import com.github.yoep.popcorn.ui.view.controls.ProgressSliderControl;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerManagerService;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerService;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerSubtitleService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ListCell;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.Collections;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
public class PlayerControlsComponent extends AbstractPlayerControlsComponent implements Initializable {
    private final SubtitleService subtitleService;
    private final LocaleText localeText;

    @FXML
    private ProgressSliderControl playProgress;
    @FXML
    private Pane subtitleSection;
    @FXML
    private LanguageSelection languageSelection;
    @FXML
    private Icon fullscreenIcon;

    private Media media;
    private SubtitleInfo subtitle;

    //region Constructors

    public PlayerControlsComponent(VideoPlayerService videoPlayerService,
                                   VideoPlayerManagerService videoPlayerManagerService,
                                   VideoPlayerSubtitleService videoPlayerSubtitleService,
                                   SubtitleService subtitleService,
                                   LocaleText localeText) {
        super(videoPlayerService, videoPlayerManagerService, videoPlayerSubtitleService);
        this.subtitleService = subtitleService;
        this.localeText = localeText;
    }

    //endregion

    //region Methods

    @EventListener
    public void onPlayVideo(PlayVideoEvent activity) {
        // update the visibility of the subtitles section
        Platform.runLater(() -> subtitleSection.setVisible(activity.isSubtitlesEnabled()));

        // check if the activity contains media information
        if (activity instanceof PlayMediaEvent) {
            var mediaActivity = (PlayMediaEvent) activity;
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

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSlider();
        initializeLanguageSelection();
    }

    private void toggleFullscreen() {
        log.trace("Toggling full screen mode");
        videoPlayerService.toggleFullscreen();
    }

    //endregion

    //region AbstractPlayerControlsComponent

    @Override
    protected void onTimeChanged(Number newValue) {
        super.onTimeChanged(newValue);
        Platform.runLater(() -> {
            if (!playProgress.isValueChanging())
                playProgress.setTime(newValue.longValue());
        });
    }

    @Override
    protected void onDurationChanged(Number newValue) {
        super.onDurationChanged(newValue);
        Platform.runLater(() -> playProgress.setDuration(newValue.longValue()));
    }

    @Override
    protected void onProgressChanged(double newValue) {
        playProgress.setLoadProgress(newValue);
    }

    //endregion

    //region PostConstruct

    @Override
    protected void initializeVideoListeners() {
        super.initializeVideoListeners();
        videoPlayerService.fullscreenProperty().addListener((observable, oldValue, newValue) -> onFullscreenChanged(newValue));
    }

    //endregion

    //region Functions

    private void initializeSlider() {
        playProgress.valueChangingProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                videoPlayerService.pause();
            } else {
                videoPlayerService.resume();
            }
        });
        playProgress.timeProperty().addListener((observableValue, oldValue, newValue) -> {
            if (playProgress.isValueChanging()) {
                videoPlayerService.seek(newValue.longValue());
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
        videoPlayerSubtitleService.subtitleProperty().addListener((observable, oldValue, newValue) ->
                languageSelection.select(newValue.getSubtitleInfo().orElse(SubtitleInfo.none())));
    }

    private void onPlayMedia(PlayMediaEvent activity) {
        this.media = activity.getMedia();

        // set the subtitle for the playback
        this.subtitle = activity.getSubtitle()
                .flatMap(Subtitle::getSubtitleInfo)
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

    private void onFullscreenChanged(boolean isFullscreen) {
        if (isFullscreen) {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.COMPRESS_UNICODE));
        } else {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.EXPAND_UNICODE));
        }
    }

    private void onSubtitleChanged(SubtitleInfo newValue) {
        videoPlayerSubtitleService.setSubtitle(newValue);
    }

    private void onSubtitleSizeChanged(int pixelChange) {
        videoPlayerSubtitleService.setSubtitleSize(videoPlayerSubtitleService.getSubtitleSize() + pixelChange);
    }

    private void setVideoTime(double time) {
        playProgress.setValueChanging(true);
        playProgress.setTime((long) time);
        playProgress.setValueChanging(false);
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

    @Override
    protected void reset() {
        log.trace("Video player controls are being reset");
        this.media = null;
        this.subtitle = null;

        Platform.runLater(() -> {
            playProgress.setTime(0);
            languageSelection.getItems().clear();
        });
    }

    @FXML
    private void onPlayPauseClicked() {
        videoPlayerService.changePlayPauseState();
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
