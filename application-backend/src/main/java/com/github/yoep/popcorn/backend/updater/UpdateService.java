package com.github.yoep.popcorn.backend.updater;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import com.github.yoep.popcorn.backend.events.ShowAboutEvent;
import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
public class UpdateService {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final PlatformProvider platform;
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;

    private final Queue<UpdateCallback> listeners = new ConcurrentLinkedDeque<>();
    private final UpdateCallback callback = createCallback();

    public UpdateService(FxLib fxLib, PopcornFx instance, PlatformProvider platform, EventPublisher eventPublisher, LocaleText localeText) {
        Objects.requireNonNull(fxLib, "fxLib cannot be null");
        Objects.requireNonNull(instance, "instance cannot be null");
        this.fxLib = fxLib;
        this.instance = instance;
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
    public Optional<VersionInfo> getUpdateInfo() {
        return Optional.ofNullable(fxLib.version_info(instance));
    }

    public UpdateState getState() {
        return fxLib.update_state(instance);
    }

    //endregion

    //region Methods

    public void startUpdateInstallation() {
        fxLib.install_update(instance);
    }

    public void downloadUpdate() {
        fxLib.download_update(instance);
    }

    public void register(UpdateCallback listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    public void checkForUpdates() {
        fxLib.check_for_updates(instance);
    }

    //endregion

    private void init() {
        fxLib.register_update_callback(instance, callback);
        onStateChanged(fxLib.update_state(instance));
    }

    private void onStateChanged(UpdateState newState) {
        if (Objects.equals(newState, UpdateState.UPDATE_AVAILABLE)) {
            eventPublisher.publish(new InfoNotificationEvent(this, localeText.get(UpdateMessage.UPDATE_AVAILABLE),
                    () -> eventPublisher.publish(new ShowAboutEvent(this))));
        } else if (Objects.equals(newState, UpdateState.INSTALLATION_FINISHED)) {
            platform.exit(3);
        }
    }

    private UpdateCallback createCallback() {
        return event -> new Thread(() -> {
            log.debug("Received update callback event {}", event);
            event.close();

            if (event.getTag() == UpdateCallbackEvent.Tag.StateChanged) {
                onStateChanged(event.getUnion().getState_changed().getNewState());
            }

            try {
                listeners.forEach(e -> e.callback(event));
            } catch (Exception ex) {
                log.error("Failed to invoke update callbacks, {}", ex.getMessage(), ex);
            }
        }).start();
    }
}
