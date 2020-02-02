package com.github.yoep.popcorn.media.resume;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ClosePlayerActivity;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.resume.models.AutoResume;
import com.github.yoep.popcorn.media.resume.models.VideoTimestamp;
import javafx.animation.PauseTransition;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.apache.commons.io.FilenameUtils;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class AutoResumeService {
    private static final String NAME = "auto-resume.json";
    private static final int AUTO_RESUME_PERCENTAGE_THRESHOLD = 90;
    private static final int IDLE_TIME = 10;

    private final ActivityManager activityManager;
    private final ObjectMapper objectMapper;
    private final PauseTransition idleTimer = new PauseTransition(Duration.seconds(IDLE_TIME));
    private final Object cacheLock = new Object();

    private AutoResume cache;

    //region Getters

    /**
     * Get the resume timestamp for the given video playback.
     *
     * @param filename The filename of the video.
     * @return Returns the last known timestamp of the video if known, else {@link Optional#empty()}.
     */
    public Optional<Long> getResumeTimestamp(String filename) {
        return getResumeTimestamp(null, filename);
    }

    /**
     * Get the resume timestamp for the given video playback.
     *
     * @param id       The media ID of the video.
     * @param filename The filename of the video.
     * @return Returns the last known timestamp of the video if known, else {@link Optional#empty()}.
     */
    public Optional<Long> getResumeTimestamp(String id, String filename) {
        Assert.hasText(filename, "filename cannot be null");
        loadVideoTimestampsToCache();

        return cache.getVideoTimestamps().stream()
                .filter(timestamp -> isIdMatching(id, timestamp) || isFilenameMatching(filename, timestamp))
                .map(VideoTimestamp::getLastKnownTime)
                .findFirst();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeIdleTimer();
        initializeListeners();
    }

    private void initializeIdleTimer() {
        idleTimer.setOnFinished(e -> onSave());
    }

    private void initializeListeners() {
        activityManager.register(ClosePlayerActivity.class, activity -> {
            var time = activity.getTime();
            var duration = activity.getDuration();

            // check if both the time and duration of the video are known
            // if not, the close activity media is not eligible for being auto resumed
            if (time == ClosePlayerActivity.UNKNOWN || duration == ClosePlayerActivity.UNKNOWN)
                return;

            // check if the duration is longer than 5 mins.
            // if not, assume that the played media was a trailer which we don't want to auto resume
            if (duration < 5 * 60 * 1000)
                return;

            var percentageWatched = ((double) time / duration) * 100;
            var id = activity.getMedia().map(Media::getId).orElse(null);
            var filename = FilenameUtils.getName(activity.getUrl());

            // check if the video is not watched more than auto resume threshold
            // if the video is watched more than the threshold
            // we assume that the video has been fully watched an we're not going to store the video for auto resume
            log.trace("Video playback of \"{}\" ({}) has been played for {}%", filename, id, percentageWatched);
            if (percentageWatched < AUTO_RESUME_PERCENTAGE_THRESHOLD) {
                // add the video the resume storage for later use
                log.debug("Storing filename \"{}\" with last known time \"{}\" as auto resume item for later use", filename, time);

                addVideoTimestamp(VideoTimestamp.builder()
                        .id(id)
                        .filename(filename)
                        .lastKnownTime(activity.getTime())
                        .build());
            } else {
                // we remove the video from the auto resume list as the user has completed video
                // and auto resuming the video is not required anymore the next time
                removeVideoTimestamp(id, filename);
            }
        });
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void destroy() {
        onSave();
    }

    //endregion

    //region Functions

    private void addVideoTimestamp(VideoTimestamp videoTimestamp) {
        loadVideoTimestampsToCache();

        var id = videoTimestamp.getId().orElse(null);
        var filename = videoTimestamp.getFilename();

        // check if the video is already known
        // if so, update the timestamp of the video rather than adding a new item
        cache.getVideoTimestamps().stream()
                .filter(timestamp -> isIdMatching(id, timestamp) || isFilenameMatching(filename, timestamp))
                .findFirst()
                .ifPresentOrElse(
                        existingTimestamp -> {
                            existingTimestamp.setFilename(videoTimestamp.getFilename());
                            existingTimestamp.setLastKnownTime(videoTimestamp.getLastKnownTime());
                        },
                        () -> cache.getVideoTimestamps().add(videoTimestamp)
                );
    }

    private void removeVideoTimestamp(String id, String filename) {
        loadVideoTimestampsToCache();

        // remove all auto resume timestamps that match the given data
        cache.getVideoTimestamps()
                .removeIf(timestamp -> isIdMatching(id, timestamp) || isFilenameMatching(filename, timestamp));
    }

    private boolean isIdMatching(String id, VideoTimestamp timestamp) {
        return timestamp.getId()
                .map(e -> e.equalsIgnoreCase(id))
                .orElse(false);
    }

    private boolean isFilenameMatching(String filename, VideoTimestamp timestamp) {
        return timestamp.getFilename().equalsIgnoreCase(filename);
    }

    private void loadVideoTimestampsToCache() {
        idleTimer.playFromStart();

        // check if the cache is already loaded
        // if so, ignore the load
        synchronized (cacheLock) {
            if (cache != null) {
                log.trace("Not updating auto resume cache as it's already present");
                return;
            }
        }

        File file = getFile();

        if (file.exists()) {
            try {
                log.debug("Loading auto resume timestamps from {}", file.getAbsolutePath());
                synchronized (cacheLock) {
                    cache = objectMapper.readValue(file, AutoResume.class);
                }
            } catch (IOException ex) {
                log.error("Failed to load the auto resume timestamps with error " + ex.getMessage(), ex);
            }
        } else {
            synchronized (cacheLock) {
                cache = AutoResume.builder().build();
            }
        }
    }

    private void save(AutoResume autoResume) {
        File file = getFile();

        try {
            log.debug("Saving auto resume timestamps to {}", file.getAbsolutePath());
            FileUtils.writeStringToFile(file, objectMapper.writeValueAsString(autoResume), Charset.defaultCharset());
        } catch (IOException ex) {
            log.error("Failed to save the auto resume timestamps with error " + ex.getMessage(), ex);
        }
    }

    private File getFile() {
        return new File(PopcornTimeApplication.APP_DIR + NAME);
    }

    private void onSave() {
        if (cache == null)
            return;

        synchronized (cacheLock) {
            save(cache);
            cache = null;
        }
    }

    //endregion
}
