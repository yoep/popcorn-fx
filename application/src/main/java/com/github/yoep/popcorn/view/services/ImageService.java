package com.github.yoep.popcorn.view.services;

import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import javafx.scene.image.Image;
import lombok.RequiredArgsConstructor;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;
import org.springframework.web.client.RestTemplate;

import java.io.ByteArrayInputStream;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Service
@RequiredArgsConstructor
public class ImageService {
    private final RestTemplate restTemplate;

    /**
     * Try to load the fanart image for the given {@link Media}.
     * This uses the Spring rest template for following redirect with 3xx.
     *
     * @param media The media to load the fanart image for.
     * @return Returns the fanart image if available, else {@link Optional#empty()}.
     */
    @Async
    public CompletableFuture<Optional<Image>> loadFanart(Media media) {
        Assert.notNull(media, "media cannot be null");
        var image = Optional.ofNullable(media.getImages())
                .map(Images::getFanart)
                .filter(e -> !e.equalsIgnoreCase("n/a"))
                .map(this::internalLoad)
                .map(this::convertToImage);

        return CompletableFuture.completedFuture(image);
    }

    /**
     * Try to load the poster image for the given {@link Media}.
     * This uses the Spring rest template for following redirect with 3xx.
     *
     * @param media The media to load the poster image for.
     * @return Returns the poster image if available, else {@link Optional#empty()}.
     */
    @Async
    public CompletableFuture<Optional<Image>> loadPoster(Media media) {
        Assert.notNull(media, "media cannot be null");
        Optional<Image> image = Optional.ofNullable(media.getImages())
                .map(Images::getPoster)
                .filter(e -> !e.equalsIgnoreCase("n/a"))
                .map(this::internalLoad)
                .map(this::convertToImage);

        return CompletableFuture.completedFuture(image);
    }

    /**
     * Load the given image.
     * This uses the Spring rest template for following redirect with 3xx.
     *
     * @param url The image url to load.
     * @return Returns the image data.
     */
    @Async
    public CompletableFuture<Image> load(String url) {
        Assert.notNull(url, "url cannot be null");
        byte[] image = internalLoad(url);

        return CompletableFuture.completedFuture(convertToImage(image));
    }

    private Image convertToImage(byte[] imageData) {
        var inputStream = new ByteArrayInputStream(imageData);

        return new Image(inputStream);
    }

    private byte[] internalLoad(String uri) {
        return restTemplate.getForObject(uri, byte[].class);
    }
}
