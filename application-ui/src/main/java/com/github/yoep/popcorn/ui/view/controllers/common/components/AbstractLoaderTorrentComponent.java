package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.AbstractTorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.AbstractTorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import com.github.yoep.popcorn.ui.events.CloseLoadEvent;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.PlayMediaEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoTorrentEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.TorrentSettings;
import com.github.yoep.popcorn.ui.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleMatcher;
import com.github.yoep.popcorn.ui.torrent.utils.SizeUtils;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.AbstractLoaderComponent;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.core.task.TaskExecutor;
import org.springframework.util.Assert;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.ExecutionException;

@Slf4j
public abstract class AbstractLoaderTorrentComponent extends AbstractLoaderComponent implements Initializable {
    protected final ApplicationEventPublisher eventPublisher;
    protected final ImageService imageService;
    protected final SubtitleService subtitleService;
    protected final TaskExecutor taskExecutor;
    protected final SettingsService settingsService;

    private final TorrentListener torrentListener = createTorrentListener();
    private final TorrentStreamListener torrentStreamListener = createTorrentStreamListener();

    protected String title;
    protected Media media;
    protected SubtitleInfo subtitleInfo;
    protected Subtitle subtitle;
    protected String quality;
    protected MediaTorrentInfo mediaTorrent;
    protected Thread torrentThread;

    private Torrent torrent;
    private TorrentStream torrentStream;

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

    protected AbstractLoaderTorrentComponent(LocaleText localeText,
                                             TorrentService torrentService,
                                             TorrentStreamService torrentStreamService,
                                             ApplicationEventPublisher eventPublisher,
                                             ImageService imageService,
                                             SubtitleService subtitleService,
                                             TaskExecutor taskExecutor,
                                             SettingsService settingsService) {
        super(localeText, torrentService, torrentStreamService);
        this.eventPublisher = eventPublisher;
        this.imageService = imageService;
        this.subtitleService = subtitleService;
        this.taskExecutor = taskExecutor;
        this.settingsService = settingsService;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeProgressBar();
    }

    protected void initializeProgressBar() {
        progressBar.setProgress(0.0);
        progressBar.setVisible(false);
    }

    //endregion

    //region Functions

    @Override
    protected void reset() {
        super.reset();

        this.title = null;
        this.media = null;
        this.subtitleInfo = null;
        this.quality = null;
        this.mediaTorrent = null;
        this.torrentThread = null;
        this.torrent = null;
        this.torrentStream = null;
        this.backgroundImage.reset();
    }

