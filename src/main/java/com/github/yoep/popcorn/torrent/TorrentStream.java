package com.github.yoep.popcorn.torrent;

import com.frostwire.jlibtorrent.SessionManager;
import com.frostwire.jlibtorrent.SettingsPack;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;

@Slf4j
@Component
@RequiredArgsConstructor
public class TorrentStream {
    private final TaskExecutor taskExecutor;
    private SessionManager torrentSession;
    private boolean initialized;

    public void startStream(String torrentUrl) {

    }

    @PostConstruct
    private void init() {
        taskExecutor.execute(() -> {
            log.trace("Initializing TorrentStream...");
            this.torrentSession = new SessionManager();
            initSettings();
            this.initialized = true;
            log.debug("TorrentStream initialized");
        });
    }

    private void initSettings() {
        SettingsPack settingsPack = (new SettingsPack())
                .anonymousMode(true)
                .connectionsLimit(200)
                .downloadRateLimit(0)
                .uploadRateLimit(1)
                .sendBufferWatermark(16)
                .activeDhtLimit(88);
        this.torrentSession.applySettings(settingsPack);
    }
}
