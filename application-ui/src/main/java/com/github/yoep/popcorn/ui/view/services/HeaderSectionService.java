package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.updater.UpdaterService;
import com.github.yoep.popcorn.ui.view.listeners.HeaderSectionListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class HeaderSectionService extends AbstractListenerService<HeaderSectionListener> {
    private final UpdaterService updaterService;

    public void executeUpdate() {
        updaterService.runUpdate();
    }

    @PostConstruct
    void init() {
        updaterService.updateAvailableProperty().addListener((observableValue, oldValue, newValue) -> onUpdateAvailableChanged(newValue));
    }

    private void onUpdateAvailableChanged(Boolean newValue) {
        invokeListeners(e -> e.onUpdateAvailableChanged(newValue));
    }
}
