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
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;

import javax.annotation.PostConstruct;
import java.io.File;

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
     * Instantiate a new video player.
     */
    public VideoPlayerVlc(NativeDiscovery nativeDiscovery) {
        mediaPlayerFactory = new MediaPlayerFactory(nativeDiscovery);
        mediaPlayer = mediaPlayerFactory.mediaPlayers().newEmbeddedMediaPlayer();

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

    protected void setTime(long time) {
        this.time.set(time);
    }

    @Override
    public long getDuration() {
        return duration.get();
    }

    @Override
    public ReadOnlyLongProperty durationProperty() {
        return duration;
    }

    protected void setDuration(long duration) {
        this.duration.set(duration);
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
    private void init() {
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

        setTime(0);
        setDuration(0);
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
