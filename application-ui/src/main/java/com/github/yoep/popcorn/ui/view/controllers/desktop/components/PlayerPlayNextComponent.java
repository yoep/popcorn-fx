package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.ui.activities.ActivityManager;
import com.github.yoep.popcorn.ui.activities.ClosePlayerActivity;
import com.github.yoep.popcorn.ui.activities.LoadMediaTorrentActivity;
import com.github.yoep.popcorn.ui.activities.PlayMediaActivity;
import com.github.yoep.popcorn.ui.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerService;
import javafx.application.Platform;
import javafx.beans.value.ChangeListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerPlayNextComponent implements Initializable {
    private static final int COUNTDOWN_FROM = 60;

    private final ActivityManager activityManager;
    private final ImageService imageService;
    private final VideoPlayerService videoPlayerService;

    private final ChangeListener<Number> timeListener = (observable, oldValue, newValue) -> onTimeChanged(newValue);
    private final ChangeListener<Number> durationListener = (observable, oldValue, newValue) -> onDurationChanged(newValue);

    private Episode nextEpisode;
    private String url;
    private String quality;
    private long time;
    private long duration;

    @FXML
    private Pane playNextPane;
    @FXML
    private ImageView playNextPoster;
    @FXML
    private Label showName;
    @FXML
    private Label episodeTitle;
    @FXML
    private Label episodeNumber;
    @FXML
    private Label playingInCountdown;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePane();
    }

    private void initializePane() {

    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
        initializeVideoPlayerListener();
    }

    private void initializeListeners() {
        activityManager.register(PlayMediaActivity.class, this::onPlayMedia);
        activityManager.register(ClosePlayerActivity.class, this::onPlayerClosed);
    }

    private void initializeVideoPlayerListener() {
        videoPlayerService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> {
            if (oldValue != null) {
                oldValue.timeProperty().removeListener(timeListener);
                oldValue.durationProperty().removeListener(durationListener);
            }

            newValue.timeProperty().addListener(timeListener);
            newValue.durationProperty().addListener(durationListener);
        });
    }

    //endregion

    //region Functions

    private void onPlayMedia(PlayMediaActivity activity) {
        var media = activity.getMedia();

        // reset the component
        reset();

        // check if the current media is an episode
        // if not, ignore the update of information
        if (!isEpisode(media)) {
            return;
        }

        var episode = (Episode) media;
        var show = episode.getShow();
        var nextEpisodeIndex = episode.getEpisode() + 1;

        if (nextEpisodeIndex <= show.getEpisodes().size()) {
            url = activity.getUrl();
            nextEpisode = show.getEpisodes().get(nextEpisodeIndex);
            quality = activity.getQuality();

            Platform.runLater(() -> {
                showName.setText(show.getTitle());
                episodeTitle.setText(nextEpisode.getTitle());
                episodeNumber.setText(String.valueOf(nextEpisodeIndex));
            });

            imageService.loadPoster(episode, 100, 140).whenComplete((image, throwable) -> {
                if (throwable == null) {
                    image.ifPresentOrElse(playNextPoster::setImage,
                            () -> playNextPoster.setImage(imageService.getPosterHolder()));
                } else {
                    playNextPoster.setImage(imageService.getPosterHolder());
                    log.error("Failed to load poster of next episode, " + throwable.getMessage(), throwable);
                }
            });
        }
    }

    private void onPlayerClosed(ClosePlayerActivity activity) {
        reset();
    }

    private void onCountdownFinished() {
        var episode = this.nextEpisode;
        var quality = this.quality;

        activityManager.register(new ClosePlayerActivity() {
            @Override
            public String getUrl() {
                return url;
            }

            @Override
            public Optional<Media> getMedia() {
                return Optional.of(episode);
            }

            @Override
            public Optional<String> getQuality() {
                return Optional.of(quality);
            }

            @Override
            public long getTime() {
                return time;
            }

            @Override
            public long getDuration() {
                return duration;
            }
        });
        activityManager.register(new LoadMediaTorrentActivity() {
            @Override
            public MediaTorrentInfo getTorrent() {
                return episode.getTorrents().get(quality);
            }

            @Override
            public Media getMedia() {
                return episode;
            }

            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Optional<SubtitleInfo> getSubtitle() {
                return Optional.empty();
            }
        });
    }

    private void onTimeChanged(Number value) {
        time = value.longValue();
        var remainingTime = (duration - time) / 1000;

        if (remainingTime <= COUNTDOWN_FROM) {
            Platform.runLater(() -> {
                playNextPane.setVisible(true);
                playingInCountdown.setText(String.valueOf(remainingTime));
            });

            if (remainingTime == 0) {
                onCountdownFinished();
            }
        }
    }

    private void onDurationChanged(Number value) {
        this.duration = value.longValue();
    }

    private boolean isEpisode(Media media) {
        return media instanceof Episode;
    }

    private void reset() {
        nextEpisode = null;
        quality = null;

        Platform.runLater(() -> {
            playNextPane.setVisible(false);
            showName.setText(null);
            episodeTitle.setText(null);
            episodeNumber.setText(null);
            playNextPoster.setImage(imageService.getPosterHolder());
        });
    }

    @FXML
    private void onPlayNextClicked(MouseEvent event) {
        event.consume();
        onCountdownFinished();
    }

    //endregion
}
