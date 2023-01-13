package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.adapters.torrent.FailedToPrepareTorrentStreamException;
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
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoTorrentEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.TorrentSettings;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.view.listeners.LoadTorrentListener;
import javafx.beans.value.ChangeListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.lang.Nullable;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

@Slf4j
@Service
@RequiredArgsConstructor
public class LoadTorrentService extends AbstractListenerService<LoadTorrentListener> {
    private static final int RETRIEVE_SUBTITLES_TIMEOUT = 15;
    private static final int DOWNLOAD_SUBTITLE_TIMEOUT = 30;

    private final TorrentService torrentService;
    private final TorrentStreamService torrentStreamService;
    private final ApplicationEventPublisher eventPublisher;
    private final SettingsService settingsService;
    private final SubtitleService subtitleService;

    private final ChangeListener<SessionState> torrentSessionListener = createSessionListener();
    private final TorrentListener torrentListener = createTorrentListener();
    private final TorrentStreamListener torrentStreamListener = createTorrentStreamListener();

    private LoadTorrentEvent event;
    private Torrent torrent;
    private TorrentStream torrentStream;
    private CompletableFuture<?> currentFuture;

    //region Methods

    @Async
    @EventListener
    public void onLoadMediaTorrent(LoadMediaTorrentEvent event) {
        loadMediaTorrent(event);
    }

    @Async
    @EventListener
    public void onLoadUrlTorrent(LoadUrlTorrentEvent event) {
        loadUrlTorrent(event);
    }

    @Async
    @EventListener
    public void onLoadUrl(LoadUrlEvent event) {
        loadUrl(event);
    }

    /**
     * Cancel the current loading process.
     */
    public void cancel() {
        doInternalCancel();

        // remove the stored info
        resetToIdleState();

        subtitleService.setActiveSubtitle(Subtitle.none());
        eventPublisher.publishEvent(new CloseLoadEvent(this));
    }

    /**
     * Retry the last load event.
     * If no event is stored, the retry will be ignored.
     */
    public void retryLoadingTorrent() {
        Optional.ofNullable(event)
                .ifPresent(eventPublisher::publishEvent);
    }

    //endregion

    //region Functions

    private void resetToIdleState() {
        this.event = null;
        this.torrent = null;
        this.torrentStream = null;
        this.currentFuture = null;
    }

