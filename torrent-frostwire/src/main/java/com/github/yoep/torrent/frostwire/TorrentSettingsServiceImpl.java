package com.github.yoep.torrent.frostwire;

import com.frostwire.jlibtorrent.SettingsPack;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentSettingsService;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import javafx.beans.value.ChangeListener;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;

@Slf4j
public class TorrentSettingsServiceImpl implements TorrentSettingsService {
    private final SettingsPack settings = defaultSettings();
    private final ChangeListener<SessionState> sessionListener = createSessionListener();
    private final TorrentSessionManager sessionManager;

    public TorrentSettingsServiceImpl(TorrentSessionManager sessionManager) {
        Objects.requireNonNull(sessionManager, "sessionManager cannot be null");
        this.sessionManager = sessionManager;
        init();
    }

    //region TorrentSettingsService

    @Override
    public TorrentSettingsService connectionsLimit(int connectionsLimit) {
        settings.connectionsLimit(connectionsLimit);
        applySettings();
        return this;
    }

    @Override
    public TorrentSettingsService downloadRateLimit(int downloadRateLimit) {
        settings.downloadRateLimit(downloadRateLimit);
        applySettings();
        return this;
    }

    @Override
    public TorrentSettingsService uploadRateLimit(int uploadRateLimit) {
        settings.uploadRateLimit(uploadRateLimit);
        applySettings();
        return this;
    }

    //endregion

    //region PostConstruct

    private void init() {
        sessionManager.stateProperty().addListener(sessionListener);
    }

    //endregion

    //region Functions

    private void applySettings() {
        sessionManager
                .getSession()
                .applySettings(settings);
    }

    private ChangeListener<SessionState> createSessionListener() {
        return (observable, oldValue, newValue) -> {
            if (newValue == SessionState.RUNNING) {
                // apply the default settings
                applySettings();
            }
        };
    }

    private SettingsPack defaultSettings() {
        return (new SettingsPack())
                .anonymousMode(true)
                .connectionsLimit(150)
                .downloadRateLimit(0)
                .uploadRateLimit(0)
                .sendBufferWatermark(16)
                .activeDhtLimit(160);
    }

    //endregion
}
