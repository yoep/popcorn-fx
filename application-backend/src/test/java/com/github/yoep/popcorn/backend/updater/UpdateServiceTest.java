package com.github.yoep.popcorn.backend.updater;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import com.github.yoep.popcorn.backend.events.NotificationEvent;
import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
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
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
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
        var event = mock(UpdateCallbackEvent.ByValue.class);
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
        UpdateCallbackEvent.ByValue event = createStateChangedEvent(UpdateState.INSTALLATION_FINISHED);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(1, UpdateCallback.class));
            return null;
        }).when(fxLib).register_update_callback(eq(instance), isA(UpdateCallback.class));
        service.init();

        listenerHolder.get().callback(event);

        verify(platform, timeout(150)).exit(3);
    }

    @Test
    void testCallbackListener_onUpdateAvailable() {
        var text = "lorem";
        var listenerHolder = new AtomicReference<UpdateCallback>();
        var eventHolder = new AtomicReference<NotificationEvent>();
        when(localeText.get(UpdateMessage.UPDATE_AVAILABLE)).thenReturn(text);
        UpdateCallbackEvent.ByValue event = createStateChangedEvent(UpdateState.UPDATE_AVAILABLE);
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(1, UpdateCallback.class));
            return null;
        }).when(fxLib).register_update_callback(eq(instance), isA(UpdateCallback.class));
        eventPublisher.register(NotificationEvent.class, e -> {
            eventHolder.set(e);
            return e;
        });
        service.init();

        listenerHolder.get().callback(event);

        verify(eventPublisher, timeout(150)).publish(isA(InfoNotificationEvent.class));
        var result = eventHolder.get();
        assertEquals(service, result.getSource());
        assertEquals(text, result.getText());
    }

    @Test
    void testStartUpdateAndExit() {
        service.startUpdateInstallation();

        verify(fxLib).install_update(instance);
    }

    @Test
    void testCheckForUpdates() {
        service.checkForUpdates();

        verify(fxLib).check_for_updates(instance);
    }

    private static UpdateCallbackEvent.ByValue createStateChangedEvent(UpdateState state) {
        var event = new UpdateCallbackEvent.ByValue();
        event.tag = UpdateCallbackEvent.Tag.StateChanged;
        event.union = new UpdateCallbackEvent.UpdateEventCUnion.ByValue();
        event.union.state_changed = new UpdateCallbackEvent.StateChangedBody();
        event.union.state_changed.newState = state;
        return event;
    }
}