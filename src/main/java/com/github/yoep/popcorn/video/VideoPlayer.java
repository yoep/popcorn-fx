package com.github.yoep.popcorn.video;

import javafx.application.Platform;
import javafx.scene.image.ImageView;
import javafx.scene.image.PixelBuffer;
import javafx.scene.image.PixelFormat;
import javafx.scene.image.WritableImage;
import javafx.scene.layout.Pane;
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

public class VideoPlayer {
    private final FXCallbackVideoSurface surface = new FXCallbackVideoSurface();
    private final MediaPlayerFactory mediaPlayerFactory;
    private final EmbeddedMediaPlayer embeddedMediaPlayer;
    private final ImageView videoImageView;

    private PixelBuffer<ByteBuffer> videoPixelBuffer;
    private WritableImage videoImage;

    /**
     * Instantiate a new video player on the given {@link ImageView}.
     *
     * @param videoImageView The image view to use as the video player.
     */
    public VideoPlayer(ImageView videoImageView) {
        Assert.notNull(videoImageView, "videoImageView cannot be null");
        this.mediaPlayerFactory = new MediaPlayerFactory();
        this.embeddedMediaPlayer = mediaPlayerFactory.mediaPlayers().newEmbeddedMediaPlayer();
        this.videoImageView = videoImageView;

        init();
    }

    /**
     * Play the given media url.
     *
     * @param url The media url to play.
     */
    public void play(String url) {
        this.embeddedMediaPlayer.media().play(url);
    }

    /**
     * Stop the current media playback.
     */
    public void stop() {
        this.embeddedMediaPlayer.controls().stop();
        this.embeddedMediaPlayer.release();
        this.mediaPlayerFactory.release();
    }

    private void init() {
        Pane parentPane = (Pane) this.videoImageView.getParent();

        this.videoImageView.setPreserveRatio(true);
        this.videoImageView.fitWidthProperty().bind(parentPane.widthProperty());
        this.videoImageView.fitHeightProperty().bind(parentPane.heightProperty());

        this.embeddedMediaPlayer.videoSurface().set(surface);
    }

    private class FXCallbackVideoSurface extends CallbackVideoSurface {
        FXCallbackVideoSurface() {
            super(new FXBufferFormatCallback(), new FXRenderCallback(), true, VideoSurfaceAdapters.getVideoSurfaceAdapter());
        }
    }

    private class FXBufferFormatCallback implements BufferFormatCallback {
        private int sourceWidth;
        private int sourceHeight;

        @Override
        public BufferFormat getBufferFormat(int sourceWidth, int sourceHeight) {
            this.sourceWidth = sourceWidth;
            this.sourceHeight = sourceHeight;
            return new RV32BufferFormat(sourceWidth, sourceHeight);
        }

        @Override
        public void allocatedBuffers(ByteBuffer[] buffers) {
            assert buffers[0].capacity() == sourceWidth * sourceHeight * 4;
            PixelFormat<ByteBuffer> pixelFormat = PixelFormat.getByteBgraPreInstance();
            videoPixelBuffer = new PixelBuffer<>(sourceWidth, sourceHeight, buffers[0], pixelFormat);
            videoImage = new WritableImage(videoPixelBuffer);
            videoImageView.setImage(videoImage);
        }
    }

    private class FXRenderCallback implements RenderCallback {
        @Override
        public void display(MediaPlayer mediaPlayer, ByteBuffer[] nativeBuffers, BufferFormat bufferFormat) {
            Platform.runLater(() -> videoPixelBuffer.updateBuffer(pb -> null));
        }
    }
}
