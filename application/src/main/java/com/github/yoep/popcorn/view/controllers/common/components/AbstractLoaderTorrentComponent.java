package com.github.yoep.popcorn.view.controllers.common.components;

import com.frostwire.jlibtorrent.TorrentInfo;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.subtitles.Subtitle;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.subtitles.models.SubtitleMatcher;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.torrent.models.StreamStatus;
import com.github.yoep.popcorn.torrent.models.Torrent;
import com.github.yoep.popcorn.view.controllers.desktop.components.AbstractLoaderComponent;
import com.github.yoep.popcorn.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.util.Assert;

import java.util.Optional;
import java.util.concurrent.ExecutionException;

@Slf4j
public abstract class AbstractLoaderTorrentComponent extends AbstractLoaderComponent {
    protected final ActivityManager activityManager;
    protected final ImageService imageService;
    protected final SubtitleService subtitleService;
    protected final TaskExecutor taskExecutor;

    protected String title;
    protected Media media;
    protected SubtitleInfo subtitleInfo;
    protected Subtitle subtitle;
    protected String quality;
    protected MediaTorrentInfo mediaTorrent;
    protected Thread torrentThread;

    @FXML
    protected Label statusText;
    @FXML
    protected Label progressPercentage;
    @FXML
    protected Label downloadText;
    @FXML
    protected Label uploadText;
    @FXML
    protected Label activePeersText;
    @FXML
    protected ProgressBar progressBar;
    @FXML
    protected Pane progressStatus;
    @FXML
    protected BackgroundImageCover backgroundImage;

    //region Constructors

    protected AbstractLoaderTorrentComponent(LocaleText localeText, TorrentService torrentService, ActivityManager activityManager, ImageService imageService, SubtitleService subtitleService, TaskExecutor taskExecutor) {
        super(localeText, torrentService);
        Assert.notNull(activityManager, "activityManager cannot be null");
        Assert.notNull(imageService, "imageService cannot be null");
        Assert.notNull(subtitleService, "subtitleService cannot be null");
        Assert.notNull(taskExecutor, "taskExecutor cannot be null");
        this.activityManager = activityManager;
        this.imageService = imageService;
        this.subtitleService = subtitleService;
        this.taskExecutor = taskExecutor;
    }

    //endregion

    //region Functions

    protected void reset() {
        this.title = null;
        this.media = null;
        this.subtitleInfo = null;
        this.quality = null;
        this.mediaTorrent = null;
        this.torrentThread = null;
        this.backgroundImage.reset();
    }

    protected void startTorrent(LoadMediaTorrentActivity activity) {
        Assert.notNull(activity.getMedia(), "LoadMediaTorrentActivity#getMedia cannot be null");
        log.debug("Starting media torrent stream for {}", activity.getMedia());

        // store the requested media locally for later use
        this.title = activity.getMedia().getTitle();
        this.media = activity.getMedia();
        this.subtitleInfo = activity.getSubtitle().orElse(null);
        this.quality = activity.getQuality();
        this.mediaTorrent = activity.getTorrent();

        this.torrentThread = new Thread(() -> {
            // reset the progress bar to "infinite" animation
            resetProgress();
            Platform.runLater(() -> {
                loadBackgroundImage();
                progressStatus.setVisible(false);
                statusText.setText(localeText.get(TorrentMessage.STARTING));
            });

            // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
            if (!torrentService.isInitialized())
                waitForTorrentStream();

            // check if the subtitle needs to be downloaded
            if (subtitleInfo != null) {
                var torrentInfo = torrentService.getTorrentInfo(mediaTorrent.getUrl());

                downloadSubtitles(torrentInfo
                        .map(this::getLargestFilename)
                        .orElse(null));
            }

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.trace("Calling torrent service stream for \"{}\"", mediaTorrent.getUrl());
            torrentService.startStream(mediaTorrent.getUrl());
        });

        taskExecutor.execute(this.torrentThread);
    }

