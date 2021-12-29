package com.github.yoep.player.chromecast;

import com.github.yoep.player.chromecast.model.VideoMetadata;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;
import org.springframework.http.MediaType;
import org.springframework.lang.Nullable;
import org.springframework.util.Assert;
import su.litvak.chromecast.api.v2.*;

import java.io.IOException;
import java.net.URI;
import java.security.GeneralSecurityException;
import java.util.*;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

@Slf4j
@ToString(exclude = {"playerState", "chromeCast", "listener"})
@EqualsAndHashCode(exclude = {"playerState", "chromeCast", "listener"})
@RequiredArgsConstructor
public class ChromecastPlayer implements Player {
    private static final Resource GRAPHIC_RESOURCE = new ClassPathResource("/external-chromecast-icon.png");
    private static final String APP_ID = "CC1AD845";
    private static final String METADATA_THUMBNAIL = "thumb";
    private static final String METADATA_THUMBNAIL_URL = "thumbnailUrl";
    private static final String METADATA_POSTER_URL = "posterUrl";

    private final ChromeCastSpontaneousEventListener listener = createEventListener();
    private final Collection<PlayerListener> listeners = new ConcurrentLinkedQueue<>();
    private final Timer statusTimer = new Timer("ChromecastPlaybackStatus");
    private final ChromeCast chromeCast;
    @Nullable
    private final ChromecastContentTypeResolver contentTypeResolver;

    private PlayerState playerState = PlayerState.UNKNOWN;
    private PlaybackThread playbackThread;
    private boolean connected;
    private boolean appLaunched;

    //region ExternalPlayer

    @Override
    public String getId() {
        return chromeCast.getName();
    }

    @Override
    public String getName() {
        return chromeCast.getTitle();
    }

    @Override
    public Optional<Resource> getGraphicResource() {
        return Optional.of(GRAPHIC_RESOURCE);
    }

    @Override
    public PlayerState getState() {
        return playerState;
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return false;
    }

    @Override
    public void dispose() {
        log.trace("Disposing Chromecast player \"{}\"", getName());
        if (appLaunched) {
            stopApp();
        }
        if (connected) {
            disconnect();
        }
    }

