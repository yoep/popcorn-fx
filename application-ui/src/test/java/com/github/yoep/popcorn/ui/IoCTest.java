package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.providers.MovieProviderService;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.ShowProviderService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleServiceImpl;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

@ExtendWith(MockitoExtension.class)
class IoCTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;

    @Test
    void testGetInstance() {
        var ioc = new IoC();
        ioc.register(SubtitleServiceImpl.class);
        ioc.registerInstance(fxLib);
        ioc.registerInstance(instance);

        var result = ioc.getInstance(SubtitleServiceImpl.class);

        assertNotNull(result);
    }

    @Test
    void testGetInstance_whenSingleton_shouldReturnSameInstance() {
        var ioc = new IoC();
        ioc.register(SubtitleServiceImpl.class);
        ioc.registerInstance(fxLib);
        ioc.registerInstance(instance);

        var expectedResult = ioc.getInstance(SubtitleServiceImpl.class);
        var result = ioc.getInstance(SubtitleServiceImpl.class);

        assertEquals(expectedResult, result);
    }

    @Test
    void testGetInstances() {
        var ioc = new IoC();
        ioc.register(MovieProviderService.class);
        ioc.register(ShowProviderService.class);
        ioc.registerInstance(fxLib);
        ioc.registerInstance(instance);

        var result = ioc.getInstances(ProviderService.class);

        assertNotNull(result);
        assertEquals(2, result.size());
    }
}