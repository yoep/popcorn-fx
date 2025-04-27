package com.github.yoep.popcorn.backend.updater;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import com.github.yoep.popcorn.backend.events.ShowAboutEvent;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class UpdateService extends AbstractListenerService<UpdateEventListener> {
    private final FxChannel fxChannel;
    private final PlatformProvider platform;
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;

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
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        return Optional.ofNullable(response.getInfo());
                    } else {
                        log.error("Failed to retrieve update information, {}", response.getError());
                        return Optional.empty();
                    }
                });
    }

    public CompletableFuture<Update.State> getState() {
        return fxChannel.send(GetUpdateStateRequest.getDefaultInstance(), GetUpdateStateResponse.parser())
                .thenApply(GetUpdateStateResponse::getState);
    }

    //endregion

    //region Methods

    public void startUpdateInstallation() {
        fxChannel.send(StartUpdateInstallationRequest.getDefaultInstance(), StartUpdateInstallationResponse.parser())
                .thenAccept(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        log.debug("Installation of update has been started");
                    } else {
                        log.warn("Failed to start update installation, {}", response.getError());
                    }
                })
                .exceptionally(ex -> {
                    log.error("Failed to start update installation", ex);
                    return null;
                });
    }

    public void startUpdateDownload() {
        fxChannel.send(StartUpdateDownloadRequest.getDefaultInstance(), StartUpdateDownloadResponse.parser())
                .thenAccept(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        log.debug("Update download has been started");
                    } else {
                        log.warn("Failed to start update download, {}", response.getError());
                    }
                })
                .exceptionally(ex -> {
                    log.error("Failed to start downloading the update", ex);
                    return null;
                });
    }

    public void checkForUpdates() {
        fxChannel.send(RefreshUpdateInfoRequest.getDefaultInstance());
    }

    //endregion

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(UpdateEvent.class), UpdateEvent.parser(), this::onUpdateEvent);
        fxChannel.send(GetUpdateStateRequest.getDefaultInstance(), GetUpdateStateResponse.parser())
                .thenApply(GetUpdateStateResponse::getState)
                .thenAccept(this::onStateChanged)
                .exceptionally(ex -> {
                    log.error("Failed to retrieve update state", ex);
                    return null;
                });
    }

    private void onUpdateEvent(UpdateEvent event) {
        switch (event.getEvent()) {
            case STATE_CHANGED -> {
                this.onStateChanged(event.getStateChanged().getNewState());
                invokeListeners(listener -> listener.onStateChanged(event.getStateChanged()));
            }
            case DOWNLOAD_PROGRESS -> invokeListeners(listener ->
                    listener.onDownloadProgress(event.getDownloadProgress()));
        }
    }

    private void onStateChanged(Update.State state) {
        switch (state) {
            case UPDATE_AVAILABLE -> eventPublisher.publish(new InfoNotificationEvent(this, localeText.get(UpdateMessage.UPDATE_AVAILABLE),
                    () -> eventPublisher.publish(new ShowAboutEvent(this))));
            case INSTALLATION_FINISHED -> platform.exit(3);
        }
    }
}