    @Override
    public void addListener(PlayerListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(PlayerListener listener) {
        listeners.remove(listener);
    }

    @Override
    public void play(PlayRequest request) {
        Assert.notNull(request, "request cannot be null");

        // check if we need to stop the previous playback thread
        stopPreviousPlaybackThreadIfNeeded();

        // run the request on a separate thread to unblock the UI
        playbackThread = new PlaybackThread(request);
        playbackThread.start();
    }

    @Override
    public void resume() {
        try {
            chromeCast.play();
        } catch (IOException ex) {
            log.error("Failed to resume the Chromecast playback, {}", ex.getMessage(), ex);
        }
    }

    @Override
    public void pause() {
        try {
            chromeCast.pause();
        } catch (IOException ex) {
            log.error("Failed to pause the Chromecast playback, {}", ex.getMessage(), ex);
        }
    }

    @Override
    public void stop() {
        try {
            stopPreviousPlaybackThreadIfNeeded();
            chromeCast.stopApp();
        } catch (IOException ex) {
            log.error("Failed to stop the Chromecast playback, {}", ex.getMessage(), ex);
        }

        statusTimer.cancel();
        statusTimer.purge();
        updateState(PlayerState.STOPPED);
    }

    @Override
    public void seek(long time) {
        try {
            chromeCast.seek(time);
        } catch (IOException ex) {
            log.error("Failed to seek within the Chromecast playback, {}", ex.getMessage(), ex);
        }
    }

    @Override
    public void volume(int volume) {
        Assert.state(volume >= 0 && volume <= 100, "Volume level must be between 0 and 100");
        var volumeLevel = volume / 100f;

        try {
            chromeCast.setVolume(volumeLevel);
        } catch (IOException ex) {
            log.error("Failed to set the volume of the Chromecast device \"{}\", {}", getName(), ex.getMessage(), ex);
        }
    }

    //endregion

    //region Functions

    private Map<String, Object> getMediaMetaData(PlayRequest request) {
        var metadata = new HashMap<String, Object>();
        metadata.put(Media.METADATA_TYPE, Media.MetadataType.MOVIE);
        metadata.put(Media.METADATA_TITLE, request.getTitle().orElse(null));
        metadata.put(METADATA_THUMBNAIL, request.getThumbnail().orElse(null));
        metadata.put(METADATA_THUMBNAIL_URL, request.getThumbnail().orElse(null));
        metadata.put(METADATA_POSTER_URL, request.getThumbnail().orElse(null));
        return metadata;
    }

    private List<Track> getMediaTracks(PlayRequest request) {
        // check if a subtitle track is provided
        // if so, add it to the media
        return request.getSubtitle()
                .map(e -> new Track(1, Track.TrackType.TEXT))
                .map(Collections::singletonList)
                .orElse(Collections.emptyList());
    }

    private VideoMetadata resolveVideoMetaData(String url) {
        // check if a content type resolver has been provided
        // if not, use the octet stream as fallback value
        if (contentTypeResolver == null) {
            return VideoMetadata.builder()
                    .contentType(MediaType.APPLICATION_OCTET_STREAM_VALUE)
                    .duration(VideoMetadata.UNKNOWN_DURATION)
                    .build();
        }

        return contentTypeResolver.resolve(URI.create(url));
    }

    private void prepareDeviceIfNeeded() {
        // check if a connection to the Chromecast device has been made
        // if not, try to create a new connection
        if (!connected) {
            connect();
        }

        // check if the media playback app has been made
        // if not, try to launch the Chromecast app
        if (!appLaunched) {
            launchApp();
        }
    }

    private void stopPreviousPlaybackThreadIfNeeded() {
        if (playbackThread != null) {
            playbackThread.stopPlayback();
        }
    }

    private void connect() {
        if (connected) {
            log.warn("Already connected to the Chromecast, ignoring connection attempt");
            return;
        }

        try {
            log.debug("Connecting to Chromecast \"{}\"", getName());
            chromeCast.connect();
            chromeCast.registerListener(listener);
            log.info("Successfully connected to Chromecast \"{}\"", getName());
            connected = true;

            launchApp();
        } catch (IOException | GeneralSecurityException ex) {
            log.error("Failed to connect to Chromecast \"{}\", {}", getName(), ex.getMessage(), ex);
        }
    }

    private void launchApp() {
        if (appLaunched) {
            log.warn("Chromecast app has already been launched, ignoring launch attempt");
            return;
        }

        try {
            if (!chromeCast.isAppRunning(APP_ID)) {
                log.debug("Launching Chromecast application {} on \"{}\"", APP_ID, getName());
                var application = chromeCast.launchApp(APP_ID);

                if (application != null) {
                    appLaunched = true;
                } else {
                    log.error("Failed to launch Chromecast app");
                }
            }
        } catch (IOException ex) {
            log.error("Failed to launch application on Chromecast \"{}\", {}", getName(), ex.getMessage(), ex);
        }
    }

    private void stopApp() {
        try {
            log.trace("Trying to stop currently running app on Chromecast \"{}\"", getName());
            chromeCast.stopApp();
            log.debug("Stopped app on the Chromecast \"{}\"", getName());
        } catch (IOException ex) {
            log.error("Failed to stop app on Chromecast \"{}\", {}", getName(), ex.getMessage(), ex);
        }
    }

    private void disconnect() {
        try {
            log.trace("Trying to disconnect from Chromecast \"{}\"", getName());
            chromeCast.unregisterListener(listener);
            chromeCast.disconnect();
            log.info("Disconnected from Chromecast \"{}\"", getName());
        } catch (IOException ex) {
            log.error("Failed to disconnect from Chromecast \"{}\", {}", getName(), ex.getMessage(), ex);
        }
    }

    private void onMediaStatusChanged(MediaStatus status) {
        if (status == null) {
            log.warn("Received invalid media status from Chromecast");
            return;
        }

        log.trace("Received chromecast media status update {}", status);
        onPlayerStateChanged(status.playerState);
        onPlayerTimeChanged(status.currentTime);
        onPlayerDurationChanged(status.media.duration);
    }

    private void onPlayerStateChanged(MediaStatus.PlayerState status) {
        switch (status) {
            case LOADING:
                updateState(PlayerState.LOADING);
                break;
            case PLAYING:
                updateState(PlayerState.PLAYING);
                break;
            case PAUSED:
                updateState(PlayerState.PAUSED);
                break;
            case BUFFERING:
                updateState(PlayerState.BUFFERING);
                break;
        }
    }

    private void onPlayerTimeChanged(double currentTime) {
        invokeSafeListeners(e -> e.onTimeChanged((long) currentTime * 1000));
    }

    private void onPlayerDurationChanged(Double duration) {
        invokeSafeListeners(e -> e.onDurationChanged(duration.longValue() * 1000));
    }

    private ChromeCastSpontaneousEventListener createEventListener() {
        return event -> {
            if (event.getType() == ChromeCastSpontaneousEvent.SpontaneousEventType.MEDIA_STATUS) {
                var status = event.getData(MediaStatus.class);
                onMediaStatusChanged(status);
            }
        };
    }

    private void updateState(PlayerState state) {
        playerState = state;
        invokeSafeListeners(e -> e.onStateChanged(state));
    }

    private void invokeSafeListeners(Consumer<PlayerListener> action) {
        listeners.forEach(e -> {
            try {
                action.accept(e);
            } catch (Exception ex) {
                log.warn("Failed to invoked player listener, {}", ex.getMessage(), ex);
            }
        });
    }

    //endregion

    /**
     * Internal playback thread which unloads the {@link ChromeCast} workload to a new separate thread
     * to allow the current calling thread to be unblocked.
     */
    private class PlaybackThread extends Thread {
        private final PlayRequest request;
        private volatile boolean keepAlive = true;

        private PlaybackThread(PlayRequest request) {
            super("ChromecastPlayback");
            Objects.requireNonNull(request, "request cannot be null");
            this.request = request;
        }

        @Override
        public void run() {
            // prepare the chromecast device if needed
            prepareDeviceIfNeeded();
            var url = request.getUrl();
            var videoMetadata = resolveVideoMetaData(url);

            try {
                log.debug("Loading url \"{}\" on Chromecast \"{}\"", url, getName());
                updateState(PlayerState.LOADING);

                var tracks = getMediaTracks(request);
                var metadata = getMediaMetaData(request);
                var media = new Media(url, videoMetadata.getContentType(), videoMetadata.getDuration().doubleValue(), Media.StreamType.BUFFERED,
                        null, metadata, null, tracks);

                var status = chromeCast.load(media);
                log.debug("Received status {}", status);

                var statusThread = new PlaybackStatusThread();
                statusTimer.schedule(statusThread, 0, 1000);

                while (keepAlive) {
                    Thread.onSpinWait();
                }
            } catch (IOException ex) {
                log.error("Failed to play url on Chromecast \"{}\", {}", getName(), ex.getMessage(), ex);
                updateState(PlayerState.ERROR);
            }
        }

        public void stopPlayback() {
            keepAlive = false;
        }
    }

    /**
     * Internal task which retrieves the latest media status of the current playback from
     * the {@link ChromeCast} device.
     */
    private class PlaybackStatusThread extends TimerTask {
        @Override
        public void run() {
            try {
                var mediaStatus = chromeCast.getMediaStatus();
                onMediaStatusChanged(mediaStatus);
            } catch (IOException e) {
                log.warn("Failed to retrieve chromecast media status, {}", e.getMessage(), e);
            }
        }
    }
}
