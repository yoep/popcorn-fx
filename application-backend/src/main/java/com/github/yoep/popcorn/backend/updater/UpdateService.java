package com.github.yoep.popcorn.backend.updater;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import com.github.yoep.popcorn.backend.events.ShowAboutEvent;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
public class UpdateService implements FxCallback<UpdateEvent> {
    private final FxChannel fxChannel;
    private final PlatformProvider platform;
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;

    private final Queue<FxCallback<UpdateEvent>> listeners = new ConcurrentLinkedDeque<>();

    public UpdateService(FxChannel fxChannel, PlatformProvider platform, EventPublisher eventPublisher, LocaleText localeText) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
        this.platform = platform;
        this.eventPublisher = eventPublisher;
        this.localeText = localeText;
        init();
    }

    //region Properties

    /**
     * Get the known update information if available.
     *
     * @return Returns the version info if available, else {@link Optional#empty()}.
     */
    public CompletableFuture<Optional<Update.VersionInfo>> getUpdateInfo() {
        return fxChannel.send(GetUpdateInfoRequest.getDefaultInstance(), GetUpdateInfoResponse.parser())
                .thenApply(GetUpdateInfoResponse::getInfo)
                .thenApply(Optional::ofNullable);
    }

    public CompletableFuture<Update.State> getState() {
        return fxChannel.send(GetUpdateStateRequest.getDefaultInstance(), GetUpdateStateResponse.parser())
                .thenApply(GetUpdateStateResponse::getState);
    }

    //endregion

    //region Methods

    public void startUpdateInstallation() {

    }

    public void downloadUpdate() {

    }

    public void register(FxCallback<UpdateEvent> listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    public void checkForUpdates() {

    }

    @Override
    public void callback(UpdateEvent message) {
        listeners.forEach(listener -> listener.callback(message));
    }

    //endregion

    private void init() {

    }
}
