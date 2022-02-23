package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.updater.UpdateService;
import com.github.yoep.popcorn.ui.updater.UpdateState;
import com.github.yoep.popcorn.ui.updater.VersionInfo;
import com.github.yoep.popcorn.ui.view.listeners.UpdateListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class UpdateSectionService extends AbstractListenerService<UpdateListener> {
    private final UpdateService updateService;

    public void updateAll() {
        onUpdateInfoChanged(updateService.getUpdateInfo().orElse(null));
        onUpdateStateChanged(updateService.getState());
    }

    public void startUpdate() {
        updateService.downloadUpdate();
    }

    @PostConstruct
    void init() {
        updateService.updateInfoProperty().addListener((observable, oldValue, newValue) -> onUpdateInfoChanged(newValue));
        updateService.stateProperty().addListener((observable, oldValue, newValue) -> onUpdateStateChanged(newValue));
    }

    private void onUpdateStateChanged(UpdateState newValue) {
        invokeListeners(e -> e.onUpdateStateChanged(newValue));

        if (newValue == UpdateState.DOWNLOAD_FINISHED) {
            updateService.startUpdateAndExit();
        }
    }

    private void onUpdateInfoChanged(VersionInfo newValue) {
        invokeListeners(e -> e.onUpdateInfoChanged(newValue));
    }
}
