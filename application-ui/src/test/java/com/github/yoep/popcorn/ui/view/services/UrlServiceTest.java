package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import javafx.application.Application;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class UrlServiceTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private Application application;
    @Mock
    private LocaleText localeText;
    @Mock
    private LoaderService loaderService;
    @InjectMocks
    private UrlService urlService;

    @Test
    void testOpen_whenUrlIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> urlService.open((String) null), "url cannot be null");
    }

    @Test
    void testProcess_whenUrlIsEmpty_shouldReturnFalse() {
        var result = urlService.process("");

        assertFalse(result);
    }

    @Test
    void testProcess_whenUrlDoesNotContainPrefix_shouldReturnFalse() {
        var result = urlService.process("lorem");

        assertFalse(result);
    }

    @Test
    void testProcess_whenUrlIsWebUrl_shouldPublishPlayVideoEvent() {
        var url = "http://youtube.com";

        var result = urlService.process(url);

        assertTrue(result);
        verify(loaderService).load(url);
    }

    @Test
    void testProcess_whenUrlIsMagnetLink_shouldPublishLoadUrlEvent() {
        var url = "magnet://lorem";

        var result = urlService.process(url);

        assertTrue(result);
        verify(loaderService).load(url);
    }
}
