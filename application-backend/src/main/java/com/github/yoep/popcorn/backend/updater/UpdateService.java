package com.github.yoep.popcorn.backend.updater;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
@Service
@RequiredArgsConstructor
public class UpdateService {
    private final FxLib fxLib;
    private final PopcornFx instance;

    private final Queue<UpdateCallback> listeners = new ConcurrentLinkedDeque<>();
    private final UpdateCallback callback = createCallback();

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

    public void startUpdateAndExit() {
    }

    public void downloadUpdate() {
        fxLib.download_update(instance);
    }

    public void register(UpdateCallback listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    //endregion

    @PostConstruct
    void init() {
        fxLib.register_update_callback(instance, callback);
    }

    private UpdateCallback createCallback() {
        return event -> {
            log.debug("Received update callback event {}", event);
            try {
                listeners.forEach(e -> e.callback(event));
            } catch (Exception ex) {
                log.error("Failed to invoke update callbacks, {}", ex.getMessage(), ex);
            }
        };
    }
}
