package com.github.yoep.video.vlc;

import com.github.yoep.popcorn.backend.adapters.video.AbstractVideoPlayer;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerException;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerNotInitializedException;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import javafx.scene.Node;
import javafx.scene.image.ImageView;
import javafx.scene.layout.Pane;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;
import org.springframework.util.StringUtils;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;

import javax.annotation.PostConstruct;
import java.io.File;
import java.util.Objects;

import static uk.co.caprica.vlcj.binding.LibVlc.libvlc_errmsg;
import static uk.co.caprica.vlcj.javafx.videosurface.ImageViewVideoSurfaceFactory.videoSurfaceForImageView;

@Slf4j
@ToString
@EqualsAndHashCode(callSuper = true)
public class VideoPlayerVlc extends AbstractVideoPlayer implements VideoPlayback {
    static final String NAME = "VLC";
    static final String DESCRIPTION = "Video backend which uses the VLC library for video playbacks.";

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
        Objects.requireNonNull(mediaPlayerFactory, "mediaPlayerFactory cannot be null");
        this.mediaPlayerFactory = mediaPlayerFactory;
        this.mediaPlayer = mediaPlayerFactory.mediaPlayers().newEmbeddedMediaPlayer();

        initialize();
    }

    //endregion

    //region Getters

    @Override
    public String getName() {
        return NAME;
    }

    @Override
    public String getDescription() {
        return DESCRIPTION;
    }

    @Override
    public boolean supports(String url) {
        return isInitialized() && StringUtils.hasText(url);
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
    public void addListener(VideoListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(VideoListener listener) {
        listeners.remove(listener);
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
        setVideoState(VideoState.READY);
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
                setVideoState(VideoState.PLAYING);
            }

            @Override
            public void paused(MediaPlayer mediaPlayer) {
                setVideoState(VideoState.PAUSED);
            }

            @Override
            public void stopped(MediaPlayer mediaPlayer) {
                setVideoState(VideoState.STOPPED);
            }

            @Override
            public void finished(MediaPlayer mediaPlayer) {
                setVideoState(VideoState.FINISHED);
            }

            @Override
            public void buffering(MediaPlayer mediaPlayer, float newCache) {
                log.trace("VLC buffer is now {}%", newCache);
                if (newCache < 100) {
                    setVideoState(VideoState.BUFFERING);
                } else {
                    setVideoState(VideoState.PLAYING);
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
                setTime(newTime);
            }

            @Override
            public void lengthChanged(MediaPlayer mediaPlayer, long newLength) {
                setDuration(newLength);
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

        setTime(0L);
        setDuration(0L);
    }

    private void setError(Throwable throwable) {
        this.error = throwable;
        setVideoState(VideoState.ERROR);
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
