package com.github.yoep.player.chromecast;

import com.github.yoep.player.chromecast.services.ChromecastService;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;
import org.springframework.util.Assert;
import su.litvak.chromecast.api.v2.ChromeCast;
import su.litvak.chromecast.api.v2.ChromeCastSpontaneousEvent;
import su.litvak.chromecast.api.v2.ChromeCastSpontaneousEventListener;
import su.litvak.chromecast.api.v2.MediaStatus;

import java.io.IOException;
import java.security.GeneralSecurityException;
import java.util.Objects;
import java.util.Optional;
import java.util.Timer;
import java.util.TimerTask;

@Slf4j
@ToString(exclude = {"playerState", "chromeCast", "listener"})
@EqualsAndHashCode(exclude = {"playerState", "chromeCast", "listener"}, callSuper = true)
@RequiredArgsConstructor
public class ChromecastPlayer extends AbstractListenerService<PlayerListener> implements Player {
    static final Resource GRAPHIC_RESOURCE = new ClassPathResource("/external-chromecast-icon.png");
    static final String MEDIA_RECEIVER_APP_ID = "CC1AD845";
    static final String MEDIA_NAMESPACE = "urn:x-cast:com.google.cast.media";
    static final String DESCRIPTION = "Chromecast streaming media device which allows the playback of videos on your TV.";

    private final ChromeCastSpontaneousEventListener listener = createEventListener();
    private final ChromeCast chromeCast;
    private final ChromecastService service;

    private PlayerState playerState = PlayerState.READY;
    private PlaybackThread playbackThread;
    private Timer statusTimer;
    private String sessionId;
    private Double originalLoadDuration;
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
    public String getDescription() {
        return DESCRIPTION;
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
        // move this operation to a separate thread
        // as the stop command on the chromecast may timeout
        new Thread(() -> {
            stopPreviousPlaybackThreadIfNeeded();
            stopApp();
        }, "Chromecast-command").start();

        Optional.ofNullable(statusTimer)
                .ifPresent(Timer::cancel);

        service.stop();
        updateState(PlayerState.STOPPED);
    }

    @Override
    public void seek(long time) {
        try {
            chromeCast.seek(service.toChromecastTime(time));
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
            if (!chromeCast.isAppRunning(MEDIA_RECEIVER_APP_ID)) {
                log.debug("Launching Chromecast application {} on \"{}\"", MEDIA_RECEIVER_APP_ID, getName());
                var application = chromeCast.launchApp(MEDIA_RECEIVER_APP_ID);

                if (application != null) {
                    sessionId = application.sessionId;
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
            appLaunched = false;
            sessionId = null;
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
            case LOADING -> updateState(PlayerState.LOADING);
            case PLAYING -> updateState(PlayerState.PLAYING);
            case PAUSED -> updateState(PlayerState.PAUSED);
            case BUFFERING -> updateState(PlayerState.BUFFERING);
        }
    }

    private void onPlayerTimeChanged(double currentTime) {
        invokeListeners(e -> e.onTimeChanged(service.toApplicationTime(currentTime)));
    }

    private void onPlayerDurationChanged(Double duration) {
        Optional.ofNullable(duration)
                .map(e -> e == Double.MAX_VALUE ? originalLoadDuration : e)
                .map(Double::longValue)
                .map(service::toApplicationTime)
                .ifPresent(e -> invokeListeners(listener -> listener.onDurationChanged(e)));
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
        invokeListeners(e -> e.onStateChanged(state));
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

            try {
                log.debug("Loading url \"{}\" on Chromecast \"{}\"", url, getName());
                updateState(PlayerState.LOADING);

                var loadRequest = service.toLoadRequest(sessionId, request);
                // store the original duration if we need it later on
                // due to the transcoding being a live stream instead of a buffered one
                originalLoadDuration = loadRequest.getMedia().getDuration();
                log.trace("Sending load request to Chromecast, {}", loadRequest);
                chromeCast.send(MEDIA_NAMESPACE, loadRequest);

                var statusThread = new PlaybackStatusThread();
                statusTimer = new Timer("ChromecastPlaybackStatus");
                statusTimer.schedule(statusThread, 0, 1000);

                while (keepAlive) {
                    Thread.onSpinWait();
                }
            } catch (Exception ex) {
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
