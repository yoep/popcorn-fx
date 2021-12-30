package com.github.yoep.popcorn.backend.media.resume;

import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.resume.models.AutoResume;
import com.github.yoep.popcorn.backend.media.resume.models.VideoTimestamp;
import com.github.yoep.popcorn.backend.storage.StorageException;
import com.github.yoep.popcorn.backend.storage.StorageService;
import com.github.yoep.popcorn.backend.utils.IdleTimer;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.time.Duration;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class AutoResumeService {
    static final String STORAGE_NAME = "auto-resume.json";
    static final int IDLE_TIME = 10;
    private static final int AUTO_RESUME_PERCENTAGE_THRESHOLD = 85;

    private final StorageService storageService;
    private final IdleTimer idleTimer = new IdleTimer(Duration.ofSeconds(IDLE_TIME));
    private final Object cacheLock = new Object();

    private AutoResume cache;
    private int cacheHash;

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

    @EventListener
    public void onClosePlayer(PlayerStoppedEvent event) {
        var time = event.getTime();
        var duration = event.getDuration();

        // check if both the time and duration of the video are known
        // if not, the close activity media is not eligible for being auto resumed
        if (time == PlayerStoppedEvent.UNKNOWN || duration == PlayerStoppedEvent.UNKNOWN) {
            log.trace("Video player time or duration is UNKNOWN, skipping auto resume check");
            return;
        }

        // check if the duration is longer than 5 mins.
        // if not, assume that the played media was a trailer which we don't want to auto resume
        if (duration < 5 * 60 * 1000)
            return;

        log.trace("Video playback was stopped with last known info: [time: {}, duration: {}]", time, duration);
        var percentageWatched = ((double) time / duration) * 100;
        var id = event.getMedia().map(Media::getId).orElse(null);
        var filename = FilenameUtils.getName(event.getUrl());

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
                    .lastKnownTime(event.getTime())
                    .build());
        } else {
            log.debug("Removing auto resume timestamp of \"{}\" ({}) as it has been fully watched", filename, id);
            // we remove the video from the auto resume list as the user has completed video
            // and auto resuming the video is not required anymore the next time
            removeVideoTimestamp(id, filename);
        }
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeIdleTimer();
    }

    private void initializeIdleTimer() {
        idleTimer.setOnTimeout(this::onSave);
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    void destroy() {
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
        idleTimer.runFromStart();

        // check if the cache is already loaded
        // if so, ignore the load
        synchronized (cacheLock) {
            if (cache != null) {
                log.trace("Not updating auto resume cache as it's already present");
                return;
            }
        }


        log.debug("Loading auto resume timestamps from storage");
        storageService.read(STORAGE_NAME, AutoResume.class)
                .ifPresentOrElse(this::handleStoredAutoResume, this::createNewAutoResume);
    }

    private void handleStoredAutoResume(AutoResume e) {
        synchronized (cacheLock) {
            cache = e;
            cacheHash = cache.hashCode();
        }
    }

    private void createNewAutoResume() {
        synchronized (cacheLock) {
            cache = AutoResume.builder().build();
        }
    }

    private void save(AutoResume autoResume) {
        try {
            log.debug("Saving auto resume timestamps to storage");
            storageService.store(STORAGE_NAME, autoResume);
            log.info("Auto resume file has been saved");
        } catch (StorageException ex) {
            log.error("Failed to save the auto resume timestamps with error " + ex.getMessage(), ex);
        }
    }

    private void onSave() {
        if (cache == null)
            return;

        synchronized (cacheLock) {
            // check if the cache was modified
            // if not, the cache will only be removed from memory but not saved again
            if (cacheHash != cache.hashCode())
                save(cache);

            cache = null;
        }
    }

    //endregion
}
