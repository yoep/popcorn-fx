package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.lib.ByteArray;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.Media;
import javafx.scene.image.Image;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;

/**
 * Image service for loading external images over HTTP/HTTPS.
 * This image service selects the correct image from the {@link Media} items and will handle redirects automatically.
 */
@Slf4j
@RequiredArgsConstructor
public class ImageService {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ExecutorService executorService;

    //region Methods

    /**
     * Get the poster holder image.
     *
     * @return Returns the poster holder image.
     */
    public Image getPosterPlaceholder() {
        return getPosterPlaceholder(0, 0);
    }

    public Image getPosterPlaceholder(double requestedWidth, double requestedHeight) {
        log.debug("Retrieving the poster placeholder");
        try (var bytes = fxLib.poster_placeholder(instance)) {
            return Optional.of(bytes)
                    .map(ByteArray::getBytes)
                    .map(ByteArrayInputStream::new)
                    .map(e -> new Image(e, requestedWidth, requestedHeight, true, true))
                    .get();
        }
    }

    public Image getArtPlaceholder() {
        log.debug("Retrieving the artwork placeholder");
        try (var bytes = fxLib.artwork_placeholder(instance)) {
            return Optional.of(bytes)
                    .map(ByteArray::getBytes)
                    .map(ByteArrayInputStream::new)
                    .map(Image::new)
                    .get();
        }
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
        return CompletableFuture.supplyAsync(() -> {
            log.debug("Loading fanart image for {}", media);
            try (var bytes = fxLib.load_fanart(instance, MediaItem.from(media))) {
                return Optional.of(bytes)
                        .map(ByteArray::getBytes)
                        .map(ByteArrayInputStream::new)
                        .map(Image::new);
            } catch (Exception ex) {
                log.error("Failed to load image, {}", ex.getMessage(), ex);
                return Optional.empty();
            }
        }, executorService);
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
        return CompletableFuture.supplyAsync(() -> {
            log.debug("Loading the poster holder for {}", media);
            try (var bytes = fxLib.load_poster(instance, MediaItem.from(media))) {
                return Optional.of(bytes)
                        .map(ByteArray::getBytes)
                        .map(ByteArrayInputStream::new)
                        .map(e -> new Image(e, width, height, true, true));
            } catch (Exception ex) {
                log.error("Failed to load image, {}", ex.getMessage(), ex);
                return Optional.empty();
            }
        }, executorService);
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
        return CompletableFuture.supplyAsync(() -> {
            try (var bytes = fxLib.load_image(instance, url)) {
                return Optional.ofNullable(bytes)
                        .map(ByteArray::getBytes)
                        .map(ByteArrayInputStream::new)
                        .map(Image::new)
                        .orElseThrow(() -> new ImageException(url, "Failed to load image data"));
            }
        }, executorService);
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
        }, executorService);
    }

    //endregion
}
