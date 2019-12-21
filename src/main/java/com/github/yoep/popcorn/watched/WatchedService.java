package com.github.yoep.popcorn.watched;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.media.providers.models.Media;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import static java.util.Arrays.asList;

@Slf4j
@Service
@RequiredArgsConstructor
public class WatchedService {
    private static final String NAME = "watched.json";
    private final List<String> cache = new ArrayList<>();
    private final ObjectMapper objectMapper;

    /**
     * Check if the given media has been watched already.
     *
     * @param media The media to check the watched state for.
     * @return Returns true if the media has already been watched, else false.
     */
    public boolean isWatched(Media media) {
        Assert.notNull(media, "media cannot be null");
        synchronized (cache) {
            return cache.contains(media.getImdbId());
        }
    }

    /**
     * Add the media item to the watched list.
     *
     * @param media the media item to add.
     */
    public void addToWatchList(Media media) {
        Assert.notNull(media, "media cannot be null");
        String key = media.getImdbId();

        // prevent media item from added twice
        if (cache.contains(key))
            return;

        synchronized (cache) {
            cache.add(key);
        }
    }

    /**
     * Remove the media item from the watched list.
     *
     * @param media The media item to remove.
     */
    public void removeFromWatchList(Media media) {
        Assert.notNull(media, "media cannot be null");
        synchronized (media) {
            cache.remove(media.getImdbId());
        }
    }

    @PostConstruct
    private void init() {
        cache.addAll(loadWatched());
    }

    @PreDestroy
    private void destroy() {
        save(cache);
    }

    private void save(List<String> watched) {
        File file = getFile();

        try {
            log.debug("Saving watched items to {}", file.getAbsolutePath());
            FileUtils.writeStringToFile(file, objectMapper.writeValueAsString(watched), Charset.defaultCharset());
        } catch (IOException ex) {
            log.error("Failed to save the watched items with error" + ex.getMessage(), ex);
        }
    }

    private List<String> loadWatched() {
        File file = getFile();

        if (file.exists()) {
            try {
                log.debug("Loading watched items from {}", file.getAbsolutePath());

                return asList(objectMapper.readValue(file, String[].class));
            } catch (IOException ex) {
                log.error("Unable to read watched items file at " + file.getAbsolutePath(), ex);
            }
        }

        return Collections.emptyList();
    }

    private File getFile() {
        return new File(PopcornTimeApplication.APP_DIR + NAME);
    }
}
