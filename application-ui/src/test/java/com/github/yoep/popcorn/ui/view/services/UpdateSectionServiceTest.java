package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.updater.*;
import com.github.yoep.popcorn.ui.view.listeners.UpdateListener;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class UpdateSectionServiceTest {
    @Mock
    private UpdateService updateService;
    @Mock
    private UpdateListener listener;
    @InjectMocks
    private UpdateSectionService updateSectionService;

    private final AtomicReference<UpdateCallback> callback = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            callback.set(invocation.getArgument(0, UpdateCallback.class));
            return null;
        }).when(updateService).register(isA(UpdateCallback.class));
    }

    @Test
    void testUpdateInfoListener_whenUpdateInfoIsChanged_shouldInvokedListeners() {
        var versionInfo = VersionInfo.builder().build();
        var event = mock(UpdateCallbackEvent.ByValue.class);
        var union = mock(UpdateCallbackEvent.UpdateEventCUnion.ByValue.class);
        var updateAvailableBody = new UpdateCallbackEvent.UpdateAvailableBody();
        updateAvailableBody.newVersion = versionInfo;
        when(event.getTag()).thenReturn(UpdateCallbackEvent.Tag.UpdateAvailable);
        when(event.getUnion()).thenReturn(union);
        when(union.getUpdate_available()).thenReturn(updateAvailableBody);
        updateSectionService.init();
        updateSectionService.addListener(listener);

        callback.get().callback(event);

        verify(listener).onUpdateInfoChanged(versionInfo);
    }

    @Test
    void testStateListener_whenStateIsChanged_shouldInvokedListeners() {
        var expectedState = UpdateState.DOWNLOADING;
        var event = mock(UpdateCallbackEvent.ByValue.class);
        var union = mock(UpdateCallbackEvent.UpdateEventCUnion.ByValue.class);
        var stateBody = new UpdateCallbackEvent.StateChangedBody();
        stateBody.newState = expectedState;
        when(event.getTag()).thenReturn(UpdateCallbackEvent.Tag.StateChanged);
        when(event.getUnion()).thenReturn(union);
        when(union.getState_changed()).thenReturn(stateBody);
        updateSectionService.init();
        updateSectionService.addListener(listener);

        callback.get().callback(event);

        verify(listener).onUpdateStateChanged(expectedState);
    }
}