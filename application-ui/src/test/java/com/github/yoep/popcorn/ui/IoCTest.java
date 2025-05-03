package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.ProviderServiceImpl;
import com.github.yoep.popcorn.backend.subtitles.SubtitleServiceImpl;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.IOException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class IoCTest {
    @Mock
    private FxChannel fxChannel;

    @Test
    void testGetInstance() {
        var ioc = new IoC();
        ioc.register(SubtitleServiceImpl.class);
        ioc.registerInstance(fxChannel);
        ioc.registerInstance(Executors.newCachedThreadPool());

        var result = ioc.getInstance(SubtitleServiceImpl.class);

        assertNotNull(result);
    }

    @Test
    void testGetInstance_whenSingleton_shouldReturnSameInstance() {
        var ioc = new IoC();
        ioc.register(SubtitleServiceImpl.class);
        ioc.registerInstance(fxChannel);
        ioc.registerInstance(Executors.newCachedThreadPool());

        var expectedResult = ioc.getInstance(SubtitleServiceImpl.class);
        var result = ioc.getInstance(SubtitleServiceImpl.class);

        assertEquals(expectedResult, result);
    }

    @Test
    void testGetInstances() {
        var ioc = new IoC();
        ioc.register(ProviderServiceImpl.class);
        ioc.registerInstance(fxChannel);
        ioc.registerInstance(Executors.newCachedThreadPool());

        var result = ioc.getInstances(ProviderService.class);

        assertNotNull(result);
        assertEquals(1, result.size());
    }

    @Test
    void testDispose() throws IOException {
        var ioc = new IoC();
        var executorService = mock(ExecutorService.class);
        ioc.registerInstance(fxChannel);
        ioc.registerInstance(executorService);

        ioc.dispose();

        verify(fxChannel).close();
        verify(executorService).shutdownNow();
    }
}