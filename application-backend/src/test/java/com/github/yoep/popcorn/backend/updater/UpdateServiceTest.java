package com.github.yoep.popcorn.backend.updater;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class UpdateServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Mock
    private PlatformProvider platform;
    @InjectMocks
    private UpdateService service;

    @Test
    void testGetUpdateInfo() {
        var version = mock(VersionInfo.class);
        when(fxLib.version_info(instance)).thenReturn(version);

        var result = service.getUpdateInfo();

        assertEquals(Optional.of(version), result);
    }

    @Test
    void testGetState() {
        var state = UpdateState.NO_UPDATE_AVAILABLE;
        when(fxLib.update_state(instance)).thenReturn(state);

        var result = service.getState();

        assertEquals(state, result);
    }

    @Test
    void testRegisterCallback_shouldInvokedListeners() {
        var callback = mock(UpdateCallback.class);
        var listenerHolder = new AtomicReference<UpdateCallback>();
        var event = mock(UpdateEvent.ByValue.class);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(1, UpdateCallback.class));
            return null;
        }).when(fxLib).register_update_callback(eq(instance), isA(UpdateCallback.class));
        service.init();

        service.register(callback);
        listenerHolder.get().callback(event);

        verify(callback, timeout(150)).callback(event);
    }

    @Test
    void testCallbackListener_onUpdateInstalling() {
        var listenerHolder = new AtomicReference<UpdateCallback>();
        var event = new UpdateEvent.ByValue();
        event.tag = UpdateEvent.Tag.StateChanged;
        event.union = new UpdateEvent.UpdateEventCUnion.ByValue();
        event.union.state_changed = new UpdateEvent.StateChangedBody();
        event.union.state_changed.newState = UpdateState.INSTALLING;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(1, UpdateCallback.class));
            return null;
        }).when(fxLib).register_update_callback(eq(instance), isA(UpdateCallback.class));
        service.init();

        listenerHolder.get().callback(event);

        verify(platform, timeout(150)).exit();
    }

    @Test
    void testStartUpdateAndExit() {
        service.startUpdateAndExit();

        verify(fxLib).install_update(instance);
    }
}