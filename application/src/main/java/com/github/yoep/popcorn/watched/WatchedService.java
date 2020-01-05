package com.github.yoep.popcorn.watched;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ClosePlayerActivity;
import com.github.yoep.popcorn.providers.models.Media;
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
    private static final int WATCHED_PERCENTAGE_THRESHOLD = 90;

    private final List<String> cache = new ArrayList<>();
    private final ActivityManager activityManager;
    private final ObjectMapper objectMapper;

    //region Methods

    /**
     * Check if the given media has been watched already.
     *
     * @param media The media to check the watched state for.
     * @return Returns true if the media has already been watched, else false.
     */
    public boolean isWatched(Media media) {
        Assert.notNull(media, "media cannot be null");
        String key = media.getId();

        return isWatched(key);
    }

    /**
     * Add the media item to the watched list.
     *
     * @param media the media item to add.
     */
    public void addToWatchList(Media media) {
        Assert.notNull(media, "media cannot be null");
        String key = media.getId();

        addToWatchList(key);
    }

    /**
     * Remove the media item from the watched list.
     *
     * @param media The media item to remove.
     */
    public void removeFromWatchList(Media media) {
        Assert.notNull(media, "media cannot be null");
        synchronized (cache) {
            cache.remove(media.getId());
        }
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeCache();
        initializeListeners();
    }

    private void initializeCache() {
        cache.addAll(loadWatched());
    }

    private void initializeListeners() {
        activityManager.register(ClosePlayerActivity.class, activity -> {
            // check if the quality is present of the media
            // if not, the played video was the trailer of the media
            if (activity.getQuality().isEmpty())
                return;

            long time = activity.getTime();
            long duration = activity.getDuration();

            // check if both the time and duration of the video are known
            // if not, the close activity media is not eligible for being auto marked as watched
            if (time == ClosePlayerActivity.UNKNOWN || duration == ClosePlayerActivity.UNKNOWN)
                return;

            double percentageWatched = ((double) time / duration) * 100;
            Media media = activity.getMedia();

            // check if the media has been watched for the percentage threshold
            // if so, mark the media as watched
            log.trace("Media playback of \"{}\" ({}) has been watched for {}%", media.getTitle(), media.getId(), percentageWatched);
            if (percentageWatched >= WATCHED_PERCENTAGE_THRESHOLD) {
                log.debug("Marking media \"{}\" ({}) automatically as watched", media.getTitle(), media.getId());
                addToWatchList(media);
            }
        });
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void destroy() {
        save(cache);
    }

    //endregion

    //region Functions

    private void addToWatchList(String key) {
        // prevent keys from being added twice
        if (cache.contains(key))
            return;

        synchronized (cache) {
            cache.add(key);
        }
    }

    private boolean isWatched(String key) {
        synchronized (cache) {
            return cache.contains(key);
        }
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

    //endregion
}
