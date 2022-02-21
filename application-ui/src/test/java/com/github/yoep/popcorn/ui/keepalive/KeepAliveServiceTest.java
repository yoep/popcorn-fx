package com.github.yoep.popcorn.ui.keepalive;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationOptions;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class KeepAliveServiceTest {
    @Mock
    private OptionsService optionsService;
    @Mock
    private PlatformProvider platformProvider;
    @InjectMocks
    private KeepAliveService keepAliveService;

    @Test
    void testInit_whenDisabledOptionIsPresent_shouldNotInvokeKeyStroke() {
        var options = ApplicationOptions.builder()
                .keepAliveDisabled(true)
                .build();
        when(optionsService.options()).thenReturn(options);

        keepAliveService.handleScreensaver();

        verify(platformProvider, times(0)).disableScreensaver();
    }

    @Test
    void testInit_whenServiceIsEnabled_shouldInvokeKeyStroke() {
        var options = ApplicationOptions.builder()
                .keepAliveDisabled(false)
                .build();
        when(optionsService.options()).thenReturn(options);

        keepAliveService.handleScreensaver();

        verify(platformProvider).disableScreensaver();
    }
}
