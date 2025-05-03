package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import com.google.protobuf.ByteString;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * Image service for loading external images over HTTP/HTTPS.
 * This image service selects the correct image from the {@link Media} items and will handle redirects automatically.
 */
@Slf4j
public class ImageService {
    private final FxChannel fxChannel;

    public ImageService(FxChannel fxChannel) {
        this.fxChannel = fxChannel;
    }

    //region Methods

    /**
     * Get the poster holder image.
     *
     * @return Returns the poster holder image.
     */
    public CompletableFuture<Image> getPosterPlaceholder() {
        return getPosterPlaceholder(0, 0);
    }

    public CompletableFuture<Image> getPosterPlaceholder(double requestedWidth, double requestedHeight) {
        log.debug("Retrieving the poster placeholder");
        return fxChannel.send(GetPosterPlaceholderRequest.getDefaultInstance(), GetPosterPlaceholderResponse.parser())
                .thenApply(response -> Optional.of(response)
                        .map(GetPosterPlaceholderResponse::getImage)
                        .map(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Image::getData)
                        .map(ByteString::newInput)
                        .map(e -> new Image(e, requestedWidth, requestedHeight, true, true))
                        .get());
    }

    public CompletableFuture<Image> getArtPlaceholder() {
        log.debug("Retrieving the artwork placeholder");
        return fxChannel.send(GetArtworkPlaceholderRequest.getDefaultInstance(), GetArtworkPlaceholderResponse.parser())
                .thenApply(response -> Optional.of(response)
                        .map(GetArtworkPlaceholderResponse::getImage)
                        .map(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Image::getData)
                        .map(ByteString::newInput)
                        .map(Image::new)
                        .get());
    }

    /**
     * Try to load the fanart image for the given {@link Media}.
     *
     * @param media The media to load the fanart image for.
     * @return Returns the fanart image if available, else {@link Optional#empty()}.
     * @throws ImageException Is thrown when the image data failed to load.
     */
    public CompletableFuture<Optional<Image>> loadFanart(Media media) {
        Objects.requireNonNull(media, "media cannot be null");
        return fxChannel.send(GetFanartRequest.newBuilder()
                        .setMedia(MediaHelper.getItem(media))
                        .build(), GetFanartResponse.parser())
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        return Optional.of(response)
                                .map(GetFanartResponse::getImage)
                                .map(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Image::getData)
                                .map(ByteString::newInput)
                                .map(Image::new);
                    } else {
                        log.warn("Failed to load fanart, {}", response.getError());
                        return Optional.empty();
                    }
                });
    }

    /**
     * Try to load the poster image for the given {@link Media}.
     *
     * @param media The media to load the poster image for.
     * @return Returns the poster image if available, else {@link Optional#empty()}.
     * @throws ImageException Is thrown when the image data failed to load.
     */
    public CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return loadPoster(media, 0, 0);
    }

    /**
     * Try to load the poster image for the given {@link Media}.
     * The image will be resized to given max width & height while preserving the ratio.
     *
     * @param media  The media to load the poster image for.
     * @param width  The max width the image should be resized to.
     * @param height The max height the image should be resized to.
     * @return Returns the poster image if available, else {@link Optional#empty()}.
     * @throws ImageException Is thrown when the image data failed to load.
     */
    public CompletableFuture<Optional<Image>> loadPoster(final Media media, final double width, final double height) {
        Objects.requireNonNull(media, "media cannot be null");
        return fxChannel.send(GetPosterRequest.newBuilder()
                        .setMedia(MediaHelper.getItem(media))
                        .build(), GetPosterResponse.parser())
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        return Optional.of(response)
                                .map(GetPosterResponse::getImage)
                                .map(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Image::getData)
                                .map(ByteString::newInput)
                                .map(Image::new);
                    } else {
                        log.warn("Failed to load poster, {}", response.getError());
                        return Optional.empty();
                    }
                });
    }

    /**
     * Load the given image.
     *
     * @param url The image url to load.
     * @return Returns the image data.
     * @throws ImageException Is thrown when the image data failed to load.
     */
    public CompletableFuture<Image> load(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        return fxChannel.send(GetImageRequest.newBuilder()
                        .setUrl(url)
                        .build(), GetImageResponse.parser())
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        return Optional.of(response)
                                .map(GetImageResponse::getImage)
                                .map(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Image::getData)
                                .map(ByteString::newInput)
                                .map(Image::new)
                                .get();
                    } else {
                        log.warn("Failed to load image url \"{}\", {}", url, response.getError());
                        throw new ImageException(url, "Failed to load image data");
                    }
                });
    }

    /**
     * Load an image from the images resources.
     *
     * @param url The images url to retrieve from the resources.
     * @return Returns the loaded image resource.
     * @throws ImageException Is thrown when the resource image failed to load.
     */
    public CompletableFuture<Image> loadResource(String url) {
        Objects.requireNonNull(url, "url cannot be empty");
        return CompletableFuture.supplyAsync(() -> {
            var classpathUrl = "/images/" + url;

            try (var resource = ImageService.class.getResourceAsStream(classpathUrl)) {
                return new Image(resource);
            } catch (IOException ex) {
                throw new ImageException(classpathUrl, ex.getMessage(), ex);
            }
        });
    }

    //endregion
}
