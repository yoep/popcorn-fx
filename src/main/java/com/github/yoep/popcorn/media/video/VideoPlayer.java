package com.github.yoep.popcorn.media.video;

import com.github.yoep.popcorn.media.video.state.PlayerState;
import com.github.yoep.popcorn.media.video.state.PlayerStateHolder;
import com.github.yoep.popcorn.media.video.state.PlayerStateListener;
import com.github.yoep.popcorn.media.video.time.TimeHolder;
import com.github.yoep.popcorn.media.video.time.TimeListener;
import javafx.application.Platform;
import javafx.geometry.Rectangle2D;
import javafx.scene.canvas.Canvas;
import javafx.scene.canvas.GraphicsContext;
import javafx.scene.image.PixelBuffer;
import javafx.scene.image.PixelFormat;
import javafx.scene.image.WritableImage;
import javafx.scene.image.WritablePixelFormat;
import javafx.scene.layout.Pane;
import javafx.scene.paint.Color;
import javafx.scene.transform.Affine;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;
import uk.co.caprica.vlcj.player.embedded.videosurface.CallbackVideoSurface;
import uk.co.caprica.vlcj.player.embedded.videosurface.VideoSurfaceAdapters;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.BufferFormat;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.BufferFormatCallback;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.RenderCallback;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.format.RV32BufferFormat;

import java.nio.ByteBuffer;

@Slf4j
public class VideoPlayer {
    private final WritablePixelFormat<ByteBuffer> pixelFormat = PixelFormat.getByteBgraPreInstance();
    private final VideoAnimationTimer timer = new VideoAnimationTimer(this::renderFrame);
    private final Canvas canvas = new Canvas();

    private final FXCallbackVideoSurface surface;
    private final MediaPlayerFactory mediaPlayerFactory;
    private final EmbeddedMediaPlayer mediaPlayer;
    private final PlayerStateHolder playerStateHolder;
    private final TimeHolder timeHolder;
    private final Pane canvasPane;

    private PixelBuffer<ByteBuffer> videoPixelBuffer;
    private WritableImage videoImage;
    private Rectangle2D updatedBuffer;

    private int bufferWidth;
    private int bufferHeight;

    /**
     * Instantiate a new video player in the given {@link Pane}.
     *
     * @param canvasPane The pane to attach the video canvas to.
     */
    public VideoPlayer(Pane canvasPane) {
        Assert.notNull(canvasPane, "parentPane cannot be null");
        this.canvasPane = canvasPane;

        this.surface = new FXCallbackVideoSurface();
        this.mediaPlayerFactory = new MediaPlayerFactory();
        this.mediaPlayer = mediaPlayerFactory.mediaPlayers().newEmbeddedMediaPlayer();
        this.playerStateHolder = new PlayerStateHolder(this.mediaPlayer);
        this.timeHolder = new TimeHolder(this.mediaPlayer);

        init();
    }

    /**
     * Get the current state of the video player.
     *
     * @return Returns the state of the player.
     */
    public PlayerState getPlayerState() {
        return playerStateHolder.getState();
    }

    /**
     * Add the given listener to this video player.
     *
     * @param listener The player state change listener .
     */
    public void addListener(PlayerStateListener listener) {
        playerStateHolder.addListener(listener);
    }

    /**
     * Add the given listener to this video player.
     *
     * @param listener The time change listener.
     */
    public void addListener(TimeListener listener) {
        timeHolder.addListener(listener);
    }

    /**
     * Play the given media url.
     *
     * @param url The media url to play.
     */
    public void play(String url) {
        timer.start();
        mediaPlayer.media().play(url);
    }

    /**
     * Pause the media playback.
     */
    public void pause() {
        timer.stop();
        mediaPlayer.controls().pause();
    }

    /**
     * Resume the media playback.
     */
    public void resume() {
        timer.start();
        mediaPlayer.controls().play();
    }

    /**
     * Stop the current media playback.
     */
    public void stop() {
        this.timer.stop();
        this.mediaPlayer.controls().stop();
        renderBlackFrame();
    }

    /**
     * Jump to the given time in the video player.
     *
     * @param time The new time in millis.
     */
    public void setTime(long time) {
        mediaPlayer.controls().setTime(time);
    }

    public void dispose() {
        stop();
        mediaPlayer.release();
        mediaPlayerFactory.release();
    }

    private void init() {
        this.canvas.widthProperty().bind(canvasPane.widthProperty());
        this.canvas.heightProperty().bind(canvasPane.heightProperty());
        this.canvas.widthProperty().addListener(event -> {
            if (!mediaPlayer.status().isPlaying()) renderFrame();
        });
        this.canvas.heightProperty().addListener(event -> {
            if (!mediaPlayer.status().isPlaying()) renderFrame();
        });
        this.canvasPane.getChildren().add(this.canvas);
        this.mediaPlayer.videoSurface().set(surface);
    }

    private void renderFrame() {
        GraphicsContext g = canvas.getGraphicsContext2D();

        double width = canvas.getWidth();
        double height = canvas.getHeight();

        renderBlackFrame();

        if (videoImage != null) {
            double imageWidth = videoImage.getWidth();
            double imageHeight = videoImage.getHeight();

            double sx = width / imageWidth;
            double sy = height / imageHeight;

            double sf = Math.min(sx, sy);

            double scaledW = imageWidth * sf;
            double scaledH = imageHeight * sf;

            Affine ax = g.getTransform();

            g.translate(
                    (width - scaledW) / 2,
                    (height - scaledH) / 2
            );

            if (sf != 1.0) {
                g.scale(sf, sf);
            }

            g.drawImage(videoImage, 0, 0);

            g.setTransform(ax);
        }
    }

    private void renderBlackFrame() {
        GraphicsContext g = canvas.getGraphicsContext2D();

        double width = canvas.getWidth();
        double height = canvas.getHeight();

        g.setFill(Color.BLACK);
        g.fillRect(0, 0, width, height);
    }

    private class FXCallbackVideoSurface extends CallbackVideoSurface {
        FXCallbackVideoSurface() {
            super(new FXBufferFormatCallback(), new FXRenderCallback(), true, VideoSurfaceAdapters.getVideoSurfaceAdapter());
        }
    }

    private class FXBufferFormatCallback implements BufferFormatCallback {
        @Override
        public BufferFormat getBufferFormat(int sourceWidth, int sourceHeight) {
            bufferWidth = sourceWidth;
            bufferHeight = sourceHeight;

            // This does not need to be done here, but you could set the video surface size to match the native video
            // size

            return new RV32BufferFormat(sourceWidth, sourceHeight);
        }

        @Override
        public void allocatedBuffers(ByteBuffer[] buffers) {
            try {
                // This is the new magic sauce, the native video buffer is used directly for the image buffer - there is no
                // full-frame buffer copy here
                videoPixelBuffer = new PixelBuffer<>(bufferWidth, bufferHeight, buffers[0], pixelFormat);
                videoImage = new WritableImage(videoPixelBuffer);
                // Since for every frame the entire buffer will be updated, we can optimise by caching the result here
                updatedBuffer = new Rectangle2D(0, 0, bufferWidth, bufferHeight);
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        }
    }

    private class FXRenderCallback implements RenderCallback {
        @Override
        public void display(MediaPlayer mediaPlayer, ByteBuffer[] nativeBuffers, BufferFormat bufferFormat) {
            Platform.runLater(() -> {
                try {
                    videoPixelBuffer.updateBuffer(pixBuf -> updatedBuffer);
                } catch (Exception ex) {
                    log.error(ex.getMessage(), ex);
                }
            });
        }
    }
}
