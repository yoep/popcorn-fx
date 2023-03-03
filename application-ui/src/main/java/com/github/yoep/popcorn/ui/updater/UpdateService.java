package com.github.yoep.popcorn.ui.updater;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.updater.UpdateCallback;
import com.github.yoep.popcorn.backend.updater.UpdateState;
import com.github.yoep.popcorn.backend.updater.VersionInfo;
import javafx.beans.property.SimpleObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
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
    public static final String UPDATE_STATE_PROPERTY = "state";

    private final PlatformProvider platformProvider;
    private final TaskExecutor taskExecutor;

    private final SimpleObjectProperty<UpdateState> state = new SimpleObjectProperty<>(this, UPDATE_STATE_PROPERTY, UpdateState.CHECKING_FOR_NEW_VERSION);
    private final Queue<UpdateCallback> listeners = new ConcurrentLinkedDeque<>();
    private final UpdateCallback callback = createCallback();

    //region Properties

    /**
     * Get the known update information if available.
     *
     * @return Returns the version info if available, else {@link Optional#empty()}.
     */
    public Optional<VersionInfo> getUpdateInfo() {
        return Optional.ofNullable(FxLib.INSTANCE.version_info(PopcornFxInstance.INSTANCE.get()));
    }

    public UpdateState getState() {
        return state.get();
    }

    //endregion

    //region Methods

    public void startUpdateAndExit() {
        taskExecutor.execute(this::doInternalUpdate);
    }

    public void downloadUpdate() {

    }

    public void register(UpdateCallback listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    //endregion

    @PostConstruct
    void init() {
        FxLib.INSTANCE.register_update_callback(PopcornFxInstance.INSTANCE.get(), callback);
    }

    private void doInternalDownload(String downloadUri) {

    }

    private void doInternalUpdate() {
        state.set(UpdateState.INSTALLING);
        platformProvider.exit();
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
