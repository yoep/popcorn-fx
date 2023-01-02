package com.github.yoep.popcorn.backend.media.resume;

import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.resume.models.AutoResume;
import com.github.yoep.popcorn.backend.media.resume.models.VideoTimestamp;
import com.github.yoep.popcorn.backend.storage.StorageService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Arrays;
import java.util.Collections;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class AutoResumeServiceTest {
    @Mock
    private StorageService storageService;
    @InjectMocks
    private AutoResumeService service;

    @Test
    void testGetResumeTimestamp_whenFilenameIsKnown_shouldReturnTheLastTimestamp() {
        var filename = "my-file.mp4";
        var lastKnownTime = 1889900;
        var resume = AutoResume.builder()
                .videoTimestamps(Collections.singletonList(VideoTimestamp.builder()
                        .filename(filename)
                        .lastKnownTime(lastKnownTime)
                        .build()))
                .build();
        when(storageService.read(AutoResumeService.STORAGE_NAME, AutoResume.class)).thenReturn(Optional.of(resume));

        var result = service.getResumeTimestamp(filename);

        assertTrue(result.isPresent(), "Expected a timestamp to be returned");
        assertEquals(lastKnownTime, result.get());
    }

    @Test
    void testGetResumeTimestamp_whenFilenameIsUnknown_shouldReturnEmpty() {
        var filename = "a-random-file-called-ipsum.mp4";
        var resume = AutoResume.builder()
                .videoTimestamps(Collections.emptyList())
                .build();
        when(storageService.read(AutoResumeService.STORAGE_NAME, AutoResume.class)).thenReturn(Optional.of(resume));

        var result = service.getResumeTimestamp(filename);

        assertTrue(result.isEmpty(), "Expected timestamp to not have been found");
    }

    @Test
    void testGetResumeTimestamp_whenIdAndFilenameIsKnown_shouldReturnTheLastTimestamp() {
        var id = "tt12";
        var filename = "lorem.mp4";
        var lastKnownTime = 120000;
        var resume = AutoResume.builder()
                .videoTimestamps(Arrays.asList(VideoTimestamp.builder()
                        .id(id)
                        .filename(filename)
                        .lastKnownTime(lastKnownTime)
                        .build(), VideoTimestamp.builder()
                        .id("tt13")
                        .filename("ipsum.mp4")
                        .lastKnownTime(1298766)
                        .build()))
                .build();
        when(storageService.read(AutoResumeService.STORAGE_NAME, AutoResume.class)).thenReturn(Optional.of(resume));

        var result = service.getResumeTimestamp(id, filename);

        assertTrue(result.isPresent(), "Expected a timestamp to be returned");
        assertEquals(lastKnownTime, result.get());
    }

    @Test
    void testOnClosePlayer_whenTimeIsUnknown_shouldNotCacheEventData() {
        var url = "my-video-url.mp4";
        var media = mock(Media.class);
        var duration = 10 * 60 * 1000;
        var event = new PlayerStoppedEvent(this, url, media, null, PlayerStoppedEvent.UNKNOWN, duration);

        service.onClosePlayer(event);
        service.destroy();

        verify(storageService, times(0)).store(eq(AutoResumeService.STORAGE_NAME), isA(AutoResume.class));
    }

    @Test
    void testOnClosePlayer_whenDurationIsUnknown_shouldNotCacheEventData() {
        var url = "my-video-url.mp4";
        var media = mock(Media.class);
        var time = 3 * 60 * 1000;
        var event = new PlayerStoppedEvent(this, url, media, null, time, PlayerStoppedEvent.UNKNOWN);

        service.onClosePlayer(event);
        service.destroy();

        verify(storageService, times(0)).store(eq(AutoResumeService.STORAGE_NAME), isA(AutoResume.class));
    }

    @Test
    void testOnClosePlayer_whenPercentageWatchedIsBelowThreshold_shouldStoreTheEventData() {
        var url = "to-continue-watching-video.mp4";
        var media = mock(Media.class);
        var duration = 10 * 60 * 1000;
        var time = (long) (duration * 0.5);
        var event = new PlayerStoppedEvent(this, url, media, null, time, duration);
        var expectedAutoResume = AutoResume.builder()
                .videoTimestamps(Collections.singletonList(VideoTimestamp.builder()
                        .filename(url)
                        .lastKnownTime(time)
                        .build()))
                .build();

        service.onClosePlayer(event);
        service.destroy();

        verify(storageService).store(AutoResumeService.STORAGE_NAME, expectedAutoResume);
    }

    @Test
    void testOnClosePlayer_whenFilenameIsAlreadyKnown_shouldOverrideTheEventData() {
        var url = "already-known-video.mp4";
        var media = mock(Media.class);
        var duration = 10 * 60 * 1000;
        var time = 5000;
        var event = new PlayerStoppedEvent(this, url, media, null, time, duration);
        var storedResume = AutoResume.builder()
                .videoTimestamps(Collections.singletonList(VideoTimestamp.builder()
                        .filename(url)
                        .lastKnownTime(2000)
                        .build()))
                .build();
        var expectedAutoResume = AutoResume.builder()
                .videoTimestamps(Collections.singletonList(VideoTimestamp.builder()
                        .filename(url)
                        .lastKnownTime(time)
                        .build()))
                .build();
        when(storageService.read(AutoResumeService.STORAGE_NAME, AutoResume.class)).thenReturn(Optional.of(storedResume));

        service.onClosePlayer(event);
        service.destroy();

        verify(storageService).store(AutoResumeService.STORAGE_NAME, expectedAutoResume);
    }

    @Test
    void testOnClosePlayer_whenPercentageWatchedIsAboveThreshold_shouldNotStoreTheEventData() {
        var url = "fully-watched-video.mp4";
        var media = mock(Media.class);
        var duration = 10 * 60 * 1000;
        var time = (long) (duration * 0.9);
        var event = new PlayerStoppedEvent(this, url, media, null, time, duration);
        var expectedAutoResume = AutoResume.builder().build();

        service.onClosePlayer(event);
        service.destroy();

        verify(storageService).store(AutoResumeService.STORAGE_NAME, expectedAutoResume);
    }

    @Test
    void testOnDestroy_whenCacheHasChanged_shouldStoreAutoResumeOnStorage() {
        var url = "my-video-url.mp4";
        var media = mock(Media.class);
        var time = 2000;
        var duration = 10 * 60 * 1000;
        var event = new PlayerStoppedEvent(this, url, media, null, time, duration);
        var expectedAutoResume = AutoResume.builder()
                .videoTimestamps(Collections.singletonList(VideoTimestamp.builder()
                        .filename(url)
                        .lastKnownTime(time)
                        .build()))
                .build();
        when(storageService.read(AutoResumeService.STORAGE_NAME, AutoResume.class)).thenReturn(Optional.empty());

        service.onClosePlayer(event);
        service.destroy();

        verify(storageService).store(AutoResumeService.STORAGE_NAME, expectedAutoResume);
    }
}