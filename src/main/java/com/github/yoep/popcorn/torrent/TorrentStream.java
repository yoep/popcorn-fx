package com.github.yoep.popcorn.torrent;

import com.frostwire.jlibtorrent.SessionManager;
import com.frostwire.jlibtorrent.SettingsPack;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.core.io.Resource;
import org.springframework.core.io.support.PathMatchingResourcePatternResolver;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.io.File;

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
            initNativeLibraries();
            initSessionManager();
            initSettings();
            this.initialized = true;
            log.debug("TorrentStream initialized");
        });
    }

    private void initNativeLibraries() {
        try {
            PathMatchingResourcePatternResolver resourcePatternResolver = new PathMatchingResourcePatternResolver(this.getClass().getClassLoader());
            Resource[] resources = resourcePatternResolver.getResources("classpath*:**/jlibtorrent-*");
            String workingDir = System.getProperty("user.dir");

            for (Resource resource : resources) {
                String filename = resource.getFilename();
                File destination = new File(workingDir + File.separator + filename);

                if (!destination.exists())
                    FileUtils.copyInputStreamToFile(resource.getInputStream(), destination);
            }
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void initSessionManager() {
        this.torrentSession = new SessionManager();
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
