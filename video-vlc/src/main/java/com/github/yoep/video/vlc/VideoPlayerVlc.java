package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.beans.property.*;
import javafx.scene.Node;
import javafx.scene.image.ImageView;
import javafx.scene.layout.Pane;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.StringUtils;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;

import javax.annotation.PostConstruct;
import java.io.File;

import static uk.co.caprica.vlcj.binding.LibVlc.libvlc_errmsg;
import static uk.co.caprica.vlcj.javafx.videosurface.ImageViewVideoSurfaceFactory.videoSurfaceForImageView;

@Slf4j
@ToString
@EqualsAndHashCode
public class VideoPlayerVlc implements VideoPlayer {
    private final ObjectProperty<PlayerState> playerState = new SimpleObjectProperty<>(this, PLAYER_STATE_PROPERTY, PlayerState.UNKNOWN);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY);
    private final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY);
    private final ImageView videoSurface = new ImageView();

    private final MediaPlayerFactory mediaPlayerFactory;
    private final EmbeddedMediaPlayer mediaPlayer;

    private Throwable error;
    private boolean bound;
    private boolean initialized;

    //region Constructors

    /**
     * Initialize a new {@link VideoPlayerVlc} instance.
     *
     * @param mediaPlayerFactory The VLC media player factory to use.
     */
    public VideoPlayerVlc(MediaPlayerFactory mediaPlayerFactory) {
        this.mediaPlayerFactory = mediaPlayerFactory;
        this.mediaPlayer = mediaPlayerFactory.mediaPlayers().newEmbeddedMediaPlayer();

        initialize();
    }

    //endregion

    //region Properties

    @Override
    public PlayerState getPlayerState() {
        return playerState.get();
    }

    @Override
    public ReadOnlyObjectProperty<PlayerState> playerStateProperty() {
        return playerState;
    }

    @Override
    public long getTime() {
        return time.get();
    }

    @Override
    public ReadOnlyLongProperty timeProperty() {
        return time;
    }

    @Override
    public long getDuration() {
        return duration.get();
    }

    @Override
    public ReadOnlyLongProperty durationProperty() {
        return duration;
    }

    //endregion

    //region Getters

    @Override
    public boolean supports(String url) {
        return !StringUtils.isEmpty(url);
    }

    @Override
    public boolean isInitialized() {
        return initialized;
    }

    @Override
    public Throwable getError() {
        return error;
    }

    @Override
    public Node getVideoSurface() {
        return videoSurface;
    }

    //endregion

    //region VideoPlayer

    @Override
    public void dispose() {
        stop();
        mediaPlayer.release();
        mediaPlayerFactory.release();
    }

    @Override
    public void play(String url) {
        checkInitialized();

        log.debug("Playing \"{}\" on VLC video player", url);
        invokeOnVlc(() -> mediaPlayer.media().play(url));
    }

    @Override
    public void pause() {
        checkInitialized();

        invokeOnVlc(() -> mediaPlayer.controls().pause());
    }

    @Override
    public void resume() {
        checkInitialized();

        invokeOnVlc(() -> mediaPlayer.controls().play());
    }

    @Override
    public void seek(long time) {
        checkInitialized();

        invokeOnVlc(() -> mediaPlayer.controls().setTime(time));
    }

    @Override
    public void stop() {
        checkInitialized();

        invokeOnVlc(() -> mediaPlayer.controls().stop());
        reset();
    }

    @Override
    public boolean supportsNativeSubtitleFile() {
        return false;
    }

    @Override
    public void subtitleFile(File file) {
        mediaPlayer.subpictures().setSubTitleFile(file);
    }

    @Override
    public void subtitleDelay(long delay) {
        log.trace("Updated subtitle delay to {} milliseconds", delay);
        mediaPlayer.subpictures().setDelay(delay * 1000);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        log.trace("Initializing VLC player");

        try {
            this.mediaPlayer.videoSurface().set(videoSurfaceForImageView(videoSurface));

            initialized = true;
            log.trace("VLC player initialization done");
        } catch (Exception ex) {
            log.error("Failed to initialize VLC player, " + ex.getMessage(), ex);
            setError(new VideoPlayerException(ex.getMessage(), ex));
        }
    }

    //endregion

    //region Functions

    private void initialize() {
        initializeListeners();
        initializeEvents();
        initializeVideoSurface();
    }

    private void initializeListeners() {
        videoSurface.parentProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null && !bound) {
                var parent = (Pane) newValue;

                bindToParent(parent);
            }
        });
    }

    private void initializeEvents() {
        mediaPlayer.events().addMediaPlayerEventListener(new MediaPlayerEventAdapter() {
            @Override
            public void playing(MediaPlayer mediaPlayer) {
                playerState.set(PlayerState.PLAYING);
            }

            @Override
            public void paused(MediaPlayer mediaPlayer) {
                playerState.set(PlayerState.PAUSED);
            }

            @Override
            public void stopped(MediaPlayer mediaPlayer) {
                playerState.set(PlayerState.STOPPED);
            }

            @Override
            public void finished(MediaPlayer mediaPlayer) {
                playerState.set(PlayerState.FINISHED);
            }

            @Override
            public void buffering(MediaPlayer mediaPlayer, float newCache) {
                log.trace("VLC buffer is now {}%", newCache);
                if (newCache < 100) {
                    playerState.set(PlayerState.BUFFERING);
                } else {
                    playerState.set(PlayerState.PLAYING);
                }
            }

            @Override
            public void error(MediaPlayer mediaPlayer) {
                var message = libvlc_errmsg();

                log.error("VLC error encountered, error: {}", message);
                setError(new VideoPlayerException("VLC media player went into error state, error: " + message));
            }

            @Override
            public void timeChanged(MediaPlayer mediaPlayer, long newTime) {
                time.set(newTime);
            }

            @Override
            public void lengthChanged(MediaPlayer mediaPlayer, long newLength) {
                duration.set(newLength);
            }
        });
    }

    private void initializeVideoSurface() {
        videoSurface.setPreserveRatio(true);
    }

    private void bindToParent(Pane parent) {
        parent.widthProperty().addListener((observable, oldValue, newValue) -> videoSurface.setFitWidth(newValue.longValue()));
        parent.heightProperty().addListener((observable, oldValue, newValue) -> videoSurface.setFitHeight(newValue.longValue()));

        bound = true;
    }

    private void reset() {
        error = null;

        time.set(0);
        duration.set(0);
    }

    private void setError(Throwable throwable) {
        this.error = throwable;
        playerState.set(PlayerState.ERROR);
    }

    private void checkInitialized() {
        if (!initialized)
            throw new VideoPlayerNotInitializedException(this);
    }

    private void invokeOnVlc(Runnable runnable) {
        mediaPlayer.submit(() -> {
            try {
                runnable.run();
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
                setError(new VideoPlayerException(ex.getMessage(), ex));
            }
        });
    }

    //endregion
}