    private synchronized void loadMediaTorrent(LoadMediaTorrentEvent event) {
        Objects.requireNonNull(event, "event cannot be null");
        Objects.requireNonNull(event.getMedia(), "event#getMedia cannot be null");
        log.debug("Loading media torrent stream for {}, {}", event.getMedia().getId(), event.getMedia().getTitle());
        resetToIdleState();

        // store the requested media locally for later use
        this.event = event;
        invokeListeners(e -> e.onMediaChanged(event.getMedia()));
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.STARTING));

        // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
        if (torrentService.getSessionState() != SessionState.RUNNING)
            waitForTorrentStream();

        // resolve the url to a torrent
        currentFuture = torrentService.getTorrentInfo(event.getTorrent().getUrl())
                .thenCompose(this::createTorrentFromTorrentInfo)
                .thenCompose(this::retrieveAndDownloadAvailableSubtitles)
                .thenCompose(this::startDownloadingTorrent)
                .exceptionally(this::handleLoadTorrentError);
    }

    private synchronized void loadUrlTorrent(LoadUrlTorrentEvent event) {
        Objects.requireNonNull(event, "event cannot be null");
        log.debug("Loading url torrent stream for {}", event.getTorrentFileInfo().getFilename());
        resetToIdleState();
        var torrentSettings = getTorrentSettings();

        // store the requested media locally for later use
        this.event = event;
        invokeListeners(e -> e.onMediaChanged(null));
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.STARTING));

        // check if the torrent stream is initialized, of not, wait for it to be initialized before proceeding
        if (torrentService.getSessionState() != SessionState.RUNNING)
            waitForTorrentStream();

        currentFuture = torrentService.create(event.getTorrentFileInfo(), torrentSettings.getDirectory(), true)
                .thenCompose(this::retrieveAndDownloadAvailableSubtitles)
                .thenCompose(this::startDownloadingTorrent)
                .exceptionally(this::handleLoadTorrentError);
    }

    private synchronized void loadUrl(LoadUrlEvent event) {
        Objects.requireNonNull(event, "event cannot be null");
        log.debug("Loading magnet url");
        resetToIdleState();

        invokeListeners(e -> e.onMediaChanged(null));
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.CONNECTING));

        torrentService.getTorrentInfo(event.getUrl()).whenComplete((torrentInfo, throwable) -> {
            if (throwable == null) {
                eventPublisher.publishEvent(new ShowTorrentDetailsEvent(this, event.getUrl(), torrentInfo));
            } else {
                log.error("Failed to retrieve torrent, " + throwable.getMessage(), throwable);
                invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.ERROR));
            }
        });
    }

    private Torrent handleLoadTorrentError(Throwable throwable) {
        // check if an error occurred while preparing the stream
        // if so, start the flow again from the start
        // by publishing the original message
        if (throwable.getCause() instanceof FailedToPrepareTorrentStreamException ex) {
            log.trace(ex.getMessage(), ex);
            log.warn("Failed to prepare torrent stream, restarting load torrent process");

            // cancel the current loading process
            doInternalCancel();

            if (event instanceof LoadMediaTorrentEvent mediaTorrentEvent) {
                loadMediaTorrent(mediaTorrentEvent);
            } else if (event instanceof LoadUrlTorrentEvent urlTorrentEvent) {
                loadUrlTorrent(urlTorrentEvent);
            } else if (event != null) {
                log.error("Failed to restart torrent loading process, unknown event {}", event.getClass().getSimpleName());
            }
        } else {
            log.error("Failed to load torrent, " + throwable.getMessage(), throwable);
            invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.ERROR));
        }

        return null;
    }

    private CompletableFuture<Torrent> createTorrentFromTorrentInfo(TorrentInfo torrentInfo) {
        var mediaTorrent = (LoadMediaTorrentEvent) event;
        var torrentSettings = getTorrentSettings();
        var torrentFileInfo = mediaTorrent.getTorrent().getFile()
                // if a filename has been given by the api
                // then search for the file given by the api
                .flatMap(torrentInfo::getByFilename)
                // otherwise, take the largest file we can find within the torrent
                .orElseGet(torrentInfo::getLargestFile);

        // update the status text to "connecting"
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.CONNECTING));

        log.trace("Creating torrent for \"{}\"", mediaTorrent.getTorrent().getUrl());
        return torrentService.create(torrentFileInfo, torrentSettings.getDirectory(), true);
    }

    private CompletableFuture<Torrent> startDownloadingTorrent(Torrent torrent) {
        log.debug("Torrent {} has been created for event {}", torrent.getFilename(), event);

        // register the torrent listener to this torrent
        this.torrent = torrent;
        this.torrent.addListener(torrentListener);

        // create a stream for this torrent
        this.torrentStream = torrentStreamService.startStream(torrent);
        this.torrentStream.addListener(torrentStreamListener);
        return CompletableFuture.completedFuture(torrent);
    }

    private CompletableFuture<Torrent> retrieveAndDownloadAvailableSubtitles(Torrent torrent) {
        var availableSubtitles = Collections.<SubtitleInfo>emptyList();
        SubtitleInfo selectedSubtitle = null;
        String quality = null;

        if (event instanceof LoadMediaTorrentEvent mediaEvent) {
            selectedSubtitle = mediaEvent.getSubtitle().orElse(null);
            quality = mediaEvent.getQuality();
            availableSubtitles = retrieveAvailableMediaSubtitles(mediaEvent.getMedia(), mediaEvent.getSubItem().orElse(null));
        } else if (event instanceof LoadUrlTorrentEvent urlEvent) {
            selectedSubtitle = urlEvent.getSubtitle().orElse(null);
            availableSubtitles = retrieveAvailableFilenameSubtitles(urlEvent.getTorrentFileInfo().getFilename());
        }

        if (selectedSubtitle == null) {
            selectedSubtitle = subtitleService.getDefaultOrInterfaceLanguage(availableSubtitles);
        }

        downloadSubtitles(selectedSubtitle, quality, torrent.getFilename());
        return CompletableFuture.completedFuture(torrent);
    }

    private List<SubtitleInfo> retrieveAvailableFilenameSubtitles(String filename) {
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.RETRIEVING_SUBTITLES));
        try {
            return subtitleService.retrieveSubtitles(filename)
                    .exceptionally(throwable -> {
                        log.error("Failed to retrieve subtitles for {}, {}", filename, throwable.getMessage());
                        return Collections.emptyList();
                    })
                    .get(RETRIEVE_SUBTITLES_TIMEOUT, TimeUnit.SECONDS);
        } catch (InterruptedException | ExecutionException ex) {
            log.debug("Retrieving filename subtitles has been cancelled");
        } catch (TimeoutException ex) {
            log.warn("Retrieving filename subtitles has timed out");
        }

        return Collections.emptyList();
    }

    private List<SubtitleInfo> retrieveAvailableMediaSubtitles(Media media, @Nullable Media subMediaItem) {
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.RETRIEVING_SUBTITLES));
        try {
            if (media instanceof MovieDetails movie) {
                return subtitleService.retrieveSubtitles(movie)
                        .exceptionally(throwable -> {
                            log.error("Failed to retrieve subtitles for movie {}, {}", movie, throwable.getMessage());
                            return Collections.emptyList();
                        })
                        .get(RETRIEVE_SUBTITLES_TIMEOUT, TimeUnit.SECONDS);
            } else if (media instanceof ShowDetails show && subMediaItem instanceof Episode episode) {
                return subtitleService.retrieveSubtitles(show, episode)
                        .exceptionally(throwable -> {
                            log.error("Failed to retrieve subtitles for episode {}, {}", episode, throwable.getMessage());
                            return Collections.emptyList();
                        })
                        .get(RETRIEVE_SUBTITLES_TIMEOUT, TimeUnit.SECONDS);
            }
        } catch (InterruptedException | ExecutionException ex) {
            log.debug("Retrieving media subtitles has been cancelled");
        } catch (TimeoutException e) {
            log.warn("Retrieving media subtitles has timed out");
        }

        return Collections.emptyList();
    }

    private void downloadSubtitles(SubtitleInfo subtitleInfo, String quality, String filename) {
        // check if the given subtitle is the special "none" subtitle, if so, ignore the subtitle download
        if (subtitleInfo == null || subtitleInfo.isNone())
            return;

        log.debug("Retrieving subtitle for {} with quality {}", filename, quality);
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.DOWNLOADING_SUBTITLE));

        // download the subtitle locally, this will cache the subtitle for later use in the video player
        try {
            var subtitle = subtitleService.downloadAndParse(subtitleInfo, SubtitleMatcher.from(filename, quality))
                    .exceptionally(throwable -> {
                        log.warn("Failed to load torrent subtitle {}, {}", subtitleInfo, throwable.getMessage(), throwable);
                        return Subtitle.none();
                    })
                    .get(DOWNLOAD_SUBTITLE_TIMEOUT, TimeUnit.SECONDS);
            subtitleService.setActiveSubtitle(subtitle);
            log.info("Torrent subtitle {} has been activated for playback", subtitle);
        } catch (InterruptedException | ExecutionException ex) {
            log.debug("Downloading of the torrent subtitle has been cancelled");
        } catch (TimeoutException e) {
            log.warn("Downloading of the torrent subtitle has timed out");
        }
    }

    private synchronized void waitForTorrentStream() {
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.INITIALIZING));

        try {
            if (torrentService.getSessionState() != SessionState.RUNNING)
                log.trace("Waiting for the torrent service to be initialized");

            // add a listener on the session state
            torrentService.sessionStateProperty().addListener(torrentSessionListener);
            this.wait();
        } catch (InterruptedException e) {
            log.error("Unexpectedly quit of wait for torrent stream monitor", e);
        }
    }

    private void doInternalCancel() {
        if (currentFuture != null && !currentFuture.isDone()) {
            log.debug("Cancelling current future task");
            currentFuture.cancel(true);
        }

        // stop the torrent if it was already created
        if (torrent != null) {
            torrent.removeListener(torrentListener);
            torrentService.remove(torrent);
        }

        // stop the torrent stream if it was already created
        if (torrentStream != null) {
            torrentStream.removeListener(torrentStreamListener);
            torrentStreamService.stopStream(torrentStream);
        }
    }

    private void onSessionStateChanged(SessionState newValue) {
        log.debug("Session state changed to {}", newValue);
        synchronized (this) {
            this.notifyAll();
            torrentService.sessionStateProperty().removeListener(torrentSessionListener);
        }
    }

    private void onStreamReady() {
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.READY));
        invokePlayActivity();
    }

    private void onStreamError(Exception error) {
        log.error("Failed to stream torrent, error: {}", error.getMessage());
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.ERROR));
    }

    private void onStreamStopped() {
        log.trace("Torrent stream has been stopped");
        resetToIdleState();
    }

    private void onDownloadStatus(DownloadStatus status) {
        invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.DOWNLOADING));
        invokeListeners(e -> e.onDownloadStatusChanged(status));
    }

    private void invokePlayActivity() {
        if (event instanceof LoadMediaTorrentEvent mediaEvent) {
            invokePlayMediaActivity(mediaEvent);
        } else if (event instanceof LoadUrlTorrentEvent torrentEvent) {
            invokePlayVideoEvent(torrentEvent);
        } else {
            log.error("Unable to start playback, unknown event type {}", event.getClass().getSimpleName());
            invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.ERROR));
        }
    }

    private void invokePlayVideoEvent(LoadUrlTorrentEvent torrentEvent) {
        var url = torrentStream.getStreamUrl();

        eventPublisher.publishEvent(PlayVideoTorrentEvent.videoTorrentBuilder()
                .source(this)
                .url(url)
                .title(torrentEvent.getTorrentFileInfo().getFilename())
                .subtitlesEnabled(true)
                .torrent(torrent)
                .torrentStream(torrentStream)
                .build());
    }

    private void invokePlayMediaActivity(LoadMediaTorrentEvent mediaEvent) {
        eventPublisher.publishEvent(PlayMediaEvent.mediaBuilder()
                .source(this)
                .media(mediaEvent.getMedia())
                .subMediaItem(mediaEvent.getSubItem().orElse(null))
                .quality(mediaEvent.getQuality())
                .subtitlesEnabled(true)
                .title(mediaEvent.getMedia().getTitle())
                .torrent(torrent)
                .torrentStream(torrentStream)
                .url(torrentStream.getStreamUrl())
                .build());
    }

    private TorrentSettings getTorrentSettings() {
        return settingsService.getSettings().getTorrentSettings();
    }

    private ChangeListener<SessionState> createSessionListener() {
        return (observable, oldValue, newValue) -> onSessionStateChanged(newValue);
    }

    private TorrentListener createTorrentListener() {
        return new AbstractTorrentListener() {
            @Override
            public void onStateChanged(TorrentState oldState, TorrentState newState) {
                if (newState == TorrentState.STARTING) {
                    log.debug("Torrent is starting");
                    invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.STARTING));
                }
            }

            @Override
            public void onError(TorrentException error) {
                invokeListeners(e -> e.onStateChanged(LoadTorrentListener.State.ERROR));
            }

            @Override
            public void onDownloadStatus(DownloadStatus status) {
                LoadTorrentService.this.onDownloadStatus(status);
            }
        };
    }

    private TorrentStreamListener createTorrentStreamListener() {
        return new AbstractTorrentStreamListener() {
            @Override
            public void onStreamError(Exception error) {
                LoadTorrentService.this.onStreamError(error);
            }

            @Override
            public void onStreamReady() {
                LoadTorrentService.this.onStreamReady();
            }

            @Override
            public void onStreamStopped() {
                LoadTorrentService.this.onStreamStopped();
            }
        };
    }

    //endregion
}
