package com.github.yoep.popcorn.ui.keepalive;

import com.github.yoep.popcorn.ui.settings.OptionsService;
import com.github.yoep.popcorn.ui.settings.models.ApplicationOptions;
import com.github.yoep.popcorn.ui.view.services.RobotService;
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
    private RobotService robotService;
    @InjectMocks
    private KeepAliveService keepAliveService;

    @Test
    void testKeepAlive_whenDisabledOptionIsPresent_shouldNotInvokeKeyStroke() {
        var options = ApplicationOptions.builder()
                .keepAliveDisabled(true)
                .build();
        when(optionsService.options()).thenReturn(options);

        keepAliveService.keepAlive();

        verify(robotService, times(0)).pressKey(KeepAliveService.SIGNAL);
    }

    @Test
    void testKeepAlive_whenServiceIsEnabled_shouldInvokeKeyStroke() {
        var options = ApplicationOptions.builder()
                .keepAliveDisabled(false)
                .build();
        when(optionsService.options()).thenReturn(options);

        keepAliveService.keepAlive();

        verify(robotService).pressKey(KeepAliveService.SIGNAL);
    }
}
