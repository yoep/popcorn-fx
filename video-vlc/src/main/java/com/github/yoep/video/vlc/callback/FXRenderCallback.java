package com.github.yoep.video.vlc.callback;

import javafx.application.Platform;
import javafx.scene.canvas.Canvas;
import javafx.scene.paint.Color;
import javafx.scene.transform.Affine;
import lombok.Data;
import lombok.extern.slf4j.Slf4j;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.BufferFormat;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.RenderCallback;

import java.nio.ByteBuffer;

@Slf4j
@Data
public class FXRenderCallback implements RenderCallback {
    private final Canvas canvas;
    private final FXBufferFormatCallback bufferFormat;

    /**
     * Initialize a new Render callback instance.
     *
     * @param canvas       The canvas to use for the rendering of the video.
     * @param bufferFormat The buffer format to use when rendering the video.
     */
    public FXRenderCallback(Canvas canvas, FXBufferFormatCallback bufferFormat) {
        this.canvas = canvas;
        this.bufferFormat = bufferFormat;

        initialize();
    }

    @Override
    public void display(MediaPlayer mediaPlayer, ByteBuffer[] nativeBuffers, BufferFormat bufferFormat) {
        Platform.runLater(() -> {
            try {
                this.bufferFormat.getVideoPixelBuffer().updateBuffer(pixBuf -> this.bufferFormat.getUpdatedBuffer());
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    /**
     * Render a black screen (frame) in the canvas.
     */
    public void renderBlackFrame() {
        var g = canvas.getGraphicsContext2D();

        double width = canvas.getWidth();
        double height = canvas.getHeight();

        g.setFill(Color.BLACK);
        g.fillRect(0, 0, width, height);
    }

    /**
     * Render a new frame in the canvas.
     */
    public void renderFrame() {
        var g = canvas.getGraphicsContext2D();
        var videoImage = bufferFormat.getVideoImage();

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

    private void initialize() {
        this.canvas.widthProperty().addListener(event -> renderFrame());
        this.canvas.heightProperty().addListener(event -> renderFrame());
    }
}
