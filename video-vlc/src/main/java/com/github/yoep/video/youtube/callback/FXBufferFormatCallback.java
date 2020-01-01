package com.github.yoep.video.youtube.callback;

import javafx.geometry.Rectangle2D;
import javafx.scene.image.PixelBuffer;
import javafx.scene.image.PixelFormat;
import javafx.scene.image.WritableImage;
import javafx.scene.image.WritablePixelFormat;
import lombok.extern.slf4j.Slf4j;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.BufferFormat;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.BufferFormatCallback;
import uk.co.caprica.vlcj.player.embedded.videosurface.callback.format.RV32BufferFormat;

import java.nio.ByteBuffer;

@Slf4j
public class FXBufferFormatCallback implements BufferFormatCallback {
    private final WritablePixelFormat<ByteBuffer> pixelFormat = PixelFormat.getByteBgraPreInstance();

    private PixelBuffer<ByteBuffer> videoPixelBuffer;
    private WritableImage videoImage;
    private Rectangle2D updatedBuffer;

    private int bufferWidth;
    private int bufferHeight;

    public PixelBuffer<ByteBuffer> getVideoPixelBuffer() {
        return videoPixelBuffer;
    }

    public WritableImage getVideoImage() {
        return videoImage;
    }

    public Rectangle2D getUpdatedBuffer() {
        return updatedBuffer;
    }

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
