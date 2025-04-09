package com.github.yoep.popcorn.backend.loader;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class LoaderServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Mock
    private EventPublisher eventPublisher;
    @InjectMocks
    private LoaderService service;

    @Test
    void testLoad() {
        var url = "magnet:?xt=urn:btih:9a5c24e8164dfe5a98d2437b7f4d6ec9a7e2e045&dn=Another%20Example%20File&tr=http%3A%2F%2Ftracker.anotherexample" +
                ".com%3A56789%2Fannounce&xl=987654321&sf=Another%20Folder";
        var handle = 548755L;
        when(fxLib.loader_load(isA(PopcornFx.class), isA(String.class))).thenReturn(handle);

        service.load(url);

        verify(fxLib).loader_load(instance, url);
        assertEquals(service.lastLoaderHandle, handle);
    }

    @Test
    void testCancel() {
        var handle = 8455L;
        service.lastLoaderHandle = handle;

        service.cancel();

        verify(fxLib).loader_cancel(instance, handle);
    }
}