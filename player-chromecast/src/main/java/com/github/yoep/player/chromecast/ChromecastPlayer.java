package com.github.yoep.player.chromecast;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
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
import java.util.Collection;
import java.util.Optional;
import java.util.concurrent.ConcurrentLinkedQueue;

@Slf4j
@ToString(exclude = {"playerState", "chromeCast", "listener"})
@EqualsAndHashCode(exclude = {"playerState", "chromeCast", "listener"})
@RequiredArgsConstructor
public class ChromecastPlayer implements Player {
    private static final Resource GRAPHIC_RESOURCE = new ClassPathResource("/external-chromecast-icon.png");
    private static final String APP_ID = "CC1AD845";

    private final ChromeCastSpontaneousEventListener listener = createEventListener();
    private final Collection<PlayerListener> listeners = new ConcurrentLinkedQueue<>();
    private final ChromeCast chromeCast;

    private PlayerState playerState = PlayerState.UNKNOWN;
    private Thread playbackThread;
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
        var url = request.getUrl();

        // check if we need to stop the previous playback thread
        stopPreviousPlaybackThreadIfNeeded();

        // run the request on a separate thread to unblock the UI
        playbackThread = new Thread(() -> {
            // prepare the chromecast device if needed
            prepareDeviceIfNeeded();

            try {
                log.debug("Loading url \"{}\" on Chromecast \"{}\"", url, getName());
                updateState(PlayerState.LOADING);
                var status = chromeCast.load(url);
                log.debug("Received status {}", status);
            } catch (IOException ex) {
                log.error("Failed to play url on Chromecast \"{}\", {}", getName(), ex.getMessage(), ex);
                updateState(PlayerState.ERROR);
            }
        }, "ChromecastPlayback");
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
        if (playbackThread != null && playbackThread.isAlive()) {
            log.debug("Interrupting the current Chromecast playback thread");
            playbackThread.interrupt();
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

        switch (status.playerState) {
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
        listeners.forEach(e -> e.onStateChanged(state));
    }

    //endregion
}