    protected void downloadSubtitles(String filename) {
        // check if the given subtitle is the special "none" subtitle, if so, ignore the subtitle download
        if (subtitleInfo == null || subtitleInfo.isNone())
            return;

        // update the status text to "downloading subtitle"
        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.DOWNLOADING_SUBTITLE)));

        // download the subtitle locally, this will cache the subtitle for later use in the video player
        try {
            subtitle = subtitleService.downloadAndParse(subtitleInfo, SubtitleMatcher.from(filename, quality)).get();
        } catch (InterruptedException | ExecutionException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    protected TorrentListener createTorrentListener() {
        return new TorrentListener() {
            @Override
            public void onLoadError(String message) {
                log.warn("Torrent loading failed: {}", message);
                updateProgressToErrorState();
            }

            @Override
            public void onStreamStarted(Torrent torrent) {
                log.debug("Torrent is starting");
                Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.STARTING)));
            }

            @Override
            public void onStreamError(Torrent torrent, Exception e) {
                log.warn("Torrent stream error: " + e.getMessage(), e);
            }

            @Override
            public void onStreamReady(Torrent torrent) {
                log.debug("Torrent is ready to be displayed");
                Platform.runLater(() -> {
                    statusText.setText(localeText.get(TorrentMessage.READY));
                    progressBar.setProgress(1);
                });
                invokePlayActivity(torrent);
            }

            @Override
            public void onStreamProgress(Torrent torrent, StreamStatus status) {
                Platform.runLater(() -> {
                    progressStatus.setVisible(true);
                    progressBar.setProgress(status.getProgress());
                    statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
                    progressPercentage.setText(String.format("%1$,.2f", status.getProgress() * 100) + "%");
                    downloadText.setText(StreamStatus.toDisplaySize(status.getDownloadSpeed()) + "/s");
                    uploadText.setText(StreamStatus.toDisplaySize(status.getUploadSpeed()) + "/s");
                    activePeersText.setText(String.valueOf(status.getSeeds()));
                });
            }

            @Override
            public void onStreamStopped() {
                log.debug("Torrent stream has stopped");
            }
        };
    }

    protected void close() {
        // stop the current torrent operation thread if one is present & alive
        if (torrentThread != null && torrentThread.getState() != Thread.State.TERMINATED)
            torrentThread.interrupt();

        torrentService.stopStream();
        activityManager.register(new CloseLoadActivity() {
        });

        reset();
    }

    private void invokePlayActivity(Torrent torrent) {
        // check if the media is known
        // if so, use the play media activity, else the play video activity
        if (media != null) {
            invokePlayMediaActivity(torrent);
        } else {
            invokePlayVideoActivity(torrent);
        }

        // reset this load after invoking the activity for memory cleanup
        reset();
    }

    private void invokePlayVideoActivity(Torrent torrent) {
        // store information locally
        var url = torrentService.getStreamUrl(torrent);
        var title = this.title;

        activityManager.register(new PlayVideoActivity() {
            @Override
            public String getUrl() {
                return url;
            }

            @Override
            public String getTitle() {
                return title;
            }

            @Override
            public boolean isSubtitlesEnabled() {
                return true;
            }
        });
    }

    private void invokePlayMediaActivity(Torrent torrent) {
        // store information locally
        var url = torrentService.getStreamUrl(torrent);
        var title = this.title;
        var media = this.media;
        var quality = this.quality;
        var subtitle = this.subtitle;

        activityManager.register(new PlayMediaActivity() {
            @Override
            public String getUrl() {
                return url;
            }

            @Override
            public String getTitle() {
                return title;
            }

            @Override
            public boolean isSubtitlesEnabled() {
                return true;
            }

            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Optional<Subtitle> getSubtitle() {
                return Optional.ofNullable(subtitle)
                        .or(() -> Optional.of(Subtitle.none()));
            }
        });
    }

    private String getLargestFilename(TorrentInfo torrentInfo) {
        var files = torrentInfo.files();
        String filename = null;
        long largestSize = 0;

        for (int i = 0; i < torrentInfo.numFiles(); i++) {
            var name = files.fileName(i);
            var size = files.fileSize(i);

            if (size > largestSize) {
                filename = name;
                largestSize = size;
            }
        }

        return filename;
    }

    private void loadBackgroundImage() {
        backgroundImage.reset();
        imageService.loadFanart(media).whenComplete((bytes, throwable) -> {
            if (throwable == null) {
                bytes.ifPresent(e -> backgroundImage.setBackgroundImage(e));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    //endregion
}
