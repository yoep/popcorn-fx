package com.github.yoep.popcorn.favorites;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.favorites.models.Favorites;
import com.github.yoep.popcorn.media.providers.models.Media;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

@Slf4j
@Service
@RequiredArgsConstructor
public class FavoriteService {
    private static final String NAME = "favorites.json";
    private final List<String> cache = new ArrayList<>();
    private final ObjectMapper objectMapper;

    /**
     * Check if the given media is a favorite of the user.
     *
     * @param media The media to check.
     * @return Returns true if the media is a favorite, else false.
     */
    public boolean isFavorite(Media media) {
        synchronized (cache) {
            return cache.contains(media.getImdbId());
        }
    }

    /**
     * Get the favorites.
     *
     * @return Returns the favorites.
     */
    public Favorites getFavorites() {
        return loadFavorites();
    }

    /**
     * Get all the media favorites.
     *
     * @return Returns the list of media.
     */
    public List<Media> getAll() {
        return loadFavorites().getAll();
    }

    /**
     * Add the given media to the favorites.
     *
     * @param media The media to add.
     */
    public void addToFavorites(Media media) {
        synchronized (cache) {
            cache.add(media.getImdbId());
        }

        Favorites favorites = loadFavorites();
        favorites.add(media);
        save(favorites);
    }

    /**
     * Remove the given media from favorites.
     *
     * @param media The media to remove.
     */
    public void removeFromFavorites(Media media) {
        synchronized (cache) {
            cache.remove(media.getImdbId());
        }

        Favorites favorites = loadFavorites();
        favorites.add(media);
        save(favorites);
    }

    @PostConstruct
    public void init() {
        // cache the favorite keys for the media cards for minimal memory consumption
        log.trace("Caching favorites ID's");
        synchronized (cache) {
            cache.addAll(loadFavorites().getAll().stream()
                    .map(Media::getImdbId)
                    .collect(Collectors.toList()));
        }
    }

    private void save(Favorites favorites) {
        File file = getFile();

        try {
            log.debug("Saving favorites to {}", file.getAbsolutePath());
            FileUtils.writeStringToFile(file, objectMapper.writeValueAsString(favorites), Charset.defaultCharset());
        } catch (IOException ex) {
            log.error("Failed to save the favorites with error" + ex.getMessage(), ex);
        }
    }

    private Favorites loadFavorites() {
        File file = getFile();

        if (file.exists()) {
            try {
                log.debug("Loading favorites from {}", file.getAbsolutePath());

                return objectMapper.readValue(file, Favorites.class);
            } catch (IOException ex) {
                log.error("Unable to read favorites file at " + file.getAbsolutePath(), ex);
            }
        }

        return Favorites.builder().build();
    }

    private File getFile() {
        return new File(PopcornTimeApplication.APP_DIR + NAME);
    }
}
