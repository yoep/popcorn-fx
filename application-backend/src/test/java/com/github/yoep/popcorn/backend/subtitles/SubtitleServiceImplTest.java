package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfoSet;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.ArrayList;
import java.util.Collections;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class SubtitleServiceImplTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private SubtitleServiceImpl service;

    @Test
    void testNone() {
        var none = new SubtitleInfo();
        none.language = SubtitleLanguage.NONE;
        when(fxLib.subtitle_none()).thenReturn(none);

        var result = service.none();

        assertEquals(none, result);
    }

    @Test
    void testCustom() {
        var custom = new SubtitleInfo();
        custom.language = SubtitleLanguage.CUSTOM;
        when(fxLib.subtitle_custom()).thenReturn(custom);

        var result = service.custom();

        assertEquals(custom, result);
    }

    @Test
    void testGetDefaultOrInterfaceLanguage_whenListIsEmpty_shouldReturnNone() {
        var none = mock(SubtitleInfo.class);
        when(fxLib.subtitle_none()).thenReturn(none);

        var result = service.getDefaultOrInterfaceLanguage(new ArrayList<>());

        assertEquals(none, result);
        verify(fxLib, times(0)).select_or_default_subtitle(eq(instance), isA(SubtitleInfoSet.ByReference.class));
    }

    @Test
    void testGetDefaultOrInterfaceLanguage() {
        var subtitleFile = mock(SubtitleFile.class);
        var subtitle = mock(SubtitleInfo.class);
        when(subtitle.getImdbId()).thenReturn("tt111111");
        when(subtitle.getLanguage()).thenReturn(SubtitleLanguage.ENGLISH);
        when(subtitle.getFiles()).thenReturn(Collections.singletonList(subtitleFile));
        when(subtitle.getLen()).thenReturn(1);

        service.getDefaultOrInterfaceLanguage(Collections.singletonList(subtitle));

        verify(fxLib).select_or_default_subtitle(eq(instance), isA(SubtitleInfoSet.ByReference.class));
    }

    @Test
    void testSubtitleEventCallback() throws ExecutionException, InterruptedException, TimeoutException {
        var eventFuture = new CompletableFuture<SubtitleEvent.ByValue>();
        var callbackHolder = new AtomicReference<SubtitleEventCallback>();
        var event = mock(SubtitleEvent.ByValue.class);
        doAnswer(invocation -> {
            callbackHolder.set(invocation.getArgument(1));
            return null;
        }).when(fxLib).register_subtitle_callback(eq(instance), isA(SubtitleEventCallback.class));
        var service = new SubtitleServiceImpl(fxLib, instance);
        service.register(eventFuture::complete);

        var callback = callbackHolder.get();
        callback.callback(event);

        var result = eventFuture.get(200, TimeUnit.MILLISECONDS);
        assertEquals(event, result);
    }

    @Test
    void testDisableSubtitle() {
        service.disableSubtitle();

        verify(fxLib).disable_subtitle(instance);
    }

    @Test
    void testReset() {
        service.reset();

        verify(fxLib).reset_subtitle(instance);
    }
}