package com.github.yoep.popcorn.backend.updater;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import com.github.yoep.popcorn.backend.events.NotificationEvent;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class UpdateServiceTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private PlatformProvider platform;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    private UpdateService service;

    private final AtomicReference<FxCallback<UpdateEvent>> subscriptionHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            subscriptionHolder.set(invocation.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(UpdateEvent.class)), isA(Parser.class), isA(FxCallback.class));
        when(fxChannel.send(isA(GetUpdateStateRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(GetUpdateStateResponse.newBuilder()
                .setState(Update.State.NO_UPDATE_AVAILABLE)
                .build()));

        service = new UpdateService(fxChannel, platform, eventPublisher, localeText);
    }

    @Test
    void testGetUpdateInfo() {
        var version = mock(Update.VersionInfo.class);
        when(fxChannel.send(isA(GetUpdateInfoRequest.class), isA(Parser.class)))
                .thenReturn(CompletableFuture.completedFuture(GetUpdateInfoResponse.newBuilder()
                        .setInfo(version)
                        .build()));

        var result = service.getUpdateInfo().resultNow();

        assertEquals(Optional.of(version), result);
    }

    @Test
    void testGetState() {
        var state = Update.State.NO_UPDATE_AVAILABLE;
        when(fxChannel.send(isA(GetUpdateStateRequest.class), isA(Parser.class)))
                .thenReturn(CompletableFuture.completedFuture(GetUpdateStateResponse.newBuilder()
                        .setState(state)
                        .build()));

        var result = service.getState().resultNow();

        assertEquals(state, result);
    }

    @Test
    void testRegisterCallback_shouldInvokedListeners() {
        var stateChanged = UpdateEvent.StateChanged.newBuilder()
                .setNewState(Update.State.NO_UPDATE_AVAILABLE)
                .build();
        var event = UpdateEvent.newBuilder()
                .setEvent(UpdateEvent.Event.STATE_CHANGED)
                .setStateChanged(stateChanged)
                .build();
        var listener = mock(UpdateEventListener.class);
        service.addListener(listener);

        subscriptionHolder.get().callback(event);

        verify(listener).onStateChanged(stateChanged);
    }

    @Test
    void testCallbackListener_onUpdateInstalling() {
        var event = UpdateEvent.newBuilder()
                .setEvent(UpdateEvent.Event.STATE_CHANGED)
                .setStateChanged(UpdateEvent.StateChanged.newBuilder()
                        .setNewState(Update.State.INSTALLATION_FINISHED)
                        .build())
                .build();

        subscriptionHolder.get().callback(event);

        verify(platform, timeout(150)).exit(3);
    }

    @Test
    void testCallbackListener_onUpdateAvailable() {
        var eventHolder = new AtomicReference<NotificationEvent>();
        var event = UpdateEvent.newBuilder()
                .setEvent(UpdateEvent.Event.STATE_CHANGED)
                .setStateChanged(UpdateEvent.StateChanged.newBuilder()
                        .setNewState(Update.State.UPDATE_AVAILABLE)
                        .build())
                .build();
        var text = "lorem";
        when(localeText.get(UpdateMessage.UPDATE_AVAILABLE)).thenReturn(text);
        eventPublisher.register(NotificationEvent.class, e -> {
            eventHolder.set(e);
            return e;
        });

        subscriptionHolder.get().callback(event);

        verify(eventPublisher, timeout(150)).publish(isA(InfoNotificationEvent.class));
        var result = eventHolder.get();
        assertEquals(service, result.getSource());
        assertEquals(text, result.getText());
    }

    @Test
    void testStartUpdateInstallation() {
        when(fxChannel.send(isA(StartUpdateInstallationRequest.class), isA(Parser.class)))
                .thenReturn(CompletableFuture.completedFuture(StartUpdateInstallationResponse.newBuilder()
                        .setResult(Response.Result.OK)
                        .build()));

        service.startUpdateInstallation();

        verify(fxChannel).send(isA(StartUpdateInstallationRequest.class), isA(Parser.class));
    }

    @Test
    void testStartUpdateDownload() {
        when(fxChannel.send(isA(StartUpdateDownloadRequest.class), isA(Parser.class)))
                .thenReturn(CompletableFuture.completedFuture(StartUpdateDownloadResponse.newBuilder()
                        .setResult(Response.Result.OK)
                        .build()));

        service.startUpdateDownload();

        verify(fxChannel).send(isA(StartUpdateDownloadRequest.class), isA(Parser.class));
    }

    @Test
    void testStartUpdateDownloadFailed() {
        when(fxChannel.send(isA(StartUpdateDownloadRequest.class), isA(Parser.class)))
                .thenReturn(CompletableFuture.completedFuture(StartUpdateDownloadResponse.newBuilder()
                        .setResult(Response.Result.ERROR)
                        .setError(Update.Error.newBuilder()
                                .setType(Update.Error.Type.INVALID_DOWNLOAD_URL)
                                .build())
                        .build()));

        service.startUpdateDownload();

        verify(fxChannel).send(isA(StartUpdateDownloadRequest.class), isA(Parser.class));
    }

    @Test
    void testCheckForUpdates() {
        service.checkForUpdates();

        verify(fxChannel).send(isA(RefreshUpdateInfoRequest.class));
    }
}