    protected void startTorrent(LoadMediaTorrentEvent activity) {
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
            if (torrentService.getSessionState() != SessionState.RUNNING)
                waitForTorrentStream();

            // resolve the url to a torrent
            torrentService.getTorrentInfo(mediaTorrent.getUrl()).whenComplete(this::onTorrentInfoRetrieved);
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

    protected void close() {
        log.debug("Cancelling torrent loader");

        // stop the current torrent operation thread if one is present & alive
        if (torrentThread != null && torrentThread.getState() != Thread.State.TERMINATED)
            torrentThread.interrupt();

        // stop the torrent if it was already created
        if (torrent != null) {
            torrentService.remove(torrent);
        }

        // stop the torrent stream if it was already created
        if (torrentStream != null) {
            torrentStreamService.stopStream(torrentStream);
        }

        eventPublisher.publishEvent(new CloseLoadEvent(this));
        reset();
    }

    protected void onTorrentCreated(Torrent torrent, Throwable throwable) {
        if (throwable == null) {
            // register the torrent listener to this torrent
            this.torrent = torrent;
            this.torrent.addListener(torrentListener);

            // create a stream for this torrent
            torrentStream = torrentStreamService.startStream(torrent);
            torrentStream.addListener(torrentStreamListener);
        } else {
            log.error("Failed to create torrent, " + throwable.getMessage(), throwable);
            updateProgressToErrorState();
        }
    }

    protected TorrentSettings getTorrentSettings() {
        return settingsService.getSettings().getTorrentSettings();
    }

    private void onTorrentInfoRetrieved(TorrentInfo torrentInfo, Throwable throwable) {
        if (throwable == null) {
            var torrentSettings = getTorrentSettings();
            var torrentFileInfo = mediaTorrent.getFile()
                    // if a filename has been given by the api
                    // then search for the file given by the api
                    .flatMap(torrentInfo::getByFilename)
                    // otherwise, take the largest file we can find within the torrent
                    .orElseGet(torrentInfo::getLargestFile);

            // check if the subtitle needs to be downloaded
            if (subtitleInfo != null) {
                downloadSubtitles(torrentFileInfo.getFilename());
            }

            // update the status text to "connecting"
            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            log.trace("Creating torrent for \"{}\"", mediaTorrent.getUrl());
            torrentService.create(torrentFileInfo, torrentSettings.getDirectory(), true).whenComplete(this::onTorrentCreated);
        } else {
            log.error("Failed to retrieve torrent info, " + throwable.getMessage(), throwable);
            updateProgressToErrorState();
        }
    }

    private void onDownloadProgress(DownloadStatus status) {
        runOnFx(() -> {
            progressStatus.setVisible(true);
            progressBar.setProgress(status.getProgress());
            progressBar.setVisible(true);
            statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
            progressPercentage.setText(String.format("%1$,.2f", status.getProgress() * 100) + "%");
            downloadText.setText(SizeUtils.toDisplaySize(status.getDownloadSpeed()) + "/s");
            uploadText.setText(SizeUtils.toDisplaySize(status.getUploadSpeed()) + "/s");
            activePeersText.setText(String.valueOf(status.getSeeds()));
        });
    }

    private void onStreamReady() {
        log.debug("Torrent is ready to be displayed");
        runOnFx(() -> {
            statusText.setText(localeText.get(TorrentMessage.READY));
            progressBar.setProgress(1);
            progressBar.setVisible(true);
        });
        invokePlayActivity();
    }

    private void invokePlayActivity() {
        // check if the media is known
        // if so, use the play media activity, else the play video activity
        if (media != null) {
            invokePlayMediaActivity();
        } else {
            invokePlayVideoEvent();
        }

        // reset this load after invoking the activity for memory cleanup
        reset();
    }

    private void invokePlayVideoEvent() {
        var url = torrentStream.getStreamUrl();

        eventPublisher.publishEvent(new PlayVideoTorrentEvent(this, url, title, true, torrent, torrentStream));
    }

    private void invokePlayMediaActivity() {
        eventPublisher.publishEvent(PlayMediaEvent.mediaBuilder()
                .source(this)
                .media(media)
                .quality(quality)
                .subtitle(subtitle)
                .subtitlesEnabled(true)
                .title(title)
                .torrent(torrent)
                .torrentStream(torrentStream)
                .url(torrentStream.getStreamUrl())
                .build());
    }

    private TorrentListener createTorrentListener() {
        return new AbstractTorrentListener() {
            @Override
            public void onStateChanged(TorrentState oldState, TorrentState newState) {
                if (newState == TorrentState.STARTING) {
                    log.debug("Torrent is starting");
                    Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.STARTING)));
                }
            }

            @Override
            public void onError(TorrentException error) {
                updateProgressToErrorState();
            }

            @Override
            public void onDownloadProgress(DownloadStatus status) {
                AbstractLoaderTorrentComponent.this.onDownloadProgress(status);
            }
        };
    }

    private TorrentStreamListener createTorrentStreamListener() {
        return new AbstractTorrentStreamListener() {
            @Override
            public void onStreamError(Exception error) {
                log.error("Failed to stream torrent, error: {}", error.getMessage());
                updateProgressToErrorState();
            }

            @Override
            public void onStreamReady() {
                AbstractLoaderTorrentComponent.this.onStreamReady();
            }

            @Override
            public void onStreamStopped() {
                log.trace("Torrent stream has been stopped");
                resetProgress();
            }
        };
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
