package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.ArrayList;
import java.util.Collections;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class SubtitleServiceImplTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Mock
    private ExecutorService executorService;
    @InjectMocks
    private SubtitleServiceImpl service;

    @Test
    void testNone() {
        var language = SubtitleLanguage.NONE;
        var none = new SubtitleInfo.ByReference();
        var expectedResult = com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo.builder()
                .language(language)
                .files(new SubtitleFile[0])
                .build();
        none.language = language;
        when(fxLib.subtitle_none()).thenReturn(none);

        var result = service.none();

        assertEquals(expectedResult, result);
    }

    @Test
    void testCustom() {
        var language = SubtitleLanguage.CUSTOM;
        var custom = new SubtitleInfo.ByReference();
        var expectedResult = com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo.builder()
                .language(language)
                .files(new SubtitleFile[0])
                .build();
        custom.language = language;
        when(fxLib.subtitle_custom()).thenReturn(custom);

        var result = service.custom();

        assertEquals(expectedResult, result);
    }

    @Test
    void testGetDefaultOrInterfaceLanguage_whenListIsEmpty_shouldReturnNone() {
        var language = SubtitleLanguage.NONE;
        var expectedResult = com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo.builder()
                .language(language)
                .files(new SubtitleFile[0])
                .build();
        var subtitleFfi = SubtitleInfo.ByReference.from(expectedResult);
        when(fxLib.subtitle_none()).thenReturn(subtitleFfi);

        var result = service.getDefaultOrInterfaceLanguage(new ArrayList<>());

        assertEquals(expectedResult, result);
        verify(fxLib, times(0)).select_or_default_subtitle(eq(instance), isA(SubtitleInfoSet.ByReference.class));
    }

    @Test
    void testGetDefaultOrInterfaceLanguage() {
        var subtitleFile = mock(SubtitleFile.class);
        var subtitle = com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo.builder()
                .imdbId("tt111111")
                .language(SubtitleLanguage.ENGLISH)
                .files(new SubtitleFile[]{subtitleFile})
                .build();
        var subtitleFfi = SubtitleInfo.ByReference.from(subtitle);
        when(fxLib.select_or_default_subtitle(eq(instance), isA(SubtitleInfoSet.ByReference.class))).thenReturn(subtitleFfi);

        service.getDefaultOrInterfaceLanguage(Collections.singletonList(subtitle));

        verify(fxLib).select_or_default_subtitle(eq(instance), isA(SubtitleInfoSet.ByReference.class));
    }

    @Test
    void testSubtitleEventCallback() throws ExecutionException, InterruptedException, TimeoutException {
        var eventFuture = new CompletableFuture<com.github.yoep.popcorn.backend.subtitles.model.SubtitleEvent>();
        var callbackHolder = new AtomicReference<SubtitleEventCallback>();
        var event = mock(SubtitleEvent.ByValue.class);
        var expectedResult = com.github.yoep.popcorn.backend.subtitles.model.SubtitleEvent.from(event);
        doAnswer(invocation -> {
            callbackHolder.set(invocation.getArgument(1));
            return null;
        }).when(fxLib).register_subtitle_callback(eq(instance), isA(SubtitleEventCallback.class));
        var service = new SubtitleServiceImpl(fxLib, instance, executorService);
        service.register(eventFuture::complete);

        var callback = callbackHolder.get();
        callback.callback(event);

        var result = eventFuture.get(200, TimeUnit.MILLISECONDS);
        assertEquals(expectedResult, result);
    }

    @Test
    void testDisableSubtitle() {
        service.disableSubtitle();
        verify(fxLib).update_subtitle_preference(instance, SubtitlePreference.ByReference.disabled());
    }

    @Test
    void testReset() {
        service.reset();

        verify(fxLib).reset_subtitle(instance);
    }
}