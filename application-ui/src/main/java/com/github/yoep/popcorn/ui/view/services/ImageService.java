package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import javafx.scene.image.Image;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.io.ClassPathResource;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;
import org.springframework.web.client.RestTemplate;

import javax.annotation.PostConstruct;
import java.io.ByteArrayInputStream;
import java.text.MessageFormat;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * Image service for loading external images over HTTP/HTTPS.
 * This image service selects the correct image from the {@link Media} items and will handle redirects automatically.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class ImageService {
    private static final int POSTER_WIDTH = 201;
    private static final int POSTER_HEIGHT = 294;
    private static final String POSTER_HOLDER = "/images/posterholder.png";

    private final RestTemplate restTemplate;
    private Image posterHolder;

    //region Methods

    /**
     * Get the poster holder image.
     *
     * @return Returns the poster holder image.
     */
    public Image getPosterHolder() {
        return posterHolder;
    }

    /**
     * Try to load the fanart image for the given {@link Media}.
     *
     * @param media The media to load the fanart image for.
     * @return Returns the fanart image if available, else {@link Optional#empty()}.
     * @throws ImageException Is thrown when the image data failed to load.
     */
    @Async
    public CompletableFuture<Optional<Image>> loadFanart(Media media) {
        Assert.notNull(media, "media cannot be null");
        var image = Optional.ofNullable(media.getImages())
                .map(Images::getFanart)
                .filter(StringUtils::isNotEmpty)
                .filter(this::isImageUrlKnown)
                .map(this::internalLoad)
                .map(this::convertToImage)
                .filter(this::isSuccessfullyLoaded);

        return CompletableFuture.completedFuture(image);
    }

    /**
     * Try to load the poster image for the given {@link Media}.
     *
     * @param media The media to load the poster image for.
     * @return Returns the poster image if available, else {@link Optional#empty()}.
     * @throws ImageException Is thrown when the image data failed to load.
     */
    @Async
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
    @Async
    public CompletableFuture<Optional<Image>> loadPoster(final Media media, final double width, final double height) {
        Assert.notNull(media, "media cannot be null");
        Optional<Image> image = Optional.ofNullable(media.getImages())
                .map(Images::getPoster)
                .filter(StringUtils::isNotEmpty)
                .filter(this::isImageUrlKnown)
                .map(this::internalLoad)
                .map(e -> this.convertToImage(e, width, height))
                .filter(this::isSuccessfullyLoaded);

        return CompletableFuture.completedFuture(image);
    }

    /**
     * Load the given image.
     *
     * @param url The image url to load.
     * @return Returns the image data.
     * @throws ImageException Is thrown when the image data failed to load.
     */
    @Async
    public CompletableFuture<Image> load(String url) {
        Assert.notNull(url, "url cannot be null");
        byte[] image = internalLoad(url);

        return CompletableFuture.completedFuture(convertToImage(image));
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        loadPosterHolder();
    }

    void loadPosterHolder() {
        try {
            var resource = getPosterHolderResource();

            posterHolder = new Image(resource.getInputStream(), POSTER_WIDTH, POSTER_HEIGHT, true, true);
        } catch (Exception ex) {
            log.error("Failed to load poster holder, " + ex.getMessage(), ex);
        }
    }

    //endregion

    //region Functions

    private boolean isImageUrlKnown(String url) {
        return !url.equalsIgnoreCase("n/a");
    }

    private boolean isSuccessfullyLoaded(Image image) {
        return !image.isError();
    }

    private Image convertToImage(byte[] imageData) {
        return convertToImage(imageData, 0, 0);
    }

    private Image convertToImage(byte[] imageData, double width, double height) {
        var inputStream = new ByteArrayInputStream(imageData);

        return new Image(inputStream, width, height, true, true);
    }

    /**
     * Get the poster holder image resource.
     *
     * @return Returns the image resource.
     */
    static ClassPathResource getPosterHolderResource() {
        return new ClassPathResource(POSTER_HOLDER);
    }

    /**
     * Load the image internally using the rest template as it automatically follows the 3xx redirects.
     *
     * @param url The image url to load.
     * @return Returns the image byte data.
     */
    byte[] internalLoad(String url) {
        try {
            var response = restTemplate.getForEntity(url, byte[].class);

            // check if the response is a success
            if (response.getStatusCode().is2xxSuccessful())
                return response.getBody();

            throw new ImageException(url, MessageFormat.format("expected status 2xx, but got {0} instead", response.getStatusCodeValue()));
        } catch (Exception ex) {
            throw new ImageException(url, ex.getMessage(), ex);
        }
    }

    //endregion
}